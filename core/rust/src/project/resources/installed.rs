use crate::kernel::{parse, Form};
use crate::package::validate_installed_root;
use crate::package_manifest::PackageManifest;
use crate::project::{normalize_coordinate, Project};
use semver::{Version, VersionReq};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(super) struct InstalledProject {
    pub coordinate: String,
    pub version: Version,
    pub project: Project,
}

#[derive(Debug, Clone)]
struct Pending {
    coordinate: String,
    requirement: String,
    chain: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct State {
    constraints: BTreeMap<String, Vec<String>>,
    selected: BTreeMap<String, InstalledProject>,
}

pub(super) fn resolve(
    project: &Project,
    distribution_root: &Path,
) -> Result<Vec<InstalledProject>, String> {
    if project.dependencies.is_empty() {
        return Ok(Vec::new());
    }
    let root_coordinate = normalize_coordinate(&project.id)?;
    let pending = project
        .dependencies
        .iter()
        .map(|(coordinate, requirement)| Pending {
            coordinate: coordinate.clone(),
            requirement: requirement.clone(),
            chain: vec![root_coordinate.clone()],
        })
        .collect();
    let state = solve(pending, State::default(), distribution_root)?;
    ordered(project, &state.selected)
}

fn solve(
    mut pending: VecDeque<Pending>,
    mut state: State,
    distribution_root: &Path,
) -> Result<State, String> {
    let Some(requirement) = pending.pop_front() else {
        return Ok(state);
    };
    let coordinate = normalize_coordinate(&requirement.coordinate)?;
    if requirement.chain.contains(&coordinate) {
        let mut cycle = requirement.chain.clone();
        cycle.push(coordinate);
        return Err(format!(
            "installed package dependency cycle: {}",
            cycle.join(" -> ")
        ));
    }
    VersionReq::parse(&requirement.requirement).map_err(|error| {
        format!(
            "invalid installed dependency requirement {} for {coordinate}: {error}",
            requirement.requirement
        )
    })?;
    let constraints = state.constraints.entry(coordinate.clone()).or_default();
    if !constraints.contains(&requirement.requirement) {
        constraints.push(requirement.requirement.clone());
        constraints.sort();
    }
    if let Some(selected) = state.selected.get(&coordinate) {
        if matches_all(&selected.version, constraints)? {
            return solve(pending, state, distribution_root);
        }
        return Err(conflict(&coordinate, constraints, &[]));
    }

    let candidates = candidates(distribution_root, &coordinate, constraints)?;
    if candidates.is_empty() {
        return Err(conflict(
            &coordinate,
            constraints,
            &installed_versions(distribution_root, &coordinate)?,
        ));
    }
    let mut failures = Vec::new();
    for candidate in candidates {
        let mut trial = state.clone();
        trial.selected.insert(coordinate.clone(), candidate.clone());
        let mut next = pending.clone();
        let mut chain = requirement.chain.clone();
        chain.push(coordinate.clone());
        for (dependency, version) in candidate.project.dependencies.iter().rev() {
            next.push_front(Pending {
                coordinate: dependency.clone(),
                requirement: version.clone(),
                chain: chain.clone(),
            });
        }
        match solve(next, trial, distribution_root) {
            Ok(resolved) => return Ok(resolved),
            Err(error) => failures.push(format!("{}@{}: {error}", coordinate, candidate.version)),
        }
    }
    Err(failures.join("; "))
}

fn matches_all(version: &Version, constraints: &[String]) -> Result<bool, String> {
    constraints
        .iter()
        .map(|value| VersionReq::parse(value).map(|requirement| requirement.matches(version)))
        .collect::<Result<Vec<_>, _>>()
        .map(|matches| matches.into_iter().all(|value| value))
        .map_err(|error| error.to_string())
}

fn candidates(
    distribution_root: &Path,
    coordinate: &str,
    constraints: &[String],
) -> Result<Vec<InstalledProject>, String> {
    let mut output = Vec::new();
    for version in installed_versions(distribution_root, coordinate)? {
        if matches_all(&version, constraints)? {
            output.push(read_registration(distribution_root, coordinate, &version)?);
        }
    }
    Ok(output)
}

fn installed_versions(distribution_root: &Path, coordinate: &str) -> Result<Vec<Version>, String> {
    let directory = registration_directory(distribution_root, coordinate)?;
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut versions = Vec::new();
    for entry in fs::read_dir(&directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
    {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("edn") {
            continue;
        }
        let value = path
            .file_stem()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("invalid installed package registration: {}", path.display()))?;
        versions.push(Version::parse(value).map_err(|error| {
            format!(
                "invalid installed package version in {}: {error}",
                path.display()
            )
        })?);
    }
    versions.sort_by(|left, right| right.cmp(left));
    Ok(versions)
}

fn read_registration(
    distribution_root: &Path,
    coordinate: &str,
    version: &Version,
) -> Result<InstalledProject, String> {
    let path =
        registration_directory(distribution_root, coordinate)?.join(format!("{version}.edn"));
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let Form::Map(entries) = parse(&source)? else {
        return Err(format!("{} must contain an EDN map", path.display()));
    };
    let registered_coordinate = field(&entries, "coordinate")?;
    if normalize_coordinate(&registered_coordinate)? != coordinate {
        return Err(format!("{} registers the wrong coordinate", path.display()));
    }
    let registered_version = Version::parse(&field(&entries, "version")?)
        .map_err(|error| format!("{} has an invalid version: {error}", path.display()))?;
    if &registered_version != version {
        return Err(format!("{} registers the wrong version", path.display()));
    }
    let archive = field(&entries, "archive-sha256")?;
    let digest = archive
        .strip_prefix("sha256:")
        .filter(|value| value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit()))
        .ok_or_else(|| format!("{} has an invalid archive digest", path.display()))?;
    let package_root = PathBuf::from(field(&entries, "root")?)
        .canonicalize()
        .map_err(|error| format!("{} points to an unavailable root: {error}", path.display()))?;
    let trusted_roots = distribution_root
        .join("roots/sha256")
        .canonicalize()
        .map_err(|error| format!("installed package root is unavailable: {error}"))?;
    if !package_root.starts_with(&trusted_roots)
        || package_root.file_name().and_then(|value| value.to_str()) != Some(digest)
    {
        return Err(format!(
            "{} points outside the content-addressed package roots",
            path.display()
        ));
    }
    let manifest = PackageManifest::read(&package_root.join("package.edn")).map_err(|error| {
        format!(
            "{} has an invalid package manifest: {error}",
            path.display()
        )
    })?;
    let project = validate_installed_root(&package_root, &manifest).map_err(|error| {
        format!(
            "{} has an invalid installed package: {error}",
            path.display()
        )
    })?;
    if normalize_coordinate(&manifest.identity)? != coordinate || manifest.version != *version {
        return Err(format!(
            "{} disagrees with the installed package manifest",
            path.display()
        ));
    }
    Ok(InstalledProject {
        coordinate: coordinate.to_owned(),
        version: version.clone(),
        project,
    })
}

fn field(entries: &[(Form, Form)], name: &str) -> Result<String, String> {
    entries
        .iter()
        .find_map(|(key, value)| match (key, value) {
            (Form::Keyword(candidate), Form::String(value)) if candidate == name => {
                Some(value.clone())
            }
            _ => None,
        })
        .ok_or_else(|| format!("installed package registration is missing :{name}"))
}

fn registration_directory(root: &Path, coordinate: &str) -> Result<PathBuf, String> {
    let (tap, package) = coordinate
        .split_once(':')
        .ok_or_else(|| format!("invalid package coordinate: {coordinate}"))?;
    let (owner, name) = package
        .split_once('/')
        .ok_or_else(|| format!("invalid package coordinate: {coordinate}"))?;
    Ok(root.join("packages").join(tap).join(owner).join(name))
}

fn conflict(coordinate: &str, constraints: &[String], versions: &[Version]) -> String {
    format!(
        "no installed version of {coordinate} satisfies [{}]; installed: [{}]",
        constraints.join(", "),
        versions
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

    fn digest(bytes: &[u8]) -> String {
        format!("sha256:{:x}", Sha256::digest(bytes))
    }

    #[test]
    fn rejects_tampered_installed_package_before_resource_loading() {
        let archive_digest = "a".repeat(64);
        let root = std::env::temp_dir().join(format!(
            "hara-installed-manifest-{}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        let package_root = root.join("roots/sha256").join(&archive_digest);
        let source = b"(ns demo.core)\n";
        let project = br#"{:hara/type :project
 :hara/version "1.0.0"
 :project/id "hara:demo/pkg"
 :project/version "1.0.0"
 :project/source-paths ["src"]
 :project/test-paths []
 :project/extension-paths []
 :project/capabilities #{}
 :project/dependencies {}}"#;
        let manifest = format!(
            r#"{{:harp/format "0.0.0-alpha"
 :package {{:identity "hara:demo/pkg" :version "1.0.0"}}
 :files {{"project.edn" {{:sha256 "{}" :size {}}}
          "src/demo/core.hal" {{:sha256 "{}" :size {}}}}}}}"#,
            digest(project),
            project.len(),
            digest(source),
            source.len()
        );

        fs::create_dir_all(package_root.join("src/demo")).unwrap();
        fs::write(package_root.join("project.edn"), project).unwrap();
        fs::write(package_root.join("src/demo/core.hal"), source).unwrap();
        fs::write(package_root.join("package.edn"), manifest).unwrap();
        let registration = root.join("packages/hara/demo/pkg/1.0.0.edn");
        fs::create_dir_all(registration.parent().unwrap()).unwrap();
        fs::write(
            &registration,
            format!(
                "{{:coordinate \"hara:demo/pkg\" :version \"1.0.0\" :archive-sha256 \"sha256:{}\" :root {}}}\n",
                archive_digest,
                Form::String(package_root.display().to_string())
            ),
        )
        .unwrap();

        read_registration(&root, "hara:demo/pkg", &Version::parse("1.0.0").unwrap()).unwrap();
        fs::write(
            package_root.join("src/demo/core.hal"),
            b"(ns demo.tampered)\n",
        )
        .unwrap();
        let error = read_registration(&root, "hara:demo/pkg", &Version::parse("1.0.0").unwrap())
            .unwrap_err();
        assert!(
            error.contains("package/digest-mismatch") || error.contains("package/size-mismatch"),
            "{error}"
        );

        fs::remove_dir_all(root).unwrap();
    }
}

fn ordered(
    project: &Project,
    selected: &BTreeMap<String, InstalledProject>,
) -> Result<Vec<InstalledProject>, String> {
    fn visit(
        coordinate: &str,
        selected: &BTreeMap<String, InstalledProject>,
        visiting: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
        output: &mut Vec<InstalledProject>,
    ) -> Result<(), String> {
        if visited.contains(coordinate) {
            return Ok(());
        }
        if !visiting.insert(coordinate.to_owned()) {
            return Err(format!(
                "installed package dependency cycle at {coordinate}"
            ));
        }
        let package = selected
            .get(coordinate)
            .ok_or_else(|| format!("installed package {coordinate} was not resolved"))?;
        for dependency in package.project.dependencies.keys() {
            visit(dependency, selected, visiting, visited, output)?;
        }
        visiting.remove(coordinate);
        visited.insert(coordinate.to_owned());
        output.push(package.clone());
        Ok(())
    }

    let mut output = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for coordinate in project.dependencies.keys() {
        visit(
            coordinate,
            selected,
            &mut visiting,
            &mut visited,
            &mut output,
        )?;
    }
    Ok(output)
}
