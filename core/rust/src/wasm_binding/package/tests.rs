use std::fs;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

use super::{bind_package, inspect_module, TEMP_SEQUENCE};

const ADD: &[u8] = b"\0asm\x01\0\0\0\x01\x07\x01\x60\x02\x7e\x7e\x01\x7e\x03\x02\x01\0\x07\x07\x01\x03add\0\0\x0a\x09\x01\x07\0\x20\0\x20\x01\x7c\x0b";
const INTERFACE: &str = r#"
  (wasm/interface
   {:schema "hara.wasm-interface/0-alpha"
    :namespace math.scalar
    :module "modules/math.wasm"
    :exports
    {sum {:wasm/export "add"
          :arguments [{:name left :hara/type :i64 :wasm/type :i64}
                      {:name right :hara/type :i64 :wasm/type :i64}]
          :returns {:hara/type :i64 :wasm/type :i64}}}})"#;

fn fixture_root(name: &str) -> PathBuf {
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "hara-wasm-bindgen-{name}-{}-{sequence}",
        std::process::id()
    ))
}

#[test]
fn inspection_keeps_machine_semantics_unresolved() {
    let root = fixture_root("inspect");
    fs::create_dir_all(&root).unwrap();
    let module = root.join("scalar_math.wasm");
    fs::write(&module, ADD).unwrap();
    let inspected = inspect_module(&module, None).unwrap();
    assert_eq!(inspected.namespace, "generated.scalar-math");
    assert!(inspected
        .interface_source
        .contains(":hara/type :unresolved"));
    assert!(inspected.inspection_source.contains(":returns :i64"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn binding_is_deterministic_atomic_and_language_neutral() {
    let root = fixture_root("bind");
    fs::create_dir_all(&root).unwrap();
    let module = root.join("math.wasm");
    let interface = root.join("interface.input.hal");
    fs::write(&module, ADD).unwrap();
    fs::write(&interface, INTERFACE).unwrap();
    let first = root.join("first");
    let second = root.join("second");
    let bound = bind_package(&interface, &module, &first).unwrap();
    bind_package(&interface, &module, &second).unwrap();
    assert_eq!(bound.namespace, "math.scalar");
    assert_eq!(bound.module, "modules/math.wasm");
    for relative in &bound.files {
        assert_eq!(
            fs::read(first.join(relative)).unwrap(),
            fs::read(second.join(relative)).unwrap()
        );
    }
    let project = fs::read_to_string(first.join("project.edn")).unwrap();
    assert!(project.contains("\"sum\" {:wasm/export \"add\""));
    assert!(!bound.files.iter().any(|path| {
        [".js", ".mjs", ".rs", ".java", ".c"]
            .iter()
            .any(|suffix| path.ends_with(suffix))
    }));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn drift_fails_before_creating_an_output_tree() {
    let root = fixture_root("drift");
    fs::create_dir_all(&root).unwrap();
    let module = root.join("math.wasm");
    let interface = root.join("interface.input.hal");
    fs::write(&module, ADD).unwrap();
    fs::write(
        &interface,
        INTERFACE.replace(
            ":returns {:hara/type :i64 :wasm/type :i64}",
            ":returns {:hara/type :i32 :wasm/type :i32}",
        ),
    )
    .unwrap();
    let output = root.join("output");
    assert!(bind_package(&interface, &module, &output)
        .unwrap_err()
        .starts_with("wasm-binding/signature-mismatch"));
    assert!(!output.exists());
    fs::remove_dir_all(root).unwrap();
}
