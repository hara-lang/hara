//! Data-only validation and exact runtime-variant resolution for generated
//! `package.edn` manifests.
//!
//! This module deliberately stops before class loading, Wasm instantiation, or
//! provider registration. It turns untrusted archive metadata into a verified,
//! deterministic selection that a runtime-specific loader can consume.

use crate::kernel::Form;
use semver::Version;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

mod parse;
#[cfg(test)]
mod tests;

const PACKAGE_FORMAT: &str = "0.0.0-alpha";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PackageRuntime {
    Jvm,
    Wasm,
}

impl PackageRuntime {
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Jvm => "jvm",
            Self::Wasm => "wasm",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageArtifactType {
    Jar,
    Wasm,
    Hta,
}

impl PackageArtifactType {
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Jar => "jar",
            Self::Wasm => "wasm",
            Self::Hta => "hta",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManifestError {
    pub code: &'static str,
    pub detail: String,
}

impl PackageManifestError {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for PackageManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for PackageManifestError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageProvenance {
    pub repository: String,
    pub commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageFile {
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageArtifact {
    pub artifact_type: PackageArtifactType,
    pub path: PathBuf,
    pub sha256: String,
    pub target: String,
    pub abi: String,
    pub entry_point: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageLifecycle {
    pub load_idempotent: bool,
    pub close_idempotent: bool,
    pub session_isolation: bool,
    pub asynchronous: bool,
    pub cancellation: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PackageVariant {
    pub runtime: PackageRuntime,
    pub artifact: PackageArtifact,
    pub required_capabilities: BTreeSet<String>,
    pub host_calls: BTreeSet<String>,
    pub exports: BTreeSet<String>,
    pub dependencies: Option<Form>,
    pub lifecycle: Option<PackageLifecycle>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PackageRuntimeRequirements {
    pub supported_targets: BTreeSet<String>,
    pub supported_abis: BTreeSet<String>,
    pub available_capabilities: BTreeSet<String>,
    pub allowed_host_calls: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageSelection {
    Portable,
    Variant(PackageVariant),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PackageManifest {
    pub format: String,
    pub identity: String,
    pub version: Version,
    pub provenance: Option<PackageProvenance>,
    pub files: BTreeMap<PathBuf, PackageFile>,
    pub variants: BTreeMap<PackageRuntime, PackageVariant>,
    canonical_edn: String,
}

impl PackageManifest {
    pub fn read(path: &Path) -> Result<Self, PackageManifestError> {
        let source = fs::read_to_string(path).map_err(|error| {
            PackageManifestError::new(
                "package/invalid-manifest",
                format!("cannot read {}: {error}", path.display()),
            )
        })?;
        Self::parse(&source)
    }

    pub fn parse(source: &str) -> Result<Self, PackageManifestError> {
        parse::parse_manifest(source)
    }

    pub fn canonical_edn(&self) -> &str {
        &self.canonical_edn
    }

    pub fn select_variant(
        &self,
        runtime: PackageRuntime,
        requirements: &PackageRuntimeRequirements,
    ) -> Result<PackageSelection, PackageManifestError> {
        if self.variants.is_empty() {
            return Ok(PackageSelection::Portable);
        }
        let variant = self.variants.get(&runtime).ok_or_else(|| {
            PackageManifestError::new(
                "package/missing-variant",
                format!(
                    "{} {} has no :{} runtime variant",
                    self.identity,
                    self.version,
                    runtime.keyword()
                ),
            )
        })?;
        if !requirements
            .supported_targets
            .contains(&variant.artifact.target)
        {
            return Err(PackageManifestError::new(
                "package/target-mismatch",
                format!(
                    ":{} artifact target {} is not supported",
                    runtime.keyword(),
                    variant.artifact.target
                ),
            ));
        }
        if !requirements.supported_abis.contains(&variant.artifact.abi) {
            return Err(PackageManifestError::new(
                "package/abi-mismatch",
                format!(
                    ":{} artifact ABI {} is not supported",
                    runtime.keyword(),
                    variant.artifact.abi
                ),
            ));
        }

        let missing_capabilities = difference(
            &variant.required_capabilities,
            &requirements.available_capabilities,
        );
        if !missing_capabilities.is_empty() {
            return Err(PackageManifestError::new(
                "package/capability-denied",
                format!("missing capabilities: {}", missing_capabilities.join(", ")),
            ));
        }
        let denied_host_calls = difference(&variant.host_calls, &requirements.allowed_host_calls);
        if !denied_host_calls.is_empty() {
            return Err(PackageManifestError::new(
                "package/host-call-denied",
                format!("denied host calls: {}", denied_host_calls.join(", ")),
            ));
        }
        Ok(PackageSelection::Variant(variant.clone()))
    }

    pub fn verify_artifact_bytes(
        &self,
        selection: &PackageSelection,
        bytes: &[u8],
    ) -> Result<(), PackageManifestError> {
        let PackageSelection::Variant(variant) = selection else {
            return Err(PackageManifestError::new(
                "package/missing-artifact",
                "portable package selection has no runtime artifact",
            ));
        };
        let file = self.files.get(&variant.artifact.path).ok_or_else(|| {
            PackageManifestError::new(
                "package/missing-artifact",
                format!(
                    "selected artifact is not declared in :files: {}",
                    variant.artifact.path.display()
                ),
            )
        })?;
        verify_bytes(&variant.artifact.path, file, bytes)
    }

    pub fn verify_files_at(&self, root: &Path) -> Result<(), PackageManifestError> {
        for (relative, expected) in &self.files {
            let path = root.join(relative);
            let metadata = fs::symlink_metadata(&path).map_err(|error| {
                PackageManifestError::new(
                    "package/missing-artifact",
                    format!("cannot inspect {}: {error}", relative.display()),
                )
            })?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(PackageManifestError::new(
                    "package/missing-artifact",
                    format!(
                        "declared package file is not a regular file: {}",
                        relative.display()
                    ),
                ));
            }
            if metadata.len() != expected.size {
                return Err(PackageManifestError::new(
                    "package/size-mismatch",
                    format!(
                        "{} has {} bytes, expected {}",
                        relative.display(),
                        metadata.len(),
                        expected.size
                    ),
                ));
            }
            let bytes = fs::read(&path).map_err(|error| {
                PackageManifestError::new(
                    "package/missing-artifact",
                    format!("cannot read {}: {error}", relative.display()),
                )
            })?;
            verify_bytes(relative, expected, &bytes)?;
        }
        Ok(())
    }
}

fn verify_bytes(
    relative: &Path,
    expected: &PackageFile,
    bytes: &[u8],
) -> Result<(), PackageManifestError> {
    if bytes.len() as u64 != expected.size {
        return Err(PackageManifestError::new(
            "package/size-mismatch",
            format!(
                "{} has {} bytes, expected {}",
                relative.display(),
                bytes.len(),
                expected.size
            ),
        ));
    }
    let actual = sha256(bytes);
    if actual != expected.sha256 {
        return Err(PackageManifestError::new(
            "package/digest-mismatch",
            format!(
                "{} has digest {}, expected {}",
                relative.display(),
                actual,
                expected.sha256
            ),
        ));
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let hexadecimal = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("sha256:{hexadecimal}")
}

fn difference(required: &BTreeSet<String>, available: &BTreeSet<String>) -> Vec<String> {
    required.difference(available).cloned().collect()
}
