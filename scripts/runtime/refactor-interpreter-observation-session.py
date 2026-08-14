#!/usr/bin/env python3
"""Move live interpreter session ownership into hara-wasm; leave a thin raw ABI."""

from pathlib import Path

root = Path(__file__).resolve().parents[2]
raw_path = root / "core/rust/interpreter-observation-raw/src/lib.rs"
source = raw_path.read_text()

constants = source.index("const ABI_VERSION")
main = """use crate::core::{EvalFiber, EvalFiberState, Value};
use crate::task::{PromiseRejection, PromiseState};
use crate::{core, json, Runtime};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

""" + source[constants:]
main = main.replace("const ABI_VERSION: i32 = 1;", "pub const ABI_VERSION: i32 = 1;", 1)

context_start = main.index("struct InterpreterContext {")
context_end = main.index("struct InterpreterObservationSession {")
context = """struct InterpreterContext {
    runtime: Runtime,
}

impl InterpreterContext {
    fn fresh() -> (Self, HashMap<String, Value>) {
        let runtime = Runtime::new();
        let environment = runtime.env.clone();
        (Self { runtime }, environment)
    }

    fn run<T>(&self, operation: impl FnOnce() -> T) -> T {
        let namespaces = self.runtime.namespace_registry.clone();
        let protocols = self.runtime.protocols.clone();
        let macros = self.runtime.macros.clone();
        core::with_macros(macros, move || {
            core::with_namespace_registry(&namespaces, move || {
                core::with_protocols(&protocols, operation)
            })
        })
    }
}

"""
main = main[:context_start] + context + main[context_end:]

abi_start = main.index("#[no_mangle]\npub extern \"C\" fn interpreter_observation_abi_version")
invoke_start = main.index("fn invoke_json(source: &str)", abi_start)
main = main[:abi_start] + main[invoke_start:]
main = main.replace(
    "fn invoke_json(source: &str) -> Vec<u8>",
    "pub fn invoke_json(source: &str) -> Vec<u8>",
    1,
)

alloc_start = main.index("fn allocate_bytes(size: usize)")
tests_start = main.index("#[cfg(test)]", alloc_start)
main = main[:alloc_start] + main[tests_start:]
main = main.replace(
    'Value::Keyword(key) if key.as_str() == name => Some(value.clone()),',
    'Value::Keyword(key) if key.as_str() == Some(name) => Some(value.clone()),',
)

main_path = root / "core/rust/src/interpreter_observation.rs"
main_path.write_text(main)

lib_path = root / "core/rust/src/lib.rs"
lib = lib_path.read_text()
needle = "#[cfg(feature = \"evaluation-journal\")]\npub mod journal;\n"
replacement = needle + "pub mod interpreter_observation;\n"
if lib.count(needle) != 1:
    raise SystemExit("lib.rs interpreter observation insertion seam changed")
lib_path.write_text(lib.replace(needle, replacement, 1))

raw_path.write_text(
    r'''use hara_wasm::interpreter_observation::{invoke_json, ABI_VERSION};

#[no_mangle]
pub extern "C" fn interpreter_observation_abi_version() -> i32 {
    ABI_VERSION
}

#[no_mangle]
pub extern "C" fn interpreter_observation_alloc(size: usize) -> *mut u8 {
    allocate_bytes(size)
}

#[no_mangle]
pub extern "C" fn interpreter_observation_dealloc(pointer: *mut u8, size: usize) {
    free_bytes(pointer, size);
}

/// Accepts one UTF-8 JSON request and returns packed `(pointer << 32) | len`.
#[no_mangle]
pub extern "C" fn interpreter_observation_invoke(pointer: *const u8, size: usize) -> u64 {
    let response = if pointer.is_null() {
        invoke_json("{}")
    } else {
        let bytes = unsafe { std::slice::from_raw_parts(pointer, size) };
        match std::str::from_utf8(bytes) {
            Ok(source) => invoke_json(source),
            Err(_) => invoke_json("{}"),
        }
    };
    pack_response(response)
}

fn allocate_bytes(size: usize) -> *mut u8 {
    let bytes = vec![0_u8; size.max(1)].into_boxed_slice();
    Box::into_raw(bytes) as *mut u8
}

fn free_bytes(pointer: *mut u8, size: usize) {
    if pointer.is_null() {
        return;
    }
    let length = size.max(1);
    unsafe {
        let slice = std::ptr::slice_from_raw_parts_mut(pointer, length);
        drop(Box::from_raw(slice));
    }
}

fn pack_response(bytes: Vec<u8>) -> u64 {
    let length = bytes.len();
    let pointer = Box::into_raw(bytes.into_boxed_slice()) as *mut u8;
    ((pointer as u64) << 32) | length as u64
}

#[cfg(test)]
mod tests {
    use super::invoke_json;

    #[test]
    fn thin_raw_abi_delegates_to_the_runtime_owned_session() {
        let response = invoke_json(
            r#"{"op":"start","sessionId":"raw/smoke","sourceId":"smoke.hal","source":"(+ 1 2)"}"#,
        );
        let response = std::str::from_utf8(&response).unwrap();
        assert!(response.contains("\"ok\":true"));
        assert!(response.contains("\"handle\":1"));
    }
}
'''
)

cargo_path = root / "core/rust/interpreter-observation-raw/Cargo.toml"
cargo_path.write_text(
    '''[package]
name = "hara-interpreter-observation-raw"
version = "0.1.6"
edition = "2021"
description = "On-demand browser-safe live interpreter observation runtime for Hara"
license = "Apache-2.0"
rust-version = "1.78"
repository = "https://github.com/hara-lang/hara"
homepage = "https://www.hara-lang.org"
publish = false

# Keep the C ABI outside the ordinary runtime workspace. The implementation
# and session ownership live in hara-wasm so this facade cannot drift from the
# production evaluator or duplicate its module graph.
[workspace]

[lib]
name = "hara_interpreter_observation_raw"
crate-type = ["cdylib", "rlib"]

[profile.browser-release]
inherits = "release"
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = "symbols"

[dependencies]
hara-wasm = { path = "..", default-features = false }
'''
)
