use super::super::source::SourceModule;
use super::super::{analyze_modules, BuildPlan};

fn plan(entrypoint: &str) -> BuildPlan {
    BuildPlan {
        project_id: "demo-app".into(),
        project_version: "0.1.0".into(),
        profile: "production".into(),
        language: "hara".into(),
        main: "app.main".into(),
        entrypoints: vec![entrypoint.into()],
        keep_vars: Vec::new(),
        keep_namespaces: Vec::new(),
        output_bundle: "target/demo-app-production.hbx".into(),
        output_report: "target/demo-app-production.shake.edn".into(),
    }
}

#[test]
fn reports_unbounded_dynamic_access_at_its_source_unit() {
    let modules = vec![SourceModule::synthetic(
        "app.main",
        "(ns app.main)\n(defn start [target] (resolve target))\n",
    )];
    let analysis = analyze_modules(&plan("app.main/start"), modules.clone()).unwrap();
    let diagnostic = analysis
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "production/unbounded-dynamic-var")
        .expect("unbounded resolve must fail production analysis");
    assert_eq!(diagnostic.location.path, "fixture:app.main");
    assert!(diagnostic.location.line >= 2);

    let mut bounded = plan("app.main/start");
    bounded.keep_vars = vec!["app.main/start".into()];
    let analysis = analyze_modules(&bounded, modules).unwrap();
    assert!(!analysis
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "production/unbounded-dynamic-var"));
}

#[test]
fn literal_dynamic_targets_become_ordinary_edges() {
    let modules = vec![
        SourceModule::synthetic("app.handlers", "(ns app.handlers)\n(defn run [] 42)\n"),
        SourceModule::synthetic(
            "app.main",
            "(ns app.main (:require [app.handlers]))\n(defn start [] (resolve 'app.handlers/run))\n",
        ),
    ];
    let analysis = analyze_modules(&plan("app.main/start"), modules).unwrap();
    assert!(analysis.runtime_closure.contains("app.handlers/run"));
    assert!(analysis.retained_vars.contains("app.handlers/run"));
}
