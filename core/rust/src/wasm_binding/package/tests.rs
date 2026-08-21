use std::fs;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

use super::{bind_package, inspect_module, BindingTarget, TEMP_SEQUENCE};

const ADD: &[u8] = b"\0asm\x01\0\0\0\x01\x07\x01\x60\x02\x7e\x7e\x01\x7e\x03\x02\x01\0\x07\x07\x01\x03add\0\0\x0a\x09\x01\x07\0\x20\0\x20\x01\x7c\x0b";
const MEMORY_MODULE: &[u8] = b"\0asm\x01\0\0\0\x01\x10\x03\x60\x01\x7f\x01\x7f\x60\x01\x7f\0\x60\x02\x7f\x7f\x01\x7e\x03\x04\x03\0\x01\x02\x05\x04\x01\x01\x01\x10\x07\x26\x04\x06memory\x02\0\x05alloc\0\0\x04free\0\x01\x0aecho_bytes\0\x02\x0a\x0e\x03\x04\0\x41\0\x0b\x02\0\x0b\x04\0\x42\0\x0b";

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

const MEMORY_INTERFACE: &str = r#"
  (wasm/interface
   {:schema "hara.wasm-interface/0-alpha"
    :namespace codec.echo
    :module "modules/echo.wasm"
    :memory {:export "memory" :allocate "alloc" :release "free"}
    :exports
    {echo {:wasm/export "echo_bytes"
           :arguments [{:name input
                        :hara/type :bytes
                        :wasm/type :i32
                        :lower [:pointer :length]
                        :ownership :borrowed}]
           :returns {:hara/type :bytes
                     :wasm/type :i64
                     :lift :packed-i64
                     :ownership :caller}}}})"#;

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
fn direct_binding_is_deterministic_atomic_and_language_neutral() {
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
    assert_eq!(bound.target, BindingTarget::CoreV1);
    for relative in &bound.files {
        assert_eq!(
            fs::read(first.join(relative)).unwrap(),
            fs::read(second.join(relative)).unwrap()
        );
    }
    let project = fs::read_to_string(first.join("project.edn")).unwrap();
    assert!(project.contains(":abi :core.v1"));
    assert!(project.contains("\"sum\" {:wasm/export \"add\""));
    assert_language_neutral(&bound.files);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn memory_binding_emits_a_truthful_semantic_package() {
    let root = fixture_root("memory-bind");
    fs::create_dir_all(&root).unwrap();
    let module = root.join("echo.wasm");
    let interface = root.join("echo.interface.hal");
    fs::write(&module, MEMORY_MODULE).unwrap();
    fs::write(&interface, MEMORY_INTERFACE).unwrap();
    let first = root.join("first");
    let second = root.join("second");
    let bound = bind_package(&interface, &module, &first).unwrap();
    bind_package(&interface, &module, &second).unwrap();

    assert_eq!(bound.namespace, "codec.echo");
    assert_eq!(bound.module, "modules/echo.wasm");
    assert_eq!(bound.target, BindingTarget::MemoryV1);
    for relative in &bound.files {
        assert_eq!(
            fs::read(first.join(relative)).unwrap(),
            fs::read(second.join(relative)).unwrap()
        );
    }

    let project = fs::read_to_string(first.join("project.edn")).unwrap();
    assert!(project.contains(":abi :memory.v1"));
    assert!(project.contains(":args [:bytes]"));
    assert!(project.contains(":returns :bytes"));
    assert!(project.contains("\"bindings.edn\""));
    let bindings = fs::read_to_string(first.join("bindings.edn")).unwrap();
    assert!(bindings.contains("hara.wasm-memory-binding/0-alpha"));
    assert!(bindings.contains(":target :memory.v1"));
    assert!(bindings.contains(":wasm/arguments [:i32 :i32]"));
    let product = fs::read_to_string(first.join("hara.build-product.edn")).unwrap();
    assert!(product.contains(":product/target :memory.v1"));
    assert_language_neutral(&bound.files);
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

fn assert_language_neutral(files: &[String]) {
    assert!(!files.iter().any(|path| {
        [".js", ".mjs", ".rs", ".java", ".c"]
            .iter()
            .any(|suffix| path.ends_with(suffix))
    }));
}
