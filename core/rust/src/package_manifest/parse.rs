use super::*;
use crate::kernel::{parse, Form};
use std::path::Component;

pub(super) fn parse_manifest(source: &str) -> Result<PackageManifest, PackageManifestError> {
    let form = parse(source).map_err(|error| {
        PackageManifestError::new(
            "package/invalid-manifest",
            format!("package.edn is not valid EDN: {error}"),
        )
    })?;
    let root = expect_map(&form, "package.edn")?;
    let format = required_string(root, "harp/format", "package.edn")?;
    if format != PACKAGE_FORMAT {
        return Err(PackageManifestError::new(
            "package/invalid-manifest",
            format!("unsupported :harp/format {format}"),
        ));
    }

    let package = expect_map(required_value(root, "package", "package.edn")?, ":package")?;
    let identity = required_string(package, "identity", ":package")?;
    let version_source = required_string(package, "version", ":package")?;
    let version = Version::parse(&version_source).map_err(|error| {
        PackageManifestError::new(
            "package/invalid-manifest",
            format!("invalid :package :version {version_source}: {error}"),
        )
    })?;
    let provenance = optional_value(package, "provenance", ":package")?
        .map(parse_provenance)
        .transpose()?;
    let files = parse_files(required_value(root, "files", "package.edn")?)?;

    if let Some(descriptor) = optional_value(root, "descriptor", "package.edn")? {
        validate_descriptor(descriptor)?;
    }

    let variants = optional_value(root, "variants", "package.edn")?
        .map(|value| parse_variants(value, &files))
        .transpose()?
        .unwrap_or_default();
    if !variants.is_empty() && provenance.is_none() {
        return Err(PackageManifestError::new(
            "package/invalid-manifest",
            "packages with runtime variants require :package :provenance",
        ));
    }

    Ok(PackageManifest {
        format,
        identity,
        version,
        provenance,
        files,
        variants,
        canonical_edn: canonical_form(&form).to_string(),
    })
}

fn parse_provenance(value: &Form) -> Result<PackageProvenance, PackageManifestError> {
    let entries = expect_map(value, ":package :provenance")?;
    let repository = required_string(entries, "repository", ":package :provenance")?;
    let commit = required_string(entries, "commit", ":package :provenance")?;
    if !matches!(commit.len(), 40 | 64)
        || !commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(PackageManifestError::new(
            "package/invalid-manifest",
            ":package :provenance :commit must be a canonical 40- or 64-character hexadecimal digest",
        ));
    }
    Ok(PackageProvenance { repository, commit })
}

fn parse_files(value: &Form) -> Result<BTreeMap<PathBuf, PackageFile>, PackageManifestError> {
    let entries = expect_map(value, ":files")?;
    let mut files = BTreeMap::new();
    for (path, declaration) in entries {
        let Form::String(path) = path else {
            return Err(PackageManifestError::new(
                "package/invalid-manifest",
                ":files keys must be archive-relative strings",
            ));
        };
        let path = parse_relative_path(path)?;
        let declaration = expect_map(declaration, ":files entry")?;
        let sha256 = required_string(declaration, "sha256", ":files entry")?;
        validate_sha256(&sha256)?;
        let size = match required_value(declaration, "size", ":files entry")? {
            Form::Number(value) if *value >= 0 => *value as u64,
            _ => {
                return Err(PackageManifestError::new(
                    "package/invalid-manifest",
                    ":files entry :size must be a non-negative integer",
                ));
            }
        };
        if files
            .insert(path.clone(), PackageFile { sha256, size })
            .is_some()
        {
            return Err(PackageManifestError::new(
                "package/invalid-manifest",
                format!("duplicate :files path {}", path.display()),
            ));
        }
    }
    Ok(files)
}

fn parse_variants(
    value: &Form,
    files: &BTreeMap<PathBuf, PackageFile>,
) -> Result<BTreeMap<PackageRuntime, PackageVariant>, PackageManifestError> {
    let entries = expect_map(value, ":variants")?;
    let mut variants = BTreeMap::new();
    for (runtime, declaration) in entries {
        let runtime = parse_runtime(runtime)?;
        let variant = parse_variant(runtime, declaration, files)?;
        if variants.insert(runtime, variant).is_some() {
            return Err(PackageManifestError::new(
                "package/duplicate-variant",
                format!("duplicate :{} runtime variant", runtime.keyword()),
            ));
        }
    }
    Ok(variants)
}

fn parse_runtime(value: &Form) -> Result<PackageRuntime, PackageManifestError> {
    match value {
        Form::Keyword(value) if value == "jvm" => Ok(PackageRuntime::Jvm),
        Form::Keyword(value) if value == "wasm" => Ok(PackageRuntime::Wasm),
        _ => Err(PackageManifestError::new(
            "package/invalid-manifest",
            ":variants keys must be :jvm or :wasm",
        )),
    }
}

fn parse_variant(
    runtime: PackageRuntime,
    value: &Form,
    files: &BTreeMap<PathBuf, PackageFile>,
) -> Result<PackageVariant, PackageManifestError> {
    let entries = expect_map(value, "runtime variant")?;
    let artifact_entries = expect_map(
        required_value(entries, "variant/artifact", "runtime variant")?,
        ":variant/artifact",
    )?;
    let artifact_type =
        match required_identifier(artifact_entries, "artifact/type", ":variant/artifact")?.as_str()
        {
            "jar" => PackageArtifactType::Jar,
            "wasm" => PackageArtifactType::Wasm,
            "hta" => PackageArtifactType::Hta,
            other => {
                return Err(PackageManifestError::new(
                    "package/invalid-manifest",
                    format!("unsupported :artifact/type :{other}"),
                ));
            }
        };
    match (runtime, artifact_type) {
        (PackageRuntime::Jvm, PackageArtifactType::Jar)
        | (PackageRuntime::Wasm, PackageArtifactType::Wasm)
        | (PackageRuntime::Wasm, PackageArtifactType::Hta) => {}
        _ => {
            return Err(PackageManifestError::new(
                "package/invalid-manifest",
                format!(
                    ":{} variant cannot use :artifact/type :{}",
                    runtime.keyword(),
                    artifact_type.keyword()
                ),
            ));
        }
    }

    let artifact_path = parse_relative_path(&required_string(
        artifact_entries,
        "artifact/path",
        ":variant/artifact",
    )?)?;
    let sha256 = required_string(artifact_entries, "artifact/sha256", ":variant/artifact")?;
    validate_sha256(&sha256)?;
    let target = required_string(artifact_entries, "artifact/target", ":variant/artifact")?;
    let abi = required_string(artifact_entries, "artifact/abi", ":variant/artifact")?;
    let entry_point = required_string(
        artifact_entries,
        "artifact/entry-point",
        ":variant/artifact",
    )?;
    let file = files.get(&artifact_path).ok_or_else(|| {
        PackageManifestError::new(
            "package/missing-artifact",
            format!(
                ":artifact/path is not declared in :files: {}",
                artifact_path.display()
            ),
        )
    })?;
    if file.sha256 != sha256 {
        return Err(PackageManifestError::new(
            "package/digest-mismatch",
            format!(
                "{} declares {}, but :files declares {}",
                artifact_path.display(),
                sha256,
                file.sha256
            ),
        ));
    }

    let required_capabilities = parse_identifier_set(
        required_value(entries, "variant/required-capabilities", "runtime variant")?,
        ":variant/required-capabilities",
    )?;
    let host_calls = optional_value(entries, "variant/host-calls", "runtime variant")?
        .map(|value| parse_identifier_set(value, ":variant/host-calls"))
        .transpose()?
        .unwrap_or_default();
    let exports = optional_value(entries, "variant/exports", "runtime variant")?
        .map(|value| parse_identifier_set(value, ":variant/exports"))
        .transpose()?
        .unwrap_or_default();
    let dependencies = optional_value(entries, "variant/dependencies", "runtime variant")?
        .map(|value| {
            expect_map(value, ":variant/dependencies")?;
            Ok::<Form, PackageManifestError>(value.clone())
        })
        .transpose()?;
    let lifecycle = optional_value(entries, "variant/lifecycle", "runtime variant")?
        .map(parse_lifecycle)
        .transpose()?;

    Ok(PackageVariant {
        runtime,
        artifact: PackageArtifact {
            artifact_type,
            path: artifact_path,
            sha256,
            target,
            abi,
            entry_point,
        },
        required_capabilities,
        host_calls,
        exports,
        dependencies,
        lifecycle,
    })
}

fn parse_lifecycle(value: &Form) -> Result<PackageLifecycle, PackageManifestError> {
    let entries = expect_map(value, ":variant/lifecycle")?;
    let load = required_identifier(entries, "lifecycle/load", ":variant/lifecycle")?;
    let close = required_identifier(entries, "lifecycle/close", ":variant/lifecycle")?;
    if load != "idempotent" || close != "idempotent" {
        return Err(PackageManifestError::new(
            "package/invalid-manifest",
            ":variant/lifecycle load and close must be :idempotent",
        ));
    }
    let session_isolation =
        required_bool(entries, "lifecycle/session-isolation", ":variant/lifecycle")?;
    if !session_isolation {
        return Err(PackageManifestError::new(
            "package/invalid-manifest",
            ":variant/lifecycle requires :lifecycle/session-isolation true",
        ));
    }
    let asynchronous = optional_value(entries, "lifecycle/async", ":variant/lifecycle")?
        .map(|value| parse_bool(value, ":lifecycle/async"))
        .transpose()?
        .unwrap_or(false);
    let cancellation = optional_value(entries, "lifecycle/cancellation", ":variant/lifecycle")?
        .map(|value| parse_bool(value, ":lifecycle/cancellation"))
        .transpose()?
        .unwrap_or(false);
    Ok(PackageLifecycle {
        load_idempotent: true,
        close_idempotent: true,
        session_isolation,
        asynchronous,
        cancellation,
    })
}

fn parse_identifier_set(
    value: &Form,
    context: &str,
) -> Result<BTreeSet<String>, PackageManifestError> {
    let Form::Set(values) = value else {
        return Err(PackageManifestError::new(
            "package/invalid-manifest",
            format!("{context} must be an EDN set"),
        ));
    };
    values
        .iter()
        .map(|value| identifier(value, context))
        .collect()
}

fn identifier(value: &Form, context: &str) -> Result<String, PackageManifestError> {
    match value {
        Form::Keyword(value) | Form::Symbol(value) | Form::String(value) if !value.is_empty() => {
            Ok(value.clone())
        }
        _ => Err(PackageManifestError::new(
            "package/invalid-manifest",
            format!("{context} must contain non-empty keywords, symbols, or strings"),
        )),
    }
}

fn required_identifier(
    entries: &[(Form, Form)],
    key: &str,
    context: &str,
) -> Result<String, PackageManifestError> {
    identifier(required_value(entries, key, context)?, &format!(":{key}"))
}

fn required_string(
    entries: &[(Form, Form)],
    key: &str,
    context: &str,
) -> Result<String, PackageManifestError> {
    match required_value(entries, key, context)? {
        Form::String(value) if !value.is_empty() => Ok(value.clone()),
        _ => Err(PackageManifestError::new(
            "package/invalid-manifest",
            format!("{context} :{key} must be a non-empty string"),
        )),
    }
}

fn required_bool(
    entries: &[(Form, Form)],
    key: &str,
    context: &str,
) -> Result<bool, PackageManifestError> {
    parse_bool(
        required_value(entries, key, context)?,
        &format!("{context} :{key}"),
    )
}

fn parse_bool(value: &Form, context: &str) -> Result<bool, PackageManifestError> {
    match value {
        Form::Bool(value) => Ok(*value),
        _ => Err(PackageManifestError::new(
            "package/invalid-manifest",
            format!("{context} must be a boolean"),
        )),
    }
}

fn expect_map<'a>(
    value: &'a Form,
    context: &str,
) -> Result<&'a [(Form, Form)], PackageManifestError> {
    match value {
        Form::Map(entries) => Ok(entries),
        _ => Err(PackageManifestError::new(
            "package/invalid-manifest",
            format!("{context} must be an EDN map"),
        )),
    }
}

fn required_value<'a>(
    entries: &'a [(Form, Form)],
    key: &str,
    context: &str,
) -> Result<&'a Form, PackageManifestError> {
    optional_value(entries, key, context)?.ok_or_else(|| {
        PackageManifestError::new(
            "package/invalid-manifest",
            format!("{context} is missing :{key}"),
        )
    })
}

fn optional_value<'a>(
    entries: &'a [(Form, Form)],
    key: &str,
    context: &str,
) -> Result<Option<&'a Form>, PackageManifestError> {
    let mut values = entries.iter().filter_map(|(candidate, value)| {
        matches!(candidate, Form::Keyword(name) if name == key).then_some(value)
    });
    let value = values.next();
    if values.next().is_some() {
        return Err(PackageManifestError::new(
            "package/invalid-manifest",
            format!("{context} contains duplicate :{key}"),
        ));
    }
    Ok(value)
}

fn parse_relative_path(value: &str) -> Result<PathBuf, PackageManifestError> {
    let path = PathBuf::from(value);
    if value.is_empty()
        || value.contains('\\')
        || value.split('/').any(str::is_empty)
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir
                    | Component::RootDir
                    | Component::Prefix(_)
                    | Component::CurDir
            )
        })
    {
        return Err(PackageManifestError::new(
            "package/invalid-manifest",
            format!("unsafe or noncanonical package path: {value}"),
        ));
    }
    Ok(path)
}

fn validate_sha256(value: &str) -> Result<(), PackageManifestError> {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return Err(PackageManifestError::new(
            "package/invalid-manifest",
            format!("digest must use sha256: prefix: {value}"),
        ));
    };
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(PackageManifestError::new(
            "package/invalid-manifest",
            format!("digest must contain 64 lowercase hexadecimal characters: {value}"),
        ));
    }
    Ok(())
}

fn validate_descriptor(value: &Form) -> Result<(), PackageManifestError> {
    match value {
        Form::Map(entries) => {
            for (key, value) in entries {
                if let Some(key) = descriptor_key(key) {
                    let tail = key.rsplit('/').next().unwrap_or(&key);
                    if matches!(
                        tail,
                        "credential"
                            | "credentials"
                            | "classloader"
                            | "socket"
                            | "raw-socket"
                            | "native-handle"
                            | "native-pointer"
                    ) {
                        return Err(PackageManifestError::new(
                            "package/invalid-descriptor",
                            format!("runtime-neutral descriptor cannot contain :{key}"),
                        ));
                    }
                }
                validate_descriptor(value)?;
            }
        }
        Form::Set(values) | Form::Vector(values) | Form::List(values) => {
            for value in values {
                validate_descriptor(value)?;
            }
        }
        Form::Tagged(_, value) => validate_descriptor(value)?,
        Form::Metadata(metadata, value) => {
            validate_descriptor(metadata)?;
            validate_descriptor(value)?;
        }
        _ => {}
    }
    Ok(())
}

fn descriptor_key(value: &Form) -> Option<String> {
    match value {
        Form::Keyword(value) | Form::Symbol(value) | Form::String(value) => {
            Some(value.to_ascii_lowercase())
        }
        _ => None,
    }
}

fn canonical_form(value: &Form) -> Form {
    match value {
        Form::Nil => Form::Nil,
        Form::Bool(value) => Form::Bool(*value),
        Form::Number(value) => Form::Number(*value),
        Form::Float(value) => Form::Float(*value),
        Form::BigInteger(value) => Form::BigInteger(value.clone()),
        Form::Decimal(value) => Form::Decimal(value.clone()),
        Form::Character(value) => Form::Character(*value),
        Form::Regex(value) => Form::Regex(value.clone()),
        Form::Tagged(tag, value) => Form::Tagged(tag.clone(), Box::new(canonical_form(value))),
        Form::Metadata(metadata, value) => Form::Metadata(
            Box::new(canonical_form(metadata)),
            Box::new(canonical_form(value)),
        ),
        Form::Symbol(value) => Form::Symbol(value.clone()),
        Form::Keyword(value) => Form::Keyword(value.clone()),
        Form::String(value) => Form::String(value.clone()),
        Form::Map(entries) => {
            let mut entries = entries
                .iter()
                .map(|(key, value)| (canonical_form(key), canonical_form(value)))
                .collect::<Vec<_>>();
            entries.sort_by(|left, right| {
                left.0
                    .to_string()
                    .cmp(&right.0.to_string())
                    .then_with(|| left.1.to_string().cmp(&right.1.to_string()))
            });
            Form::Map(entries)
        }
        Form::Set(values) => {
            let mut values = values.iter().map(canonical_form).collect::<Vec<_>>();
            values.sort_by_key(ToString::to_string);
            Form::Set(values)
        }
        Form::Vector(values) => Form::Vector(values.iter().map(canonical_form).collect()),
        Form::List(values) => Form::List(values.iter().map(canonical_form).collect()),
    }
}
