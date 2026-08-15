/// Builds the fully bootstrapped namespace registry used by native embedding hosts.
///
/// Hosts receive the same Foundation Vars, primitive values and protocol wiring
/// as a normal Hara runtime without depending on crate-private bootstrap helpers.
pub fn embedding_namespace_registry() -> kernel::NamespaceRegistry<core::Value> {
    Runtime::new().namespace_registry.clone()
}
use wasm_bindgen::prelude::*;

include!(concat!(env!("OUT_DIR"), "/embedded_hal.rs"));

const EAGER_HAL_RESOURCES: &[&str] = &[
    "std.foundation.string",
    "std.foundation.promise",
    "std.foundation.bytes",
    "std.foundation.coroutine",
    "std.foundation.pretty",
];

fn ignore_socket_event(_event: core::SocketEvent) {}

#[wasm_bindgen(start)]
pub fn init_wasm() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct PromiseHandle {
    promise: core::Promise,
}

#[wasm_bindgen]
impl PromiseHandle {
    fn from_promise(promise: core::Promise) -> PromiseHandle {
        PromiseHandle { promise }
    }

    #[wasm_bindgen(constructor)]
    pub fn new() -> PromiseHandle {
        PromiseHandle {
            promise: core::Promise::new(),
        }
    }

    pub fn state(&self) -> String {
        match self.promise.state() {
            core::PromiseState::Pending => "pending".into(),
            core::PromiseState::Fulfilled(_) => "fulfilled".into(),
            core::PromiseState::Rejected(_) => "rejected".into(),
        }
    }

    pub fn resolve(&self, value: &str) -> bool {
        self.promise.resolve(core::Value::String(value.into()))
    }

    pub fn reject(&self, error: &str) -> bool {
        self.promise.reject(error)
    }

    pub fn adopt(&self, other: &PromiseHandle) -> bool {
        self.promise.adopt(&other.promise)
    }

    pub fn value(&self) -> Result<String, JsValue> {
        match self.promise.state() {
            core::PromiseState::Pending => Err(JsValue::from_str("promise is pending")),
            core::PromiseState::Fulfilled(value) => Ok(value.display()),
            core::PromiseState::Rejected(error) => Err(JsValue::from_str(&error.message())),
        }
    }
}

#[wasm_bindgen]
pub struct Runtime {
    env: HashMap<String, core::Value>,
    test_runner: String,
    protocols: core::ProtocolRegistry,
    extensions: core::ExtensionRegistry,
    wasm_extensions: HashMap<String, extension::WasmExtension>,
    providers: core::ProviderRegistry,
    package_catalog: core::PackageCatalog,
    resources: HashMap<String, String>,
    resource_overrides: HashSet<String>,
    #[cfg(feature = "bytecode-vm")]
    bytecode_resources: HashMap<String, (String, Vec<u8>)>,
    loaded_resources: HashSet<String>,
    halc_schema_definitions: HashMap<String, Form>,
    halc_function_schemas: HashMap<String, Form>,
    halc_schema_types: HashMap<String, kernel::SchemaType>,
    halc_function_types: HashMap<String, kernel::SchemaType>,
    halc_inferred_function_types: HashMap<String, kernel::SchemaType>,
    namespace_registry: kernel::NamespaceRegistry<core::Value>,
    macros: Rc<RefCell<HashMap<(String, String), Rc<core::Function>>>>,
    generated_configs: HashMap<String, kernel::GeneratedNamespaceConfig>,
    #[cfg(feature = "evaluation-journal")]
    next_journal_id: u64,
    #[cfg(target_arch = "wasm32")]
    host_handler: Option<js_sys::Function>,
    #[cfg(not(target_arch = "wasm32"))]
    native_host_handler:
        Option<Rc<dyn Fn(String, String, Vec<core::Value>) -> Result<core::Value, String>>>,
    #[cfg(not(target_arch = "wasm32"))]
    native_modules: native_module::Registry,
    #[cfg(not(target_arch = "wasm32"))]
    extension_roots: Vec<std::path::PathBuf>,
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // Namespace vars and the flattened environment retain native export
        // closures, and those closures retain the extension session. Release
        // the bindings before dropping the session owners so provider
        // shutdown is deterministic at the Runtime boundary.
        let namespaces = self.wasm_extensions.keys().cloned().collect::<Vec<_>>();
        for namespace in self.namespace_registry.all() {
            for (symbol, var) in namespace.mappings() {
                if var
                    .symbol()
                    .get_namespace()
                    .is_some_and(|owner| namespaces.iter().any(|extension| extension == owner))
                {
                    namespace.unmap(&symbol);
                }
            }
        }
        self.env.clear();
        self.wasm_extensions.clear();
    }
}


