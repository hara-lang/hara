use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "hara-project-{name}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn scaffolds_discovers_and_edits_dependencies() {
    let root = temp("app");
    let project = new_app(&root, "hello-app").unwrap();
    assert_eq!(
        discover(&root.join("src/hello_app")).unwrap().id,
        "hello-app"
    );
    set_dependency(&project, "hara:hara/graph", Some("^1.2.0")).unwrap();
    assert_eq!(
        read(&root).unwrap().dependencies["hara:hara/graph"],
        "^1.2.0"
    );
    set_dependency(&project, "hara:hara/graph", None).unwrap();
    assert!(read(&root).unwrap().dependencies.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_escaping_source_paths() {
    let root = temp("unsafe");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("project.edn"), "{:hara/type :project :hara/version \"1\" :project/id x :project/version \"1.0.0\" :project/source-paths [\"../src\"] :project/test-paths [] :project/extension-paths [] :project/capabilities #{}}").unwrap();
    assert!(read(&root).unwrap_err().contains("cannot escape"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
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

#[test]
fn registers_project_sources_for_cross_file_requires() {
    let root = temp("resources");
    fs::create_dir_all(root.join("src/demo")).unwrap();
    fs::write(root.join("project.edn"), "{:hara/type :project :hara/version \"1.0.0\" :project/id demo/app :project/version \"1.0.0\" :project/source-paths [\"src\"] :project/test-paths [] :project/extension-paths [] :project/capabilities #{}}").unwrap();
    fs::write(
        root.join("src/demo/helper.hal"),
        "(ns demo.helper) (defn answer [] 42)",
    )
    .unwrap();
    let project = read(&root).unwrap();
    let mut runtime = Runtime::new();
    register_sources(&project, &mut runtime).unwrap();
    assert_eq!(
        runtime
            .eval_native("(ns demo.main (:require [demo.helper :as helper])) (helper/answer)")
            .unwrap(),
        "42"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_discovery_ignores_editor_artifacts() {
    let root = temp("editor-artifacts");
    fs::create_dir_all(root.join("src/demo")).unwrap();
    fs::write(root.join("src/demo/core.hal"), "(ns demo.core)").unwrap();
    fs::write(root.join("src/demo/.#core.hal"), "unreadable editor lock").unwrap();
    fs::write(root.join("src/demo/#core.hal#"), "invalid editor backup").unwrap();
    assert_eq!(
        files_in(&root, &[PathBuf::from("src")]).unwrap(),
        vec![root.join("src/demo/core.hal")]
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn resolves_language_profiles_with_main_and_options_inheritance() {
    let root = temp("profiles");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("project.edn"), "{:hara/type :project :hara/version \"1.0.0\" :project/id demo/app :project/version \"1.0.0\" :project/source-paths [] :project/test-paths [] :project/extension-paths [] :project/capabilities #{} :project/main demo.core/app :project/default-profile :web :project/profiles {:web {:profile/language :hoplite :profile/options {:port 8080}} :admin {:profile/language :hoplite :profile/main demo.admin/app}}}").unwrap();
    let project = read(&root).unwrap();
    let web = project.resolve_profile(None).unwrap().unwrap();
    assert_eq!(
        (web.name.as_str(), web.language.as_str(), web.main.as_str()),
        ("web", "hoplite", "demo.core/app")
    );
    assert_eq!(web.options.to_string(), "{:port 8080}");
    let admin = project.resolve_profile(Some("admin")).unwrap().unwrap();
    assert_eq!(admin.main, "demo.admin/app");
    assert_eq!(admin.options, Form::Map(Vec::new()));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_missing_profile_language_and_unknown_default() {
    let root = temp("invalid-profiles");
    fs::create_dir_all(&root).unwrap();
    let prefix = "{:hara/type :project :hara/version \"1.0.0\" :project/id demo/app :project/version \"1.0.0\" :project/source-paths [] :project/test-paths [] :project/extension-paths [] :project/capabilities #{} ";
    fs::write(
        root.join("project.edn"),
        format!("{prefix}:project/profiles {{:web {{}}}}}}"),
    )
    .unwrap();
    assert!(read(&root).unwrap_err().contains(":profile/language"));
    fs::write(root.join("project.edn"), format!("{prefix}:project/default-profile :missing :project/profiles {{:web {{:profile/language :hoplite}}}}}}")).unwrap();
    assert!(read(&root).unwrap_err().contains("is not declared"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn expands_project_aliases_and_rejects_cycles() {
    let root = temp("aliases");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("project.edn"), "{:hara/type :project :hara/version \"1.0.0\" :project/id demo/app :project/version \"1.0.0\" :project/source-paths [] :project/test-paths [] :project/extension-paths [] :project/capabilities #{} :project/aliases {:check-code [\"manage\" \"analyse\"] :all [\"check-code\" \":all\"]}}").unwrap();
    let project = read(&root).unwrap();
    assert_eq!(
        expand_aliases(&project, &["all".into(), "xt.lang".into()]).unwrap(),
        vec!["manage", "analyse", ":all", "xt.lang"]
    );
    fs::write(root.join("project.edn"), "{:hara/type :project :hara/version \"1.0.0\" :project/id demo/app :project/version \"1.0.0\" :project/source-paths [] :project/test-paths [] :project/extension-paths [] :project/capabilities #{} :project/aliases {:a [\"b\"] :b [\"a\"]}}").unwrap();
    assert!(expand_aliases(&read(&root).unwrap(), &["a".into()])
        .unwrap_err()
        .contains("cycle"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn selects_rust_runtime_profile_and_can_resolve_jvm_overlay() {
    let root = temp("runtime-profiles");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("project.edn"),
        "{:hara/type :project :hara/version \"1.0.0\" :project/id demo/app :project/version \"1.0.0\" :project/source-paths [\"src\"] :project/test-paths [\"test\"] :project/extension-paths [\"extensions\"] :project/capabilities #{} :project/dependencies {\"hara:hara/base\" {:version \"^1.0.0\"}} :project/profiles {:production {:profile/language :hara}} :project/runtime-profiles {:rust {:runtime/source-paths [\"src-rust\"] :runtime/test-paths [\"test-rust\"] :runtime/extension-paths [\"extensions-rust\"] :runtime/dependencies {:hara {\"hara:hara/crypto\" {:version \"^1.0.0\"}}}} :jvm {:runtime/source-paths [\"src-jvm\"] :runtime/native-source-paths [\"src-java\"] :runtime/target-path \"target/jvm/classes\" :runtime/dependencies {:maven {org.postgresql/postgresql {:version \"42.7.7\"}}}}}}",
    )
    .unwrap();
    let project = read(&root).unwrap();
    assert_eq!(project.active_runtime, "rust");
    assert_eq!(
        project.source_paths,
        vec![PathBuf::from("src"), PathBuf::from("src-rust")]
    );
    assert_eq!(
        project.test_paths,
        vec![PathBuf::from("test"), PathBuf::from("test-rust")]
    );
    assert_eq!(
        project.extension_paths,
        vec![
            PathBuf::from("extensions"),
            PathBuf::from("extensions-rust")
        ]
    );
    assert_eq!(project.dependencies["hara:hara/base"], "^1.0.0");
    assert_eq!(project.dependencies["hara:hara/crypto"], "^1.0.0");
    assert!(project.profiles.contains_key("production"));

    let jvm = project.resolve_runtime_profile("jvm").unwrap();
    assert_eq!(
        jvm.source_paths,
        vec![PathBuf::from("src"), PathBuf::from("src-jvm")]
    );
    assert_eq!(jvm.native_source_paths, vec![PathBuf::from("src-java")]);
    assert_eq!(jvm.target_path, Some(PathBuf::from("target/jvm/classes")));
    assert_eq!(
        jvm.maven_dependencies["org.postgresql/postgresql"],
        "42.7.7"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_legacy_jvm_project_keys_and_conflicting_runtime_dependencies() {
    let root = temp("runtime-invalid");
    fs::create_dir_all(&root).unwrap();
    let prefix = "{:hara/type :project :hara/version \"1.0.0\" :project/id demo/app :project/version \"1.0.0\" :project/source-paths [] :project/test-paths [] :project/extension-paths [] :project/capabilities #{} ";
    fs::write(
        root.join("project.edn"),
        format!("{prefix}:jvm/source-paths [\"src-java\"]}}"),
    )
    .unwrap();
    let legacy = read(&root).unwrap_err();
    assert!(legacy.contains(":project/runtime-profiles :jvm :runtime/native-source-paths"));

    fs::write(
        root.join("project.edn"),
        format!("{prefix}:project/dependencies {{\"hara:hara/crypto\" {{:version \"^1.0.0\"}}}} :project/runtime-profiles {{:rust {{:runtime/dependencies {{:hara {{\"hara:hara/crypto\" {{:version \"^2.0.0\"}}}}}}}}}}}}"),
    )
    .unwrap();
    let conflict = read(&root).unwrap_err();
    assert!(conflict.contains("conflicting Hara dependency requirements"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn runtime_namespace_alternatives_are_isolated_but_effective_duplicates_fail() {
    let root = temp("runtime-namespaces");
    fs::create_dir_all(root.join("src-rust/demo")).unwrap();
    fs::create_dir_all(root.join("src-jvm/demo")).unwrap();
    fs::create_dir_all(root.join("src/demo")).unwrap();
    fs::write(
        root.join("project.edn"),
        "{:hara/type :project :hara/version \"1.0.0\" :project/id demo/app :project/version \"1.0.0\" :project/source-paths [\"src\"] :project/test-paths [] :project/extension-paths [] :project/capabilities #{} :project/runtime-profiles {:rust {:runtime/source-paths [\"src-rust\"]} :jvm {:runtime/source-paths [\"src-jvm\"]}}}",
    )
    .unwrap();
    fs::write(
        root.join("src-rust/demo/adapter.hal"),
        "(ns demo.adapter) (def runtime :rust)",
    )
    .unwrap();
    fs::write(
        root.join("src-jvm/demo/adapter.hal"),
        "(ns demo.adapter) (def runtime :jvm)",
    )
    .unwrap();
    let project = read(&root).unwrap();
    let mut runtime = Runtime::new();
    register_sources(&project, &mut runtime).unwrap();

    fs::write(
        root.join("src/demo/adapter.hal"),
        "(ns demo.adapter) (def runtime :shared)",
    )
    .unwrap();
    let project = read(&root).unwrap();
    let mut runtime = Runtime::new();
    assert!(register_sources(&project, &mut runtime)
        .unwrap_err()
        .contains("duplicate namespace demo.adapter"));
    fs::remove_dir_all(root).unwrap();
}

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
    let changed = fs::read_to_string(&manifest).unwrap().replace(
        ":project/source-paths [\"src\"]",
        ":project/source-paths [\"src-next\"]",
    );
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
    assert!(!root.read_dir().unwrap().any(|entry| entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .ends_with(".tmp")));
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
