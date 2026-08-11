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
fn creates_and_validates_an_empty_lock() {
    let root = temp("lock");
    let project = new_app(&root, "lock-app").unwrap();
    let lock = sync_lock(&project, LockMode::Default).unwrap();
    assert_eq!(
        fs::read_to_string(&lock).unwrap(),
        "{:lock/format \"0.0.0-alpha\" :packages {}}\n"
    );
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
