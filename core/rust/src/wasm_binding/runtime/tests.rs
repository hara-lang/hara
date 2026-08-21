use crate::core::Value;

use super::{WasmtimeMemoryExecutor, MAX_VALUE_BYTES};
use crate::wasm_binding::WasmInterface;

const MEMORY_MODULE: &[u8] = b"\0asm\x01\0\0\0\x01\x14\x04\x60\x01\x7f\x01\x7f\x60\x01\x7f\0\x60\x02\x7f\x7f\x01\x7e\x60\0\x01\x7f\x03\x05\x04\0\x01\x02\x03\x05\x04\x01\x01\x01\x10\x06\x06\x01\x7f\x01\x41\0\x0b\x07\x36\x05\x06memory\x02\0\x05alloc\0\0\x04free\0\x01\x0aecho_bytes\0\x02\x0drelease_count\0\x03\x0a\x23\x04\x05\0\x41\x80\x08\x0b\x09\0\x23\0\x41\x01\x6a\x24\0\x0b\x0c\0\x20\x01\xad\x42\x20\x86\x20\0\xad\x84\x0b\x04\0\x23\0\x0b";

fn interface(value_type: &str, ownership: &str) -> WasmInterface {
    let source = format!(
        r#"(wasm/interface
             {{:schema "hara.wasm-interface/0-alpha"
               :namespace codec.echo
               :module "echo.wasm"
               :memory {{:export "memory" :allocate "alloc" :release "free"}}
               :exports
               {{echo {{:wasm/export "echo_bytes"
                        :arguments [{{:name input
                                     :hara/type :{value_type}
                                     :wasm/type :i32
                                     :lower [:pointer :length]
                                     :ownership :borrowed}}]
                        :returns {{:hara/type :{value_type}
                                  :wasm/type :i64
                                  :lift :packed-i64
                                  :ownership :{ownership}}}
                release-count {{:wasm/export "release_count"
                                :arguments []
                                :returns {{:hara/type :i32 :wasm/type :i32}}}}}}}})"#
    );
    WasmInterface::parse(&source, "fixture").unwrap()
}

#[test]
fn lowers_and_lifts_bytes_with_exactly_once_release() {
    let executor = WasmtimeMemoryExecutor::compile(
        MEMORY_MODULE,
        interface("bytes", "caller").memory_plan().unwrap(),
    )
    .unwrap();
    let value = Value::Bytes(vec![1, 2, 3, 4]);
    assert_eq!(executor.invoke("echo", &[value.clone()]).unwrap(), value);
    assert_eq!(
        executor.invoke("release-count", &[]).unwrap(),
        Value::Number(1)
    );
}

#[test]
fn lowers_and_lifts_utf8_strings_through_the_same_executor() {
    let executor = WasmtimeMemoryExecutor::compile(
        MEMORY_MODULE,
        interface("string", "caller").memory_plan().unwrap(),
    )
    .unwrap();
    let value = Value::String("hara memory binding".into());
    assert_eq!(executor.invoke("echo", &[value.clone()]).unwrap(), value);
    assert_eq!(
        executor.invoke("release-count", &[]).unwrap(),
        Value::Number(1)
    );
}

#[test]
fn rejects_oversized_inputs_before_allocating_module_memory() {
    let executor = WasmtimeMemoryExecutor::compile(
        MEMORY_MODULE,
        interface("bytes", "caller").memory_plan().unwrap(),
    )
    .unwrap();
    let error = executor
        .invoke("echo", &[Value::Bytes(vec![0; MAX_VALUE_BYTES + 1])])
        .unwrap_err();
    assert!(error.starts_with("extension/resource-limit"));
    assert_eq!(
        executor.invoke("release-count", &[]).unwrap(),
        Value::Number(0)
    );
}

#[test]
fn transferred_inputs_are_released_only_when_the_call_does_not_complete() {
    let source = r#"
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
                            :ownership :transferred}]
               :returns {:hara/type :bytes
                         :wasm/type :i64
                         :lift :packed-i64
                         :ownership :callee}}
         release-count {:wasm/export "release_count"
                        :arguments []
                        :returns {:hara/type :i32 :wasm/type :i32}}})"#
    let plan = WasmInterface::parse(source, "fixture")
        .unwrap()
        .memory_plan()
        .unwrap();
    let executor = WasmtimeMemoryExecutor::compile(MEMORY_MODULE, plan).unwrap();
    let value = Value::Bytes(vec![9, 8, 7]);
    assert_eq!(executor.invoke("echo", &[value.clone()]).unwrap(), value);
    assert_eq!(
        executor.invoke("release-count", &[]).unwrap(),
        Value::Number(0)
    );
}
