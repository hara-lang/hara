use super::{inspect_direct, WasmInterface, WasmValueType};

const START_SENTINEL: &[u8] = b"\0asm\x01\0\0\0\x08\x01\0";

const SCALAR_INTERFACE: &str = r#"
  (wasm/interface
   {:schema "hara.wasm-interface/0-alpha"
    :namespace math.scalar
    :module "modules/math.wasm"
    :exports
    {add {:wasm/export "add_i64"
          :arguments [{:name left :hara/type :i64 :wasm/type :i64}
                      {:name right :hara/type :i64 :wasm/type :i64}]
          :returns {:hara/type :i64 :wasm/type :i64}}}})"#;

const MEMORY_INTERFACE: &str = r#"
  (wasm/interface
   {:schema "hara.wasm-interface/0-alpha"
    :namespace codec.echo
    :module "echo.wasm"
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

#[test]
fn parses_scalar_interface_without_evaluation() {
    let interface = WasmInterface::parse(SCALAR_INTERFACE, "fixture").unwrap();
    assert_eq!(interface.namespace, "math.scalar");
    assert_eq!(interface.module, "modules/math.wasm");
    assert_eq!(interface.exports[0].name, "add");
    assert_eq!(interface.exports[0].wasm_export, "add_i64");
    assert_eq!(
        interface.exports[0].arguments[0].wasm_type,
        WasmValueType::I64
    );
    assert_eq!(interface.direct_exports()[0].0, "add_i64");
    assert_eq!(interface.digest().len(), 71);
    assert!(interface.digest().starts_with("sha256:"));
    assert_eq!(
        WasmInterface::parse(&interface.canonical_source(), "canonical").unwrap(),
        interface
    );
}

#[test]
fn parses_explicit_memory_semantics_without_executing_them() {
    let interface = WasmInterface::parse(MEMORY_INTERFACE, "fixture").unwrap();
    let memory = interface.memory.as_ref().unwrap();
    assert_eq!(memory.export, "memory");
    assert_eq!(memory.allocate.as_deref(), Some("alloc"));
    assert_eq!(memory.release.as_deref(), Some("free"));
    assert_eq!(
        WasmInterface::parse(&interface.canonical_source(), "canonical").unwrap(),
        interface
    );
}

#[test]
fn static_inspection_records_a_start_function_without_running_it() {
    let inspection = inspect_direct(START_SENTINEL).unwrap();
    assert_eq!(inspection.start, Some(0));
}

#[test]
fn canonicalizes_map_and_set_order() {
    let left = r#"
      {:schema "hara.wasm-interface/0-alpha"
       :namespace math.scalar
       :module "modules/math.wasm"
       :capabilities [:random :clock]
       :exports
       {subtract {:wasm/export "sub"
                  :arguments [{:name left :hara/type :i64 :wasm/type :i64}
                              {:name right :hara/type :i64 :wasm/type :i64}]
                  :returns {:hara/type :i64 :wasm/type :i64}}
        add {:wasm/export "add_i64"
             :arguments [{:name left :hara/type :i64 :wasm/type :i64}
                         {:name right :hara/type :i64 :wasm/type :i64}]
             :returns {:hara/type :i64 :wasm/type :i64}
             :capabilities [:clock :random]}}}"#;
    let right = r#"
      {:exports
       {add {:capabilities [:random :clock]
             :returns {:wasm/type :i64 :hara/type :i64}
             :arguments [{:wasm/type :i64 :hara/type :i64 :name left}
                         {:hara/type :i64 :name right :wasm/type :i64}]
             :wasm/export "add_i64"}
        subtract {:returns {:wasm/type :i64 :hara/type :i64}
                  :wasm/export "sub"
                  :arguments [{:wasm/type :i64 :name left :hara/type :i64}
                              {:name right :wasm/type :i64 :hara/type :i64}]}}
       :module "modules/math.wasm"
       :capabilities [:clock :random]
       :namespace math.scalar
       :schema "hara.wasm-interface/0-alpha"}"#;
    let left = WasmInterface::parse(left, "left").unwrap();
    let right = WasmInterface::parse(right, "right").unwrap();
    assert_eq!(left, right);
    assert_eq!(left.canonical_source(), right.canonical_source());
    assert_eq!(left.digest(), right.digest());
}

#[test]
fn rejects_executable_unknown_duplicate_and_unsafe_sources() {
    for source in [
        "(do (println \"not data\"))".to_owned(),
        SCALAR_INTERFACE.replace(":module \"modules/math.wasm\"", ":module \"../math.wasm\""),
        SCALAR_INTERFACE.replace(
            ":schema \"hara.wasm-interface/0-alpha\"",
            ":schema \"hara.wasm-interface/9\"",
        ),
        SCALAR_INTERFACE.replace(":namespace math.scalar", ":namespace Math.scalar"),
        SCALAR_INTERFACE.replace(":exports", ":unknown true :exports"),
        SCALAR_INTERFACE.replace(":name left", ":name left :name duplicate"),
    ] {
        let error = WasmInterface::parse(&source, "fixture").unwrap_err();
        assert!(error.starts_with("wasm-interface/"));
    }
}

#[test]
fn rejects_ambiguous_and_future_semantics() {
    let mismatch = SCALAR_INTERFACE.replace(
        ":name left :hara/type :i64 :wasm/type :i64",
        ":name left :hara/type :i32 :wasm/type :i64",
    );
    assert!(WasmInterface::parse(&mismatch, "mismatch")
        .unwrap_err()
        .contains("maps :i32 to :i64"));

    let missing_ownership = SCALAR_INTERFACE.replace(
        ":name left :hara/type :i64 :wasm/type :i64",
        ":name left :hara/type :bytes :wasm/type :i32 :lower [:pointer :length]",
    );
    assert!(WasmInterface::parse(&missing_ownership, "bytes")
        .unwrap_err()
        .contains("requires :ownership"));

    let missing_memory = MEMORY_INTERFACE.replace(
        ":memory {:export \"memory\" :allocate \"alloc\" :release \"free\"}",
        "",
    );
    assert!(WasmInterface::parse(&missing_memory, "bytes")
        .unwrap_err()
        .contains("require an explicit :memory contract"));

    let asynchronous = SCALAR_INTERFACE.replace(
        ":returns {:hara/type :i64 :wasm/type :i64}",
        ":returns {:hara/type :i64 :wasm/type :i64} :async true",
    );
    assert!(WasmInterface::parse(&asynchronous, "async")
        .unwrap_err()
        .starts_with("wasm-interface/feature-unsupported"));

    let handles = SCALAR_INTERFACE.replace(
        ":exports",
        ":handles {stream {:tag stream :release \"stream_drop\"}} :exports",
    );
    assert!(WasmInterface::parse(&handles, "handles")
        .unwrap_err()
        .starts_with("wasm-interface/feature-unsupported"));
}
