use super::{declared_namespace, files_in, Project};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

mod installed;

/// Returns namespace resources from installed dependencies followed by the
/// automatically selected native Rust profile of the consuming project.
pub fn source_resources(project: &Project) -> Result<Vec<(String, String)>, String> {
    source_resources_at(project, &dist_root())
}

pub(crate) fn source_resources_at(
    project: &Project,
    distribution_root: &Path,
) -> Result<Vec<(String, String)>, String> {
    let mut resources = Vec::new();
    let mut declarations = BTreeMap::<String, (String, PathBuf)>::new();
    for dependency in installed::resolve(project, distribution_root)? {
        collect_project(
            &dependency.project,
            &format!("{}@{}", dependency.coordinate, dependency.version),
            &mut declarations,
            &mut resources,
        )?;
    }
    collect_project(
        project,
        &format!("{}@{}", project.id, project.version),
        &mut declarations,
        &mut resources,
    )?;
    Ok(resources)
}

fn collect_project(
    project: &Project,
    owner: &str,
    declarations: &mut BTreeMap<String, (String, PathBuf)>,
    resources: &mut Vec<(String, String)>,
) -> Result<(), String> {
    for path in files_in(&project.root, &project.source_paths)? {
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let namespace = declared_namespace(&source)
            .map_err(|error| format!("{}: {error}", path.display()))?
            .ok_or_else(|| format!("{} does not declare an ns or ns+ namespace", path.display()))?;
        if let Some((previous_owner, previous_path)) =
            declarations.insert(namespace.clone(), (owner.to_owned(), path.clone()))
        {
            return Err(format!(
                "duplicate namespace {namespace}: {previous_owner} ({}) and {owner} ({})",
                previous_path.display(),
                path.display()
            ));
        }
        resources.push((namespace, source));
    }
    Ok(())
}

fn dist_root() -> PathBuf {
    if let Some(root) = std::env::var_os("HARA_DIST_HOME") {
        return PathBuf::from(root);
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hara/dist")
}
