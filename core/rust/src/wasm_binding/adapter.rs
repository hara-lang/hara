#![cfg(not(target_arch = "wasm32"))]

use sha2::{Digest, Sha256};
use wasm_encoder::{
    EntityType, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction,
    Module, TypeSection, ValType,
};

use crate::kernel::Form;

use super::{inspect_direct, BindingFunction, WasmInterface, WasmValueType};

pub const ADAPTER_MANIFEST_SCHEMA: &str = "hara.wasm-adapter/0-alpha";
const ADAPTER_TARGET: &str = "core.v1-forward";
const LIBRARY_IMPORT_MODULE: &str = "hara/library";

/// A deterministic adapter module and the manifest describing its composition.
///
/// The first adapter revision is deliberately a scalar forwarding boundary:
/// the adapter imports the verified library exports under one stable module
/// name and exports the Hara-facing names. Rich memory and HTA lifecycle
/// operations remain explicit follow-up revisions rather than guessed from
/// machine-level values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterArtifact {
    pub bytes: Vec<u8>,
    pub manifest: String,
    pub module_digest: String,
    pub interface_digest: String,
    pub adapter_digest: String,
}

/// Generate the portable scalar adapter for a verified library/interface pair.
///
/// Inspection only parses the module bytes. It never instantiates the wrapped
/// library or runs its start function.
pub fn generate_adapter(
    module_bytes: &[u8],
    interface: &WasmInterface,
) -> Result<AdapterArtifact, String> {
    let inspection = inspect_direct(module_bytes)?;
    if inspection.start.is_some() {
        return Err("wasm-adapter/start-denied: wrapped module declares a start function".into());
    }
    interface.verify_direct(&inspection)?;
    let exports = ordered_exports(interface)?;

    let bytes = emit_forwarder(&exports)?;
    let module_digest = digest(module_bytes);
    let interface_digest = interface.digest();
    let adapter_digest = digest(&bytes);
    let manifest = adapter_manifest(
        interface,
        &module_digest,
        &interface_digest,
        &adapter_digest,
        &exports,
    );

    Ok(AdapterArtifact {
        bytes,
        manifest,
        module_digest,
        interface_digest,
        adapter_digest,
    })
}

fn ordered_exports(interface: &WasmInterface) -> Result<Vec<BindingFunction>, String> {
    let mut exports = interface.exports.clone();
    exports.sort_by(|left, right| left.name.cmp(&right.name));
    for pair in exports.windows(2) {
        if pair[0].name == pair[1].name {
            return Err(format!(
                "wasm-adapter/export-ambiguous: duplicate Hara export {}",
                pair[0].name
            ));
        }
    }
    Ok(exports)
}

fn emit_forwarder(exports: &[BindingFunction]) -> Result<Vec<u8>, String> {
    let mut module = Module::new();
    let mut types = TypeSection::new();

    for export in exports {
        types.function(
            export
                .arguments
                .iter()
                .map(|argument| val_type(argument.wasm_type)),
            result_types(export.returns.wasm_type),
        );
    }
    module.section(&types);

    let mut imports = ImportSection::new();
    for (index, export) in exports.iter().enumerate() {
        imports.import(
            LIBRARY_IMPORT_MODULE,
            &export.wasm_export,
            EntityType::Function(index as u32),
        );
    }
    module.section(&imports);

    let mut functions = FunctionSection::new();
    for index in 0..exports.len() {
        functions.function(index as u32);
    }
    module.section(&functions);

    let mut exports_section = ExportSection::new();
    let import_count = exports.len() as u32;
    for (index, export) in exports.iter().enumerate() {
        exports_section.export(&export.name, ExportKind::Func, import_count + index as u32);
    }
    module.section(&exports_section);

    let mut code = wasm_encoder::CodeSection::new();
    for (index, export) in exports.iter().enumerate() {
        let mut function = Function::new([]);
        for argument in 0..export.arguments.len() {
            function.instruction(&Instruction::LocalGet(argument as u32));
        }
        function.instruction(&Instruction::Call(index as u32));
        function.instruction(&Instruction::End);
        code.function(&function);
    }
    module.section(&code);
    Ok(module.finish())
}

fn adapter_manifest(
    interface: &WasmInterface,
    module_digest: &str,
    interface_digest: &str,
    adapter_digest: &str,
    exports: &[BindingFunction],
) -> String {
    let exports = exports
        .iter()
        .map(|export| {
            Form::Map(vec![
                (keyword("hara/name"), symbol(&export.name)),
                (keyword("wasm/export"), string(&export.wasm_export)),
            ])
        })
        .collect();
    Form::Map(vec![
        (keyword("schema"), string(ADAPTER_MANIFEST_SCHEMA)),
        (keyword("target"), keyword(ADAPTER_TARGET)),
        (keyword("namespace"), symbol(&interface.namespace)),
        (
            keyword("composition"),
            Form::Map(vec![
                (keyword("import-module"), string(LIBRARY_IMPORT_MODULE)),
                (keyword("library"), string(&interface.module)),
            ]),
        ),
        (
            keyword("inputs"),
            Form::Map(vec![
                (keyword("module-digest"), string(module_digest)),
                (keyword("interface-digest"), string(interface_digest)),
            ]),
        ),
        (keyword("adapter-digest"), string(adapter_digest)),
        (
            keyword("tool"),
            Form::Map(vec![
                (keyword("name"), string("hara-wasm-bindgen")),
                (keyword("version"), string(env!("CARGO_PKG_VERSION"))),
            ]),
        ),
        (keyword("exports"), Form::Vector(exports)),
    ])
    .to_string()
}

fn result_types(value: WasmValueType) -> Vec<ValType> {
    match value {
        WasmValueType::Void => Vec::new(),
        value => vec![val_type(value)],
    }
}

fn val_type(value: WasmValueType) -> ValType {
    match value {
        WasmValueType::I32 => ValType::I32,
        WasmValueType::I64 => ValType::I64,
        WasmValueType::F32 => ValType::F32,
        WasmValueType::F64 => ValType::F64,
        WasmValueType::Void => panic!("void is not a parameter type"),
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn keyword(value: &str) -> Form {
    Form::Keyword(value.to_owned())
}

fn symbol(value: &str) -> Form {
    Form::Symbol(value.to_owned())
}

fn string(value: &str) -> Form {
    Form::String(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm_binding::inspect_direct;

    const ADD: &[u8] =
        b"\0asm\x01\0\0\0\x01\x07\x01\x60\x02\x7e\x7e\x01\x7e\x03\x02\x01\0\x07\x07\x01\x03add\0\0\x0a\x09\x01\x07\0\x20\0\x20\x01\x7c\x0b";
    const START: &[u8] = b"\0asm\x01\0\0\0\x08\x01\0";

    fn interface() -> WasmInterface {
        WasmInterface::parse(
            r#"
            (wasm/interface
             {:schema "hara.wasm-interface/0-alpha"
              :namespace math.scalar
              :module "math.wasm"
              :exports
              {sum {:wasm/export "add"
                    :arguments [{:name left :hara/type :i64 :wasm/type :i64}
                                {:name right :hara/type :i64 :wasm/type :i64}]
                    :returns {:hara/type :i64 :wasm/type :i64}}}})
            "#,
            "fixture",
        )
        .unwrap()
    }

    fn multi_interface() -> WasmInterface {
        WasmInterface::parse(
            r#"
            (wasm/interface
             {:schema "hara.wasm-interface/0-alpha"
              :namespace math.scalar
              :module "math.wasm"
              :exports
              {difference {:wasm/export "sub"
                           :arguments [{:name left :hara/type :i64 :wasm/type :i64}
                                       {:name right :hara/type :i64 :wasm/type :i64}]
                           :returns {:hara/type :i64 :wasm/type :i64}}
               sum {:wasm/export "add"
                    :arguments [{:name left :hara/type :i64 :wasm/type :i64}
                                {:name right :hara/type :i64 :wasm/type :i64}]
                    :returns {:hara/type :i64 :wasm/type :i64}}}})
            "#,
            "fixture",
        )
        .unwrap()
    }

    fn multi_library() -> Vec<u8> {
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.function([ValType::I64, ValType::I64], [ValType::I64]);
        module.section(&types);

        let mut functions = FunctionSection::new();
        functions.function(0);
        functions.function(0);
        module.section(&functions);

        let mut exports = ExportSection::new();
        exports.export("add", ExportKind::Func, 0);
        exports.export("sub", ExportKind::Func, 1);
        module.section(&exports);

        let mut code = wasm_encoder::CodeSection::new();
        for instruction in [Instruction::I64Add, Instruction::I64Sub] {
            let mut function = Function::new([]);
            function.instruction(&Instruction::LocalGet(0));
            function.instruction(&Instruction::LocalGet(1));
            function.instruction(&instruction);
            function.instruction(&Instruction::End);
            code.function(&function);
        }
        module.section(&code);
        module.finish()
    }

    #[test]
    fn adapter_is_deterministic_and_records_all_input_digests() {
        let interface = interface();
        let first = generate_adapter(ADD, &interface).unwrap();
        let second = generate_adapter(ADD, &interface).unwrap();
        assert_eq!(first, second);
        assert!(first.manifest.contains("hara.wasm-adapter/0-alpha"));
        assert!(first.manifest.contains(":module-digest"));
        assert!(first.manifest.contains(":interface-digest"));
        assert!(first.manifest.contains(":adapter-digest"));
    }

    #[test]
    fn adapter_exports_hara_names_and_imports_exact_library_names() {
        let artifact = generate_adapter(ADD, &interface()).unwrap();
        let inspection = inspect_direct(&artifact.bytes).unwrap();
        assert_eq!(inspection.imports[0].module, "hara/library");
        assert_eq!(inspection.imports[0].name, "add");
        assert_eq!(inspection.exports[0].name, "sum");
        assert_eq!(
            inspection.exports[0].signature.arguments,
            vec!["i64", "i64"]
        );
        assert_eq!(inspection.exports[0].signature.returns, "i64");
    }

    #[test]
    fn adapter_forwards_calls_when_composed_with_the_wrapped_library() {
        let artifact = generate_adapter(ADD, &interface()).unwrap();
        let engine = wasmtime::Engine::default();
        let library = wasmtime::Module::new(&engine, ADD).unwrap();
        let adapter = wasmtime::Module::new(&engine, &artifact.bytes).unwrap();
        let mut store = wasmtime::Store::new(&engine, ());
        let library_instance = wasmtime::Instance::new(&mut store, &library, &[]).unwrap();
        let add = library_instance.get_func(&mut store, "add").unwrap();
        let adapter_instance =
            wasmtime::Instance::new(&mut store, &adapter, &[add.into()]).unwrap();
        let sum = adapter_instance
            .get_typed_func::<(i64, i64), i64>(&mut store, "sum")
            .unwrap();

        assert_eq!(sum.call(&mut store, (19, 23)).unwrap(), 42);
    }

    #[test]
    fn multi_export_adapter_forwards_each_import_with_its_declared_signature() {
        let library_bytes = multi_library();
        let artifact = generate_adapter(&library_bytes, &multi_interface()).unwrap();
        let engine = wasmtime::Engine::default();
        let library = wasmtime::Module::new(&engine, &library_bytes).unwrap();
        let adapter = wasmtime::Module::new(&engine, &artifact.bytes).unwrap();
        let mut store = wasmtime::Store::new(&engine, ());
        let library_instance = wasmtime::Instance::new(&mut store, &library, &[]).unwrap();
        let add = library_instance.get_func(&mut store, "add").unwrap();
        let sub = library_instance.get_func(&mut store, "sub").unwrap();
        let adapter_instance =
            wasmtime::Instance::new(&mut store, &adapter, &[sub.into(), add.into()]).unwrap();
        let difference = adapter_instance
            .get_typed_func::<(i64, i64), i64>(&mut store, "difference")
            .unwrap();
        let sum = adapter_instance
            .get_typed_func::<(i64, i64), i64>(&mut store, "sum")
            .unwrap();

        assert_eq!(difference.call(&mut store, (23, 19)).unwrap(), 4);
        assert_eq!(sum.call(&mut store, (19, 23)).unwrap(), 42);
    }

    #[test]
    fn adapter_output_order_is_canonical_for_constructed_interfaces() {
        let mut interface = multi_interface();
        interface.exports.reverse();
        let first = generate_adapter(&multi_library(), &interface).unwrap();
        interface.exports.reverse();
        let second = generate_adapter(&multi_library(), &interface).unwrap();

        assert_eq!(first.bytes, second.bytes);
    }

    #[test]
    fn malformed_or_richer_interfaces_are_rejected_before_generation() {
        let interface = WasmInterface::parse(
            r#"
            {:schema "hara.wasm-interface/0-alpha"
             :namespace codec.echo
             :module "echo.wasm"
             :memory {:export "memory" :allocate "alloc"}
             :exports
             {echo {:wasm/export "echo"
                    :arguments [{:name input :hara/type :bytes :wasm/type :i32
                                 :lower [:pointer :length] :ownership :borrowed}]
                    :returns {:hara/type :bytes :wasm/type :i64
                              :lift :packed-i64 :ownership :callee}}}}
            "#,
            "fixture",
        )
        .unwrap();
        let error = generate_adapter(ADD, &interface).unwrap_err();
        assert!(error.contains("memory requires"));
    }

    #[test]
    fn start_functions_are_rejected_during_static_validation() {
        let error = generate_adapter(START, &interface()).unwrap_err();
        assert!(error.starts_with("wasm-adapter/start-denied"));
    }
}
