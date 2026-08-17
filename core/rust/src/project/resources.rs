use super::{declared_namespace, files_in, Project};
use std::collections::BTreeMap;
use std::fs;

/// Returns namespace resources from the automatically selected native Rust profile.
pub fn source_resources(project: &Project) -> Result<Vec<(String, String)>, String> {
    let mut resources = Vec::new();
    let mut declarations = BTreeMap::new();
    for path in files_in(&project.root, &project.source_paths)? {
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let namespace = declared_namespace(&source)
            .map_err(|error| format!("{}: {error}", path.display()))?
            .ok_or_else(|| format!("{} does not declare an ns or ns+ namespace", path.display()))?;
        if let Some(previous) = declarations.insert(namespace.clone(), path.clone()) {
            return Err(format!(
                "duplicate namespace {namespace} in effective :rust profile: {} and {}",
                previous.display(),
                path.display()
            ));
        }
        resources.push((namespace, source));
    }
    Ok(resources)
}
