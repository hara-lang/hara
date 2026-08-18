#![cfg(target_arch = "wasm32")]

use std::cell::RefCell;
use wasm_bindgen::{JsCast, JsValue};

use crate::core::Value;
use crate::extension::{ExtensionManifest, WasmAbi, WasmExtensionProvider};

pub(crate) struct BrowserWasmProvider {
    module: js_sys::WebAssembly::Module,
    instance: RefCell<Option<js_sys::WebAssembly::Instance>>,
}

impl BrowserWasmProvider {
    pub(crate) fn compile(bytes: &[u8]) -> Result<Self, String> {
        let buffer = js_sys::Uint8Array::from(bytes);
        let module = js_sys::WebAssembly::Module::new(buffer.as_ref())
            .map_err(|error| format!("native/module-invalid: {}", js_error(error)))?;
        if js_sys::WebAssembly::Module::imports(&module).length() != 0 {
            return Err("native/module-import-denied: direct WASM must be import-free".into());
        }
        Ok(Self {
            module,
            instance: RefCell::new(None),
        })
    }
}

impl WasmExtensionProvider for BrowserWasmProvider {
    fn supports(&self, abi: WasmAbi) -> bool {
        abi == WasmAbi::CoreV1
    }

    fn start(&self, manifest: &ExtensionManifest) -> Result<(), String> {
        if !manifest.capabilities.is_empty() {
            return Err("native/capability-denied: direct WASM has no host authority".into());
        }
        let instance = js_sys::WebAssembly::Instance::new(&self.module, &js_sys::Object::new())
            .map_err(|error| format!("native/module-invalid: {}", js_error(error)))?;
        *self.instance.borrow_mut() = Some(instance);
        Ok(())
    }

    fn invoke(
        &self,
        manifest: &ExtensionManifest,
        export: &str,
        arguments: &[Value],
    ) -> Result<Value, String> {
        let instance = self.instance.borrow();
        let exports = instance
            .as_ref()
            .ok_or("native/import-not-started")?
            .exports();
        let function = js_sys::Reflect::get(&exports, &JsValue::from_str(export))
            .map_err(js_error)?
            .dyn_into::<js_sys::Function>()
            .map_err(|_| format!("native/export-missing: {export}"))?;
        let args = js_sys::Array::new();
        let specification = manifest
            .exports
            .iter()
            .find(|(name, _)| name == export)
            .map(|(_, specification)| specification)
            .ok_or_else(|| format!("native/export-missing: {export}"))?;
        for (wire, argument) in specification.arguments.iter().zip(arguments) {
            args.push(&match (wire.as_str(), argument) {
                ("i64", Value::Number(value)) => js_sys::BigInt::from(*value).into(),
                ("i32", Value::Number(value)) if i32::try_from(*value).is_ok() => {
                    JsValue::from_f64(*value as f64)
                }
                ("f32" | "f64", Value::Number(value)) => JsValue::from_f64(*value as f64),
                ("f32" | "f64", Value::Float(value)) => JsValue::from_f64(*value),
                _ => return Err(format!("native/type-error: {export} expects {wire}")),
            });
        }
        let result = function
            .apply(&JsValue::UNDEFINED, &args)
            .map_err(|error| format!("native/invoke-failed: {export} ({})", js_error(error)))?;
        match specification.returns.as_str() {
            "void" if result.is_undefined() => Ok(Value::Nil),
            "i64" if result.is_bigint() => i64::try_from(result.unchecked_into::<js_sys::BigInt>())
                .map(Value::Number)
                .map_err(|_| format!("native/result-out-of-range: {export}")),
            "i32" => result
                .as_f64()
                .map(|value| Value::Number(value as i32 as i64))
                .ok_or_else(|| format!("native/result-type-invalid: {export}")),
            "f32" | "f64" => result
                .as_f64()
                .map(Value::Float)
                .ok_or_else(|| format!("native/result-type-invalid: {export}")),
            _ => Err(format!("native/result-type-invalid: {export}")),
        }
    }

    fn cancel(&self, _manifest: &ExtensionManifest, _request: u64) -> Result<(), String> {
        Err("native/cancel-unsupported: core.v1 calls are synchronous".into())
    }

    fn shutdown(&self, _manifest: &ExtensionManifest) {
        self.instance.borrow_mut().take();
    }
}

fn js_error(value: JsValue) -> String {
    value
        .as_string()
        .or_else(|| {
            js_sys::Reflect::get(&value, &JsValue::from_str("message"))
                .ok()
                .and_then(|value| value.as_string())
        })
        .unwrap_or_else(|| format!("{value:?}"))
}
