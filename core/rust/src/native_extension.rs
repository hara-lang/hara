#![cfg(not(target_arch = "wasm32"))]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::extension::ExtensionManifest;
use crate::kernel::{parse, Form};

const MAX_PROJECT_BYTES: u64 = 1024 * 1024;
const MAX_MODULE_BYTES: u64 = 64 * 1024 * 1024;

pub struct ExtensionPackage {
    pub root: PathBuf,
    pub descriptor: PathBuf,
    pub source: String,
    pub manifest: ExtensionManifest,
}

impl ExtensionPackage {
    pub fn load(root: &Path) -> Result<Self, String> {
        let mut packages = packages_in_project(root)?;
        match packages.len() {
            1 => Ok(packages.remove(0)),
            0 => Err(format!(
                "extension/malformed: {} does not declare :project/extensions",
                root.display()
            )),
            count => Err(format!(
                "extension/ambiguous: {} declares {count} extension namespaces",
                root.display()
            )),
        }
    }

    pub fn discover(namespace: &str, roots: &[PathBuf]) -> Result<Option<Self>, String> {
        let mut candidates = Vec::new();
        for root in roots {
            for project in project_manifests(root)? {
                for package in packages_from_manifest(&project)? {
                    if package.manifest.namespace == namespace {
                        candidates.push(package);
                    }
                }
            }
        }
        candidates.sort_by(|left, right| left.descriptor.cmp(&right.descriptor));
        candidates.dedup_by(|left, right| left.descriptor == right.descriptor);
        match candidates.len() {
            0 => Ok(None),
            1 => Ok(candidates.pop()),
            _ => Err(format!(
                "extension/ambiguous: multiple projects export {namespace}: {:?}",
                candidates
                    .iter()
                    .map(|package| &package.descriptor)
                    .collect::<Vec<_>>()
            )),
        }
    }

    pub fn module_bytes(&self) -> Result<Vec<u8>, String> {
        let module =
            self.manifest.module.as_deref().ok_or_else(|| {
                format!("extension/module-unavailable: {}", self.manifest.namespace)
            })?;
        let path = self.resolve(module)?;
        let metadata = path
            .metadata()
            .map_err(|error| format!("extension/module-unavailable: {error}"))?;
        if metadata.len() > MAX_MODULE_BYTES {
            return Err(format!("extension/module-too-large: {}", path.display()));
        }
        fs::read(&path).map_err(|error| format!("extension/module-unavailable: {error}"))
    }

    pub fn declared_files(&self) -> Vec<String> {
        let mut paths = Vec::new();
        if let Some(module) = &self.manifest.module {
            paths.push(module.clone());
        }
        paths.extend(
            self.manifest
                .targets
                .values()
                .map(|target| target.module.clone()),
        );
        paths.extend(self.manifest.assets.clone());
        paths.sort();
        paths.dedup();
        paths
    }

    fn validate_declared_files(&self) -> Result<(), String> {
        for relative in self.declared_files() {
            self.resolve(&relative)?;
        }
        Ok(())
    }

    pub fn resolve(&self, relative: &str) -> Result<PathBuf, String> {
        let root = self
            .root
            .canonicalize()
            .map_err(|error| format!("extension/asset-unavailable: {error}"))?;
        let declaration_root = self.manifest.root.as_deref().unwrap_or(".");
        let path = root
            .join(declaration_root)
            .join(relative)
            .canonicalize()
            .map_err(|error| {
                format!(
                    "extension/asset-unavailable: {}/{} ({error})",
                    self.manifest.namespace, relative
                )
            })?;
        if !path.starts_with(&root) || !path.is_file() {
            return Err(format!("extension/path-denied: {relative}"));
        }
        Ok(path)
    }
}

pub fn configured_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(current) = env::current_dir() {
        for directory in current.ancestors() {
            if directory.join("project.edn").is_file() {
                roots.push(directory.to_path_buf());
                roots.push(directory.join("extensions"));
                break;
            }
        }
    }
    if let Some(configured) = env::var_os("HARA_EXTENSION_PATH") {
        roots.extend(env::split_paths(&configured));
    }
    roots
}

pub fn package_exists(namespace: &str, roots: &[PathBuf]) -> bool {
    ExtensionPackage::discover(namespace, roots)
        .ok()
        .flatten()
        .is_some()
}

fn packages_in_project(root: &Path) -> Result<Vec<ExtensionPackage>, String> {
    let descriptor = if root.is_file() {
        root.to_path_buf()
    } else {
        root.join("project.edn")
    };
    packages_from_manifest(&descriptor)
}

fn packages_from_manifest(descriptor: &Path) -> Result<Vec<ExtensionPackage>, String> {
    let metadata = descriptor
        .metadata()
        .map_err(|error| format!("extension/asset-unavailable: {error}"))?;
    if metadata.len() > MAX_PROJECT_BYTES {
        return Err(format!(
            "extension/malformed {}: project manifest is too large",
            descriptor.display()
        ));
    }
    let project_source = fs::read_to_string(descriptor)
        .map_err(|error| format!("extension/malformed {}: {error}", descriptor.display()))?;
    let Form::Map(project) = parse(&project_source)
        .map_err(|error| format!("extension/malformed {}: {error}", descriptor.display()))?
    else {
        return Err("extension/malformed: project.edn must be a map".into());
    };
    let version = value(&project, "project/version")
        .ok_or("extension/malformed: project.edn is missing :project/version")?;
    let version = scalar(version, "project/version")?;
    let Some(Form::Map(extensions)) = value(&project, "project/extensions") else {
        return Ok(Vec::new());
    };
    let root = descriptor
        .parent()
        .ok_or("extension/root-invalid: project.edn has no parent")?
        .to_path_buf();
    extensions
        .iter()
        .map(|(namespace, declaration)| {
            let namespace = scalar(namespace, "extension namespace")?;
            let Form::Map(declaration) = declaration else {
                return Err(format!(
                    "extension/malformed {}: declaration for {namespace} must be a map",
                    descriptor.display()
                ));
            };
            let mut normalized = declaration.clone();
            normalized.push((
                Form::Keyword("namespace".into()),
                Form::String(namespace.clone()),
            ));
            normalized.push((
                Form::Keyword("version".into()),
                Form::String(version.clone()),
            ));
            let source = Form::Map(normalized).to_string();
            let package = ExtensionPackage {
                root: root.clone(),
                descriptor: descriptor.to_path_buf(),
                manifest: ExtensionManifest::parse(&source, &descriptor.display().to_string())?,
                source,
            };
            package.validate_declared_files()?;
            Ok(package)
        })
        .collect()
}

fn project_manifests(root: &Path) -> Result<Vec<PathBuf>, String> {
    let root = absolute(root)?;
    if root.is_file() {
        return Ok(
            (root.file_name().and_then(|name| name.to_str()) == Some("project.edn"))
                .then_some(root)
                .into_iter()
                .collect(),
        );
    }
    let mut pending = vec![root];
    let mut manifests = Vec::new();
    while let Some(directory) = pending.pop() {
        let manifest = directory.join("project.edn");
        if manifest.is_file() {
            manifests.push(manifest);
            continue;
        }
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        pending.extend(
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| {
                    path.is_dir()
                        && path.file_name().and_then(|name| name.to_str()) != Some("target")
                }),
        );
    }
    manifests.sort();
    Ok(manifests)
}

fn value<'a>(entries: &'a [(Form, Form)], key: &str) -> Option<&'a Form> {
    entries.iter().find_map(|(candidate, value)| {
        matches!(candidate, Form::Keyword(name) if name == key).then_some(value)
    })
}

fn scalar(form: &Form, label: &str) -> Result<String, String> {
    match form {
        Form::String(value) | Form::Symbol(value) => Ok(value.clone()),
        _ => Err(format!(
            "extension/malformed: {label} must be a string or symbol"
        )),
    }
}

fn absolute(path: &Path) -> Result<PathBuf, String> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|error| format!("extension/root-invalid: {error}"))?
            .join(path)
    };
    if path.exists() {
        path.canonicalize()
            .map_err(|error| format!("extension/root-invalid: {error}"))
    } else {
        Ok(path)
    }
}
