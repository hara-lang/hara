#![cfg(not(target_arch = "wasm32"))]

use std::cell::RefCell;

use wasmtime::{Config, Engine, Instance, Module, Store, StoreLimits, StoreLimitsBuilder, Val};

use crate::core::Value;
use crate::extension::{ExtensionExport, ExtensionManifest, WasmAbi, WasmExtensionProvider};
use crate::wasm_binding::{MemoryBindingPlan, WasmtimeMemoryExecutor};

struct Session {
    store: Store<StoreLimits>,
    instance: Instance,
}

/// Process-shareable compiled code. Hosts can store one of these per artifact
/// digest and creates a fresh provider/store for every session that loads it.
#[derive(Clone)]
pub struct CompiledWasmModule {
    engine: Engine,
    module: Module,
    exports: Vec<(String, ExtensionExport)>,
}

impl CompiledWasmModule {
    pub fn compile(bytes: &[u8]) -> Result<Self, String> {
        let exports = crate::direct_wasm::exports(bytes)?;
        let mut config = Config::new();
        config.consume_fuel(true);
        let engine = Engine::new(&config)
            .map_err(|error| format!("extension/engine-unavailable: {error}"))?;
        let module = Module::new(&engine, bytes)
            .map_err(|error| format!("extension/module-invalid: {error}"))?;
        if module.imports().next().is_some() {
            return Err("extension/module-invalid: extension modules must be import-free".into());
        }
        Ok(Self {
            engine,
            module,
            exports,
        })
    }

    pub fn provider(&self) -> WasmtimeExtensionProvider {
        WasmtimeExtensionProvider {
            mode: ProviderMode::Direct {
                engine: self.engine.clone(),
                module: self.module.clone(),
                session: RefCell::new(None),
            },
        }
    }

    pub fn direct_exports(&self) -> Result<Vec<(String, ExtensionExport)>, String> {
        Ok(self.exports.clone())
    }
}

/// Import-free Wasmtime host for the direct scalar core.v1 ABI.
pub struct WasmtimeExtensionProvider {
    mode: ProviderMode,
}

enum ProviderMode {
    Direct {
        engine: Engine,
        module: Module,
        session: RefCell<Option<Session>>,
    },
    Memory(WasmtimeMemoryExecutor),
}

impl WasmtimeExtensionProvider {
    pub fn compile(bytes: &[u8]) -> Result<Self, String> {
        Ok(CompiledWasmModule::compile(bytes)?.provider())
    }

    pub fn compile_memory(
        bytes: &[u8],
        plan: MemoryBindingPlan,
    ) -> Result<Self, String> {
        Ok(Self {
            mode: ProviderMode::Memory(WasmtimeMemoryExecutor::compile(bytes, plan)?),
        })
    }
}

impl WasmExtensionProvider for WasmtimeExtensionProvider {
    fn supports(&self, abi: WasmAbi) -> bool {
        matches!(
            (&self.mode, abi),
            (ProviderMode::Direct { .. }, WasmAbi::CoreV1)
                | (ProviderMode::Memory(_), WasmAbi::MemoryV1)
        )
    }

    fn start(&self, manifest: &ExtensionManifest) -> Result<(), String> {
        if !manifest.capabilities.is_empty() {
            return Err(format!(
                "extension/capability-denied: {:?} for {}",
                manifest.capabilities, manifest.namespace
            ));
        }
        if let ProviderMode::Memory(executor) = &self.mode {
            let plan = executor.plan();
            if manifest.exports.len() != plan.functions.len()
                || manifest.exports.iter().any(|(name, specification)| {
                    plan.functions
                        .iter()
                        .find(|function| function.name == *name)
                        .map_or(true, |function| {
                            specification.raw_name(name) != function.wasm_export
                        })
                })
            {
                return Err(format!(
                    "extension/manifest-mismatch: memory.v1 exports for {} do not match bindings.edn",
                    manifest.namespace
                ));
            }
            return Ok(());
        }
        let ProviderMode::Direct {
            engine,
            module,
            session,
        } = &self.mode
        else {
            unreachable!()
        };
        let limits = StoreLimitsBuilder::new()
            .memory_size(64 * 1024 * 1024)
            .instances(1)
            .memories(1)
            .tables(1)
            .build();
        let mut store = Store::new(engine, limits);
        store.limiter(|limits| limits);
        let instance = Instance::new(&mut store, module, &[])
            .map_err(|error| format!("extension/module-invalid: {error}"))?;
        for (name, specification) in &manifest.exports {
            let raw_name = specification.raw_name(name);
            let function = instance.get_func(&mut store, raw_name).ok_or_else(|| {
                format!(
                    "extension/malformed: module has no export {raw_name} for public name {name}"
                )
            })?;
            if function.ty(&store).results().len() > 1 {
                return Err(format!(
                    "extension/abi-type-unsupported: {name} has multiple results"
                ));
            }
        }
        *session.borrow_mut() = Some(Session { store, instance });
        Ok(())
    }

    fn invoke(
        &self,
        manifest: &ExtensionManifest,
        export: &str,
        arguments: &[Value],
    ) -> Result<Value, String> {
        if let ProviderMode::Memory(executor) = &self.mode {
            return executor.invoke(export, arguments);
        }
        let ProviderMode::Direct {
            session,
            ..
        } = &self.mode
        else {
            unreachable!()
        };
        let specification = manifest
            .exports
            .iter()
            .find(|(name, _)| name == export)
            .map(|(_, specification)| specification)
            .ok_or_else(|| format!("extension/export-missing: {export}"))?;
        let raw_name = specification.raw_name(export);
        let mut session = session.borrow_mut();
        let session = session
            .as_mut()
            .ok_or_else(|| format!("extension/not-started: {}", manifest.namespace))?;
        let function = session
            .instance
            .get_func(&mut session.store, raw_name)
            .ok_or_else(|| format!("extension/export-missing: {export} -> {raw_name}"))?;
        let values = specification
            .arguments
            .iter()
            .zip(arguments)
            .map(|(wire_type, value)| argument(export, wire_type, value))
            .collect::<Result<Vec<_>, _>>()?;
        let mut results = if specification.returns == "void" {
            Vec::new()
        } else {
            vec![default_result(&specification.returns)?]
        };
        session
            .store
            .set_fuel(10_000_000)
            .map_err(|error| format!("extension/execution-limit: {error}"))?;
        function
            .call(&mut session.store, &values, &mut results)
            .map_err(|error| {
                format!(
                    "extension/invoke-failed: {}/{} ({error})",
                    manifest.namespace, export
                )
            })?;
        result(export, &specification.returns, results.into_iter().next())
    }

    fn cancel(&self, _manifest: &ExtensionManifest, _request: u64) -> Result<(), String> {
        Err("extension/cancel-unsupported: core.v1 calls are synchronous".into())
    }

    fn shutdown(&self, _manifest: &ExtensionManifest) {
        if let ProviderMode::Direct { session, .. } = &self.mode {
            session.borrow_mut().take();
        }
    }
}

fn argument(export: &str, wire_type: &str, value: &Value) -> Result<Val, String> {
    let type_error = || format!("extension/type-error: {export} expects {wire_type}");
    match (wire_type, value) {
        ("i32", Value::Number(value)) => i32::try_from(*value)
            .map(Val::I32)
            .map_err(|_| type_error()),
        ("i64", Value::Number(value)) => Ok(Val::I64(*value)),
        ("f32", Value::Float(value)) => Ok(Val::F32((*value as f32).to_bits())),
        ("f32", Value::Number(value)) => Ok(Val::F32((*value as f32).to_bits())),
        ("f64", Value::Float(value)) => Ok(Val::F64(value.to_bits())),
        ("f64", Value::Number(value)) => Ok(Val::F64((*value as f64).to_bits())),
        ("boolean", Value::Bool(value)) => Ok(Val::I32(i32::from(*value))),
        _ => Err(type_error()),
    }
}

fn default_result(wire_type: &str) -> Result<Val, String> {
    match wire_type {
        "i32" | "boolean" => Ok(Val::I32(0)),
        "i64" => Ok(Val::I64(0)),
        "f32" => Ok(Val::F32(0)),
        "f64" => Ok(Val::F64(0)),
        _ => Err(format!("extension/abi-type-unsupported: {wire_type}")),
    }
}

fn result(export: &str, wire_type: &str, value: Option<Val>) -> Result<Value, String> {
    match (wire_type, value) {
        ("void", None) => Ok(Value::Nil),
        ("i32", Some(Val::I32(value))) => Ok(Value::Number(i64::from(value))),
        ("i64", Some(Val::I64(value))) => Ok(Value::Number(value)),
        ("f32", Some(Val::F32(value))) => Ok(Value::Float(f32::from_bits(value) as f64)),
        ("f64", Some(Val::F64(value))) => Ok(Value::Float(f64::from_bits(value))),
        ("boolean", Some(Val::I32(value))) => Ok(Value::Bool(value != 0)),
        _ => Err(format!(
            "extension/abi-type-unsupported: {export} -> {wire_type}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::extension::{ExtensionManifest, Value, WasmExtension};

    use super::WasmtimeExtensionProvider;

    const ADD: &[u8] = b"\0asm\x01\0\0\0\x01\x07\x01\x60\x02\x7e\x7e\x01\x7e\x03\x02\x01\0\x07\x07\x01\x03add\0\0\x0a\x09\x01\x07\0\x20\0\x20\x01\x7c\x0b";
    const ALIASED_MANIFEST: &str = r#"
      {:namespace "math.scalar"
       :version "0.1.0"
       :provider :wasm
       :module "math.wasm"
       :abi :core.v1
       :exports {"sum" {:wasm/export "add"
                         :args [:i64 :i64]
                         :returns :i64}}
       :capabilities []}"#;

    #[test]
    fn invokes_a_raw_wasm_export_through_a_public_hara_name() {
        let manifest = ExtensionManifest::parse(ALIASED_MANIFEST, "fixture").unwrap();
        let provider = WasmtimeExtensionProvider::compile(ADD).unwrap();
        let mut extension = WasmExtension::new(manifest, provider).unwrap();
        let bindings = extension.require().unwrap();
        assert_eq!(bindings[0].name, "sum");
        assert_eq!(
            bindings[0]
                .invoke(&[Value::Number(19), Value::Number(23)])
                .unwrap(),
            Value::Number(42)
        );
    }
}
