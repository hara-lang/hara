from pathlib import Path
import re


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one match, found {count}: {old[:80]!r}")
    path.write_text(text.replace(old, new, 1))


project = Path("core/rust/src/project.rs")
replace_once(
    project,
    """/// Registers namespaces from the automatically selected native Rust profile.
pub fn register_sources(project: &Project, runtime: &mut Runtime) -> Result<(), String> {
    let mut resources = Vec::new();""",
    """/// Returns namespace resources from the automatically selected native Rust profile.
pub fn source_resources(project: &Project) -> Result<Vec<(String, String)>, String> {
    let mut resources = Vec::new();""",
)
replace_once(
    project,
    """        resources.push((namespace, source));
    }
    for (namespace, source) in resources {
        runtime.register_resource(&namespace, &source);
    }
    Ok(())
}""",
    """        resources.push((namespace, source));
    }
    Ok(resources)
}

/// Registers namespaces from the automatically selected native Rust profile.
pub fn register_sources(project: &Project, runtime: &mut Runtime) -> Result<(), String> {
    for (namespace, source) in source_resources(project)? {
        runtime.register_resource(&namespace, &source);
    }
    Ok(())
}""",
)

project_tests = Path("core/rust/src/project/tests.rs")
source = project_tests.read_text()
pattern = re.compile(
    r"#\[test\]\nfn registers_project_sources_for_cross_file_requires\(\) \{.*?\n\}\n\n#\[test\]\nfn source_discovery_ignores_editor_artifacts",
    re.S,
)
replacement = r'''#[test]
fn registers_project_sources_for_cross_file_requires() {
    let root = temp("resources");
    fs::create_dir_all(root.join("packages/core/src/demo")).unwrap();
    fs::create_dir_all(root.join("src/demo")).unwrap();
    fs::write(root.join("project.edn"), "{:hara/type :project :hara/version \"1.0.0\" :project/id demo/app :project/version \"1.0.0\" :project/source-paths [\"packages/core/src\" \"src\"] :project/test-paths [] :project/extension-paths [] :project/capabilities #{}}").unwrap();
    fs::write(
        root.join("packages/core/src/demo/helper.hal"),
        "(ns demo.helper) (defn answer [] 40)",
    )
    .unwrap();
    fs::write(
        root.join("src/demo/app.hal"),
        "(ns demo.app (:require [demo.helper :as helper])) (defn answer [] (+ 2 (helper/answer)))",
    )
    .unwrap();
    let project = read(&root).unwrap();
    assert_eq!(
        source_resources(&project)
            .unwrap()
            .into_iter()
            .map(|(namespace, _)| namespace)
            .collect::<Vec<_>>(),
        vec!["demo.helper".to_owned(), "demo.app".to_owned()]
    );
    let mut runtime = Runtime::new();
    register_sources(&project, &mut runtime).unwrap();
    assert_eq!(
        runtime
            .eval_native("(ns demo.main (:require [demo.app :as app])) (app/answer)")
            .unwrap(),
        "42"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_discovery_ignores_editor_artifacts'''
updated, count = pattern.subn(replacement, source, count=1)
if count != 1:
    raise SystemExit(f"{project_tests}: expected one project test replacement, found {count}")
project_tests.write_text(updated)

repl = Path("core/rust/src/bin/hara/repl.rs")
replace_once(
    repl,
    """use crate::cli::Options;
use hara_wasm::native_cli::RuntimeBroker;
use hara_wasm::resp::{RespConnection, RespServer, RespValue};""",
    """use crate::cli::Options;
use hara_wasm::native_cli::RuntimeBroker;
use hara_wasm::project;
use hara_wasm::resp::{RespConnection, RespServer, RespValue};""",
)
replace_once(
    repl,
    """pub(crate) fn run_repl(options: &Options, offline: bool) -> Result<(), String> {
    let broker = RuntimeBroker::start_with(""",
    """fn register_project_resources(options: &Options, broker: &RuntimeBroker) -> Result<(), String> {
    let Some(path) = options.project.as_deref() else {
        return Ok(());
    };
    let selected = project::discover(path)?;
    for (namespace, source) in project::source_resources(&selected)? {
        broker.register_resource(&namespace, &source)?;
    }
    Ok(())
}

pub(crate) fn run_repl(options: &Options, offline: bool) -> Result<(), String> {
    let broker = RuntimeBroker::start_with(""",
)
replace_once(
    repl,
    """        options.allow_postgres,
    )?;
    let mut resp = RespController::new(options.host.clone(), options.port, broker.clone());""",
    """        options.allow_postgres,
    )?;
    register_project_resources(options, &broker)?;
    let mut resp = RespController::new(options.host.clone(), options.port, broker.clone());""",
)
replace_once(
    repl,
    """mod tests {
    use super::{command_hint, fuzzy_score, gradient, incomplete, rendered_splash, DEFAULT_SPLASH};""",
    """mod tests {
    use super::{
        command_hint, fuzzy_score, gradient, incomplete, register_project_resources,
        rendered_splash, DEFAULT_SPLASH,
    };
    use crate::cli::Options;
    use hara_wasm::native_cli::RuntimeBroker;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "hara-repl-{name}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }""",
)
replace_once(
    repl,
    """    #[test]
    fn completion_scoring_and_hints_match_java_behavior() {
        assert!(fuzzy_score("mp", "map").is_some());
        assert!(fuzzy_score("zzz", "map").is_none());
        assert_eq!(command_hint("/re"), Some("sp".into()));
        assert!(command_hint("/docs").unwrap().contains("language"));
    }
}""",
    """    #[test]
    fn completion_scoring_and_hints_match_java_behavior() {
        assert!(fuzzy_score("mp", "map").is_some());
        assert!(fuzzy_score("zzz", "map").is_none());
        assert_eq!(command_hint("/re"), Some("sp".into()));
        assert!(command_hint("/docs").unwrap().contains("language"));
    }

    #[test]
    fn project_resources_reach_root_and_future_sessions() {
        let root = temp("project-resources");
        fs::create_dir_all(root.join("packages/core/src/demo")).unwrap();
        fs::create_dir_all(root.join("src/demo")).unwrap();
        fs::write(root.join("project.edn"), "{:hara/type :project :hara/version \\"1.0.0\\" :project/id demo/repl :project/version \\"1.0.0\\" :project/source-paths [\\"packages/core/src\\" \\"src\\"] :project/test-paths [] :project/extension-paths [] :project/capabilities #{}}").unwrap();
        fs::write(
            root.join("packages/core/src/demo/helper.hal"),
            "(ns demo.helper) (defn answer [] 40)",
        )
        .unwrap();
        fs::write(
            root.join("src/demo/app.hal"),
            "(ns demo.app (:require [demo.helper :as helper])) (defn answer [] (+ 2 (helper/answer)))",
        )
        .unwrap();

        let mut options = Options::default();
        options.project = Some(root.clone());
        options.root = Some(root.clone());
        let broker = RuntimeBroker::start_with(Some(root.clone()), false, false, false).unwrap();
        register_project_resources(&options, &broker).unwrap();
        assert_eq!(broker.resources().unwrap(), vec!["demo.app", "demo.helper"]);
        assert_eq!(
            broker
                .eval(
                    "ROOT",
                    "(ns repl.probe (:require [demo.app :as app])) (app/answer)",
                )
                .unwrap(),
            "42"
        );
        broker.create("SECOND").unwrap();
        assert_eq!(
            broker
                .eval(
                    "SECOND",
                    "(ns repl.second (:require [demo.app :as app])) (app/answer)",
                )
                .unwrap(),
            "42"
        );
        fs::remove_dir_all(root).unwrap();
    }
}""",
)
