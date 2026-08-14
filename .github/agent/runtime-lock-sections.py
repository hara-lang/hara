from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected one match in {path}, found {count}: {old[:120]!r}")
    file.write_text(text.replace(old, new, 1))


project = "core/rust/src/project.rs"
replace_once(
    project,
    "use semver::{Version, VersionReq};\nuse std::collections::BTreeMap;\nuse std::fs;\n",
    "use semver::{Version, VersionReq};\nuse sha2::{Digest, Sha256};\nuse std::collections::BTreeMap;\nuse std::fmt::Write as _;\nuse std::fs;\n",
)

old_sync = '''/// Creates or validates the lockfile for graphs that need no remote packages.
/// Remote graphs deliberately stop here until the reviewed registry and
/// identity clients can provide the required signed release metadata.
pub fn sync_lock(project: &Project, mode: LockMode) -> Result<PathBuf, String> {
    let lock = project.root.join("project.lock.edn");
    if !project.dependencies.is_empty() {
        return Err(format!(
            "project sync requires the reviewed registry client to resolve {} declared dependencies",
            project.dependencies.len()
        ));
    }
    match mode {
        LockMode::Locked | LockMode::Frozen if !lock.is_file() => {
            return Err(format!(
                "{} requires an existing project.lock.edn",
                mode.flag()
            ));
        }
        LockMode::Locked | LockMode::Frozen => validate_empty_lock(&lock)?,
        LockMode::Default | LockMode::Offline => {
            fs::write(&lock, "{:lock/format \\"0.0.0-alpha\\" :packages {}}\\n")
                .map_err(|error| format!("cannot write {}: {error}", lock.display()))?;
        }
    }
    Ok(lock)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    Default,
    Offline,
    Locked,
    Frozen,
}

impl LockMode {
    pub fn flag(self) -> &'static str {
        match self {
            Self::Default => "sync",
            Self::Offline => "--offline",
            Self::Locked => "--locked",
            Self::Frozen => "--frozen",
        }
    }
}
'''
new_sync = '''const LOCK_FORMAT: &str = "0.0.0-alpha";

/// Returns the stable digest used to bind one lock section to the normalized
/// active runtime declaration. Host-specific absolute paths are never hashed.
pub fn runtime_declaration_digest(project: &Project) -> String {
    let mut declaration = String::new();
    digest_scalar(&mut declaration, "runtime", &project.active_runtime);
    digest_paths(&mut declaration, "source-paths", &project.source_paths);
    digest_paths(&mut declaration, "test-paths", &project.test_paths);
    digest_paths(
        &mut declaration,
        "extension-paths",
        &project.extension_paths,
    );
    digest_paths(
        &mut declaration,
        "native-source-paths",
        &project.native_source_paths,
    );
    digest_scalar(
        &mut declaration,
        "target-path",
        &project
            .runtime_target_path
            .as_ref()
            .map(portable_path)
            .unwrap_or_default(),
    );
    digest_values(&mut declaration, "capabilities", &project.capabilities);
    digest_dependencies(
        &mut declaration,
        "hara-dependencies",
        &project.dependencies,
    );
    digest_dependencies(
        &mut declaration,
        "maven-dependencies",
        &project.maven_dependencies,
    );
    for (namespace, extension) in &project.extensions {
        digest_scalar(
            &mut declaration,
            "extension-namespace",
            namespace,
        );
        digest_scalar(
            &mut declaration,
            "extension-declaration",
            &extension.to_string(),
        );
    }

    let digest = Sha256::digest(declaration.as_bytes());
    let mut output = String::with_capacity(71);
    output.push_str("sha256:");
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

/// Reconciles only the automatically active runtime section. Existing
/// inactive sections are retained as parsed lock data and the result is
/// replaced through a temporary file.
pub fn sync_lock(project: &Project, mode: LockMode) -> Result<PathBuf, String> {
    let lock = project.root.join("project.lock.edn");
    let unresolved = project.dependencies.len() + project.maven_dependencies.len();
    if unresolved != 0 {
        return Err(format!(
            "project sync requires reviewed resolvers for {unresolved} active runtime dependencies"
        ));
    }
    match mode {
        LockMode::Locked | LockMode::Frozen if !lock.is_file() => {
            return Err(format!(
                "{} requires an existing project.lock.edn",
                mode.flag()
            ));
        }
        LockMode::Locked | LockMode::Frozen => validate_runtime_lock(project, &lock)?,
        LockMode::Default | LockMode::Offline => {
            let mut form = if lock.is_file() {
                read_lock(&lock)?
            } else {
                empty_lock()
            };
            reconcile_runtime_section(project, &mut form)?;
            write_lock_atomic(&lock, &form)?;
        }
    }
    Ok(lock)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    Default,
    Offline,
    Locked,
    Frozen,
}

impl LockMode {
    pub fn flag(self) -> &'static str {
        match self {
            Self::Default => "sync",
            Self::Offline => "--offline",
            Self::Locked => "--locked",
            Self::Frozen => "--frozen",
        }
    }
}
'''
replace_once(project, old_sync, new_sync)

old_validate = '''fn validate_empty_lock(path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let form = parse(&source).map_err(|error| format!("{}: {error}", path.display()))?;
    let entries = map(&form, "project.lock.edn must be an EDN map")?;
    if matches!(lookup(entries, "lock/format"), Some(Form::String(version)) if version == "0.0.0-alpha")
        && matches!(lookup(entries, "packages"), Some(Form::Map(entries)) if entries.is_empty())
    {
        Ok(())
    } else {
        Err(format!(
            "{} is not a lockfile written by this CLI",
            path.display()
        ))
    }
}
'''
new_validate = '''fn digest_scalar(output: &mut String, label: &str, value: &str) {
    write!(output, "{label}={}:{}\\n", value.len(), value)
        .expect("writing to a String cannot fail");
}

fn digest_paths(output: &mut String, label: &str, values: &[PathBuf]) {
    for value in values {
        digest_scalar(output, label, &portable_path(value));
    }
}

fn digest_values(output: &mut String, label: &str, values: &[String]) {
    for value in values {
        digest_scalar(output, label, value);
    }
}

fn digest_dependencies(
    output: &mut String,
    label: &str,
    values: &BTreeMap<String, String>,
) {
    for (coordinate, requirement) in values {
        digest_scalar(output, &format!("{label}-coordinate"), coordinate);
        digest_scalar(output, &format!("{label}-requirement"), requirement);
    }
}

fn portable_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\\\', "/")
}

fn empty_lock() -> Form {
    Form::Map(vec![
        (
            Form::Keyword("lock/format".into()),
            Form::String(LOCK_FORMAT.into()),
        ),
        (
            Form::Keyword("runtime-sections".into()),
            Form::Map(Vec::new()),
        ),
        (Form::Keyword("packages".into()), Form::Map(Vec::new())),
    ])
}

fn active_runtime_section(project: &Project) -> Form {
    Form::Map(vec![
        (
            Form::Keyword("runtime".into()),
            Form::Keyword(project.active_runtime.clone()),
        ),
        (
            Form::Keyword("declaration-digest".into()),
            Form::String(runtime_declaration_digest(project)),
        ),
        (Form::Keyword("packages".into()), Form::Map(Vec::new())),
        (Form::Keyword("maven".into()), Form::Map(Vec::new())),
    ])
}

fn read_lock(path: &Path) -> Result<Form, String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let form = parse(&source).map_err(|error| format!("{}: {error}", path.display()))?;
    validate_lock_root(path, &form)?;
    Ok(form)
}

fn validate_lock_root(path: &Path, form: &Form) -> Result<(), String> {
    let entries = map(form, "project.lock.edn must be an EDN map")?;
    if !matches!(lookup(entries, "lock/format"), Some(Form::String(version)) if version == LOCK_FORMAT)
    {
        return Err(format!(
            "{} is not a lockfile written by this CLI",
            path.display()
        ));
    }
    if !matches!(lookup(entries, "packages"), Some(Form::Map(_))) {
        return Err(format!(
            "{} is incomplete: :packages must be a map",
            path.display()
        ));
    }
    if let Some(sections) = lookup(entries, "runtime-sections") {
        if !matches!(sections, Form::Map(_)) {
            return Err(format!(
                "{} is incomplete: :runtime-sections must be a map",
                path.display()
            ));
        }
    }
    Ok(())
}

fn reconcile_runtime_section(project: &Project, form: &mut Form) -> Result<(), String> {
    let entries = map_mut(form, "project.lock.edn must be an EDN map")?;
    let sections = ensure_map_entry(
        entries,
        "runtime-sections",
        "project.lock.edn :runtime-sections must be a map",
    )?;
    let section = active_runtime_section(project);
    if let Some((_, value)) = sections.iter_mut().find(|(key, _)| {
        key_name(key).as_deref() == Some(project.active_runtime.as_str())
    }) {
        *value = section;
    } else {
        sections.push((
            Form::Keyword(project.active_runtime.clone()),
            section,
        ));
    }
    ensure_map_entry(
        entries,
        "packages",
        "project.lock.edn :packages must be a map",
    )?;
    Ok(())
}

fn ensure_map_entry<'a>(
    entries: &'a mut Vec<(Form, Form)>,
    key: &str,
    message: &str,
) -> Result<&'a mut Vec<(Form, Form)>, String> {
    let index = match entries
        .iter()
        .position(|(candidate, _)| key_name(candidate).as_deref() == Some(key))
    {
        Some(index) => index,
        None => {
            entries.push((Form::Keyword(key.into()), Form::Map(Vec::new())));
            entries.len() - 1
        }
    };
    match &mut entries[index].1 {
        Form::Map(values) => Ok(values),
        _ => Err(message.into()),
    }
}

fn validate_runtime_lock(project: &Project, path: &Path) -> Result<(), String> {
    let form = read_lock(path)?;
    let root = map(&form, "project.lock.edn must be an EDN map")?;
    let sections = match lookup(root, "runtime-sections") {
        Some(Form::Map(values)) => values,
        _ => {
            return Err(format!(
                "active :{} lock section is absent",
                project.active_runtime
            ))
        }
    };
    let section = sections
        .iter()
        .find(|(key, _)| key_name(key).as_deref() == Some(project.active_runtime.as_str()))
        .map(|(_, value)| value)
        .ok_or_else(|| {
            format!(
                "active :{} lock section is absent",
                project.active_runtime
            )
        })?;
    let entries = map(
        section,
        &format!(
            "active :{} lock section is incomplete",
            project.active_runtime
        ),
    )?;
    if !matches!(lookup(entries, "runtime"), Some(Form::Keyword(runtime)) if runtime == &project.active_runtime)
    {
        return Err(format!(
            "active :{} lock section is incomplete: :runtime differs",
            project.active_runtime
        ));
    }
    let expected = runtime_declaration_digest(project);
    match lookup(entries, "declaration-digest") {
        Some(Form::String(actual)) if actual == &expected => {}
        Some(Form::String(_)) => {
            return Err(format!(
                "active :{} lock section is stale: declaration digest differs",
                project.active_runtime
            ))
        }
        _ => {
            return Err(format!(
                "active :{} lock section is incomplete: missing :declaration-digest",
                project.active_runtime
            ))
        }
    }
    if !matches!(lookup(entries, "packages"), Some(Form::Map(_))) {
        return Err(format!(
            "active :{} lock section is incomplete: :packages must be a map",
            project.active_runtime
        ));
    }
    if !matches!(lookup(entries, "maven"), Some(Form::Map(_))) {
        return Err(format!(
            "active :{} lock section is incomplete: :maven must be a map",
            project.active_runtime
        ));
    }
    Ok(())
}

fn write_lock_atomic(path: &Path, form: &Form) -> Result<(), String> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("project.lock.edn");
    let temporary = path.with_file_name(format!(".{name}.{}.tmp", std::process::id()));
    fs::write(&temporary, format!("{form}\\n"))
        .map_err(|error| format!("cannot write {}: {error}", temporary.display()))?;
    match fs::rename(&temporary, path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            fs::remove_file(path)
                .map_err(|remove| format!("cannot replace {}: {remove}", path.display()))?;
            fs::rename(&temporary, path)
                .map_err(|rename| format!("cannot replace {}: {rename}", path.display()))
        }
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            Err(format!("cannot replace {}: {error}", path.display()))
        }
    }
}
'''
replace_once(project, old_validate, new_validate)

# Replace the pre-runtime empty lock assertion with the runtime-keyed contract.
tests = "core/rust/src/project/tests.rs"
old_test = '''#[test]
fn creates_and_validates_an_empty_lock() {
    let root = temp("lock");
    let project = new_app(&root, "lock-app").unwrap();
    let lock = sync_lock(&project, LockMode::Default).unwrap();
    assert_eq!(
        fs::read_to_string(&lock).unwrap(),
        "{:lock/format \\"0.0.0-alpha\\" :packages {}}\\n"
    );
    sync_lock(&project, LockMode::Frozen).unwrap();
    fs::remove_dir_all(root).unwrap();
}
'''
new_test = '''#[test]
fn creates_and_validates_an_empty_runtime_lock_section() {
    let root = temp("lock");
    let project = new_app(&root, "lock-app").unwrap();
    let lock = sync_lock(&project, LockMode::Default).unwrap();
    let source = fs::read_to_string(&lock).unwrap();
    assert!(source.contains(":runtime-sections"));
    assert!(source.contains(":rust {:runtime :rust"));
    assert!(source.contains(&runtime_declaration_digest(&project)));
    assert!(source.contains(":packages {}"));
    assert!(source.contains(":maven {}"));
    sync_lock(&project, LockMode::Frozen).unwrap();
    fs::remove_dir_all(root).unwrap();
}
'''
replace_once(tests, old_test, new_test)

extra_tests = r'''

#[test]
fn locked_modes_reject_absent_incomplete_and_stale_runtime_sections() {
    let root = temp("runtime-lock-validation");
    let project = new_app(&root, "runtime-lock-app").unwrap();
    let lock = root.join("project.lock.edn");

    fs::write(
        &lock,
        "{:lock/format \"0.0.0-alpha\" :runtime-sections {} :packages {}}\n",
    )
    .unwrap();
    assert!(sync_lock(&project, LockMode::Locked)
        .unwrap_err()
        .contains("active :rust lock section is absent"));

    fs::write(
        &lock,
        format!(
            "{{:lock/format \"0.0.0-alpha\" :runtime-sections {{:rust {{:runtime :rust :declaration-digest \"{}\" :packages {{}}}}}} :packages {{}}}}\n",
            runtime_declaration_digest(&project)
        ),
    )
    .unwrap();
    assert!(sync_lock(&project, LockMode::Frozen)
        .unwrap_err()
        .contains("incomplete: :maven must be a map"));

    sync_lock(&project, LockMode::Default).unwrap();
    let manifest = root.join("project.edn");
    let changed = fs::read_to_string(&manifest)
        .unwrap()
        .replace(":project/source-paths [\"src\"]", ":project/source-paths [\"src-next\"]");
    fs::write(&manifest, changed).unwrap();
    let changed_project = read(&root).unwrap();
    assert!(sync_lock(&changed_project, LockMode::Locked)
        .unwrap_err()
        .contains("stale: declaration digest differs"));
    sync_lock(&changed_project, LockMode::Default).unwrap();
    sync_lock(&changed_project, LockMode::Frozen).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn active_runtime_sync_preserves_inactive_sections() {
    let root = temp("runtime-lock-preserve");
    let project = new_app(&root, "runtime-lock-preserve-app").unwrap();
    let lock = root.join("project.lock.edn");
    let inactive = ":jvm {:runtime :jvm :declaration-digest \"sha256:keep-jvm\" :packages {\"org.example/library\" {:version \"1.0.0\"}} :maven {\"org.example/library\" {:version \"1.0.0\"}}}";
    fs::write(
        &lock,
        format!(
            "{{:lock/format \"0.0.0-alpha\" :runtime-sections {{{inactive} :rust {{:runtime :rust :declaration-digest \"sha256:replace-rust\" :packages {{}} :maven {{}}}}}} :packages {{}}}}\n"
        ),
    )
    .unwrap();

    sync_lock(&project, LockMode::Offline).unwrap();
    let source = fs::read_to_string(&lock).unwrap();
    assert!(source.contains(inactive));
    assert!(source.contains(&runtime_declaration_digest(&project)));
    assert!(!source.contains("sha256:replace-rust"));
    assert!(!root
        .read_dir()
        .unwrap()
        .any(|entry| entry.unwrap().file_name().to_string_lossy().ends_with(".tmp")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn runtime_declaration_digest_is_stable_and_profile_sensitive() {
    let root = temp("runtime-lock-digest");
    fs::create_dir_all(&root).unwrap();
    let manifest = root.join("project.edn");
    fs::write(
        &manifest,
        "{:hara/type :project :hara/version \"1.0.0\" :project/id demo/app :project/version \"1.0.0\" :project/source-paths [\"src\"] :project/test-paths [] :project/extension-paths [] :project/capabilities #{} :project/runtime-profiles {:rust {:runtime/source-paths [\"src-rust\"]}}}",
    )
    .unwrap();
    let first = read(&root).unwrap();
    assert_eq!(
        runtime_declaration_digest(&first),
        runtime_declaration_digest(&read(&root).unwrap())
    );
    fs::write(
        &manifest,
        "{:hara/type :project :hara/version \"1.0.0\" :project/id demo/app :project/version \"1.0.0\" :project/source-paths [\"src\"] :project/test-paths [] :project/extension-paths [] :project/capabilities #{} :project/runtime-profiles {:rust {:runtime/source-paths [\"src-rust-next\"]}}}",
    )
    .unwrap();
    assert_ne!(
        runtime_declaration_digest(&first),
        runtime_declaration_digest(&read(&root).unwrap())
    );
    fs::remove_dir_all(root).unwrap();
}
'''
Path(tests).write_text(Path(tests).read_text() + extra_tests)
