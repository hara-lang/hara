use super::{
    apply_manage_edits, eval_runtime, manage_arguments, manage_editor_json, manage_units,
    test_results, ManageFormat, Options,
};
use hara_wasm::kernel::parse;
use hara_wasm::project;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempProject(PathBuf);

impl TempProject {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("hara-project-eval-{}-{nonce}", std::process::id()));
        fs::create_dir_all(root.join("src/demo")).unwrap();
        fs::create_dir_all(root.join("test/demo")).unwrap();
        fs::write(root.join("project.edn"), "{:hara/type :project\n :hara/version \"1.0.0\"\n :project/id demo/project-eval\n :project/version \"0.1.0\"\n :project/source-paths [\"src\"]\n :project/test-paths [\"test\"]\n :project/extension-paths []\n :project/capabilities #{}\n :project/dependencies {}}\n").unwrap();
        fs::write(
            root.join("src/demo/rules.hal"),
            "(ns demo.rules)\n\n(defn answer [] 42)\n",
        )
        .unwrap();
        Self(root)
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn project_eval_registers_sources_without_a_root_mount() {
    let project = TempProject::new();
    let options = Options {
        project: Some(project.0.clone()),
        ..Options::default()
    };
    assert!(options.root.is_none());
    let mut runtime = eval_runtime(&options).unwrap();
    assert_eq!(
        runtime
            .eval_native(
                "(ns demo.invoke\n  (:require [demo.rules :as rules]))\n\n(rules/answer)\n"
            )
            .unwrap(),
        "42"
    );
}

#[test]
fn project_test_accepts_native_test_vectors() {
    assert_eq!(
        test_results("[{:name \"yes\" :pass true} {:name \"no\" :pass false}]").unwrap(),
        (1, 1)
    );
}

#[test]
fn project_test_accepts_code_test_summaries() {
    assert_eq!(
        test_results("{:status :failed :counts {:passed 3 :failed 1 :error 1 :timeout 1}}")
            .unwrap(),
        (3, 3)
    );
}

#[test]
fn project_test_keeps_printed_legacy_vectors_compatible() {
    assert_eq!(
        test_results("\"[{:name \\\"yes\\\" :pass true}]\"").unwrap(),
        (1, 0)
    );
}

#[test]
fn manage_arguments_keep_writes_explicit_and_values_in_data() {
    let parsed = manage_arguments(
        "grep",
        &[
            "demo.core".into(),
            "--from".into(),
            "demo.core".into(),
            "--to".into(),
            "demo.next".into(),
            "--pattern".into(),
            "TODO".into(),
            "--pattern".into(),
            "FIXME".into(),
            "--write".into(),
        ],
    )
    .unwrap();
    assert!(parsed.write);
    assert_eq!(parsed.namespaces, ["demo.core"]);
    assert!(parsed.options.contains(":from \"demo.core\""));
    assert!(parsed.options.contains(":to \"demo.next\""));
    assert!(parsed.options.contains(":patterns [\"TODO\" \"FIXME\"]"));
}

#[test]
fn manage_arguments_support_editor_json_and_added_override() {
    let parsed = manage_arguments(
        "scaffold",
        &[
            "demo.core".into(),
            "--added".into(),
            "5.8".into(),
            "--format".into(),
            "editor-json".into(),
        ],
    )
    .unwrap();
    assert_eq!(parsed.format, ManageFormat::EditorJson);
    assert_eq!(parsed.namespaces, ["demo.core"]);
    assert!(parsed.options.contains(":added \"5.8\""));
}

#[test]
fn manage_arguments_reject_obsolete_import_and_purge_modes() {
    assert!(manage_arguments(
        "import",
        &["demo.core".into(), "--form".into(), "(def x 1)".into()]
    )
    .unwrap_err()
    .contains("import --form is obsolete"));
    assert!(manage_arguments(
        "purge",
        &["demo.core".into(), "--pattern".into(), "TODO".into()]
    )
    .unwrap_err()
    .contains("purge --pattern is obsolete"));
}

#[test]
fn manage_units_pair_test_source_and_project_version() {
    let fixture = TempProject::new();
    fs::write(
        fixture.0.join("test/demo/rules_test.hal"),
        "(ns demo.rules-test)\n",
    )
    .unwrap();
    let project = project::discover(&fixture.0).unwrap();
    let units = manage_units(&project, "scaffold", &["demo.rules".into()]).unwrap();
    assert!(units.contains(":test-path"));
    assert!(units.contains("rules_test.hal"));
    assert!(units.contains(":test-source \"(ns demo.rules-test)\\n\""));
    assert!(units.contains(":version \"0.1.0\""));
    assert!(units.contains(":project-version \"0.1.0\""));
}

#[test]
fn editor_json_has_stable_schema_and_edit_shape() {
    let plan = parse(
        "{:operation :scaffold\n          :summary {:changed 1 :findings 0}\n          :findings []\n          :edits [{:path \"test/demo/core_test.hal\"\n                   :before \"\" :after \"(ns demo.core-test)\\n\"\n                   :changed true :create true :new [answer]}]}",
    )
    .unwrap();
    let json = manage_editor_json(&plan, false).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["schema"], "code.manage.editor/0-alpha");
    assert_eq!(value["operation"], "scaffold");
    assert_eq!(value["write"], false);
    assert_eq!(value["edits"][0]["changed"], true);
    assert_eq!(value["edits"][0]["create"], true);
    assert!(value["edits"][0].get("new").is_none());
}

#[test]
fn manage_edits_preflight_stale_content_before_writing() {
    let project = TempProject::new();
    let path = project.0.join("src/demo/rules.hal");
    let plan = parse(&format!(
        "{{:edits [{{:path {:?} :before {:?} :after {:?}}}]}}",
        path.to_string_lossy(),
        "stale",
        "replacement"
    ))
    .unwrap();
    assert!(apply_manage_edits(&project.0, &plan)
        .unwrap_err()
        .contains("stale edit"));
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        "(ns demo.rules)\n\n(defn answer [] 42)\n"
    );
}

#[test]
fn manage_edits_apply_validated_content_inside_project() {
    let project = TempProject::new();
    let path = project.0.join("src/demo/rules.hal");
    let before = fs::read_to_string(&path).unwrap();
    let after = before.replace("42", "43");
    let plan = parse(&format!(
        "{{:edits [{{:path {:?} :before {:?} :after {:?}}}]}}",
        path.to_string_lossy(),
        before,
        after
    ))
    .unwrap();
    apply_manage_edits(&project.0, &plan).unwrap();
    assert!(fs::read_to_string(path).unwrap().contains("43"));
}

#[cfg(unix)]
#[test]
fn manage_edits_reject_symlink_targets_outside_project() {
    use std::os::unix::fs::symlink;

    let project = TempProject::new();
    let outside = project.0.parent().unwrap().join(format!(
        "outside-symlink-{}-{}.hal",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::write(&outside, "outside").unwrap();
    let link = project.0.join("src/demo/link.hal");
    symlink(&outside, &link).unwrap();
    let plan = parse(&format!(
        "{{:edits [{{:path {:?} :before {:?} :after {:?}}}]}}",
        link.to_string_lossy(),
        "outside",
        "replacement"
    ))
    .unwrap();
    assert!(apply_manage_edits(&project.0, &plan)
        .unwrap_err()
        .contains("escapes project root"));
    assert_eq!(fs::read_to_string(&outside).unwrap(), "outside");
    fs::remove_file(outside).unwrap();
}

#[test]
fn manage_edits_reject_paths_outside_project() {
    let project = TempProject::new();
    let outside = project.0.parent().unwrap().join("outside.hal");
    let plan = parse(&format!(
        "{{:edits [{{:path {:?} :before {:?} :after {:?}}}]}}",
        outside.to_string_lossy(),
        "",
        "replacement"
    ))
    .unwrap();
    assert!(apply_manage_edits(&project.0, &plan)
        .unwrap_err()
        .contains("escapes project root"));
    assert!(!outside.exists());
}
