#![cfg(not(target_arch = "wasm32"))]

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{mpsc, Arc};

use crate::core::{Promise, Value};
use crate::invoke_hta::InvokeHtaError;
use crate::lang::data::Symbol;
use crate::{
    InProcessSandboxProvider, Runtime, SandboxId, SandboxSpec, SandboxStatus, SessionId,
    SessionKernel,
};

mod arguments;
mod documentation;
use arguments::{
    boolean as boolean_argument, keyword, optional_string as optional_string_argument,
    string as string_argument, strings as strings_argument, strings_value, tap_value,
};
pub use documentation::{Documentation, DocumentationValue};

// Optimized brokers stay within the production 8 MiB ceiling. Debug evaluator
// frames are much larger and need the same development allowance as the CLI
// and portable test runner while loading the full language library.
const RUNTIME_BROKER_STACK_SIZE: usize = if cfg!(debug_assertions) {
    64 * 1024 * 1024
} else {
    8 * 1024 * 1024
};

#[derive(Clone, Copy)]
enum RuntimeBootstrap {
    Full,
    Core,
}

enum Request {
    Eval {
        session: String,
        source: String,
        reply: mpsc::Sender<Result<String, String>>,
    },
    Namespace {
        session: String,
        reply: mpsc::Sender<Result<String, String>>,
    },
    Complete {
        session: String,
        prefix: String,
        reply: mpsc::Sender<Result<Vec<String>, String>>,
    },
    Doc {
        session: String,
        symbol: String,
        reply: mpsc::Sender<Result<Documentation, String>>,
    },
    Create {
        session: String,
        reply: mpsc::Sender<Result<String, String>>,
    },
    Close {
        session: String,
        reply: mpsc::Sender<Result<String, String>>,
    },
    List {
        reply: mpsc::Sender<Result<Vec<String>, String>>,
    },
    Info {
        session: String,
        reply: mpsc::Sender<Result<String, String>>,
    },
    RegisterResource {
        name: String,
        source: String,
        reply: mpsc::Sender<Result<(), String>>,
    },
    RemoveResource {
        name: String,
        reply: mpsc::Sender<Result<(), String>>,
    },
    ListResources {
        reply: mpsc::Sender<Result<Vec<String>, String>>,
    },
    InstallModule {
        session: String,
        manifest: String,
        module: crate::wasmtime_provider::CompiledWasmModule,
        reply: mpsc::Sender<Result<String, String>>,
    },
    InvokeModule {
        session: String,
        namespace: String,
        export: String,
        arguments: Vec<u8>,
        reply: mpsc::Sender<Result<Vec<u8>, String>>,
    },
    InvokeHta {
        session: String,
        qualified_var: String,
        arguments: Vec<u8>,
        reply: mpsc::Sender<Result<Vec<u8>, InvokeHtaError>>,
    },
    SandboxOpen {
        spec: SandboxSpec,
        reply: mpsc::Sender<Result<SandboxId, String>>,
    },
    SandboxEval {
        sandbox: SandboxId,
        source: String,
        reply: mpsc::Sender<Result<String, String>>,
    },
    SandboxCall {
        sandbox: SandboxId,
        callable: String,
        arguments: Vec<u8>,
        reply: mpsc::Sender<Result<Vec<u8>, String>>,
    },
    SandboxCancel {
        sandbox: SandboxId,
        reply: mpsc::Sender<Result<bool, String>>,
    },
    SandboxStatus {
        sandbox: SandboxId,
        reply: mpsc::Sender<Result<SandboxStatus, String>>,
    },
    SandboxClose {
        sandbox: SandboxId,
        reply: mpsc::Sender<Result<(), String>>,
    },
    Shutdown,
}

struct BrokerHandle {
    sender: mpsc::Sender<Request>,
}

impl Drop for BrokerHandle {
    fn drop(&mut self) {
        let _ = self.sender.send(Request::Shutdown);
    }
}

#[derive(Clone)]
pub struct RuntimeBroker {
    handle: Arc<BrokerHandle>,
}

impl RuntimeBroker {
    pub fn start() -> Result<Self, String> {
        Self::start_with_bootstrap(None, false, false, false, RuntimeBootstrap::Full)
    }

    /// Starts an isolated broker with the portable L0 runtime.
    ///
    /// This is intended for small embedding surfaces and focused tests
    /// that do not require the language-level Foundation bundle.
    pub fn start_core() -> Result<Self, String> {
        Self::start_with_bootstrap(None, false, false, false, RuntimeBootstrap::Core)
    }

    pub fn start_with(
        root: Option<PathBuf>,
        native_sockets: bool,
        allow_process: bool,
        allow_postgres: bool,
    ) -> Result<Self, String> {
        Self::start_with_bootstrap(
            root,
            native_sockets,
            allow_process,
            allow_postgres,
            RuntimeBootstrap::Full,
        )
    }

    fn start_with_bootstrap(
        root: Option<PathBuf>,
        native_sockets: bool,
        allow_process: bool,
        allow_postgres: bool,
        bootstrap: RuntimeBootstrap,
    ) -> Result<Self, String> {
        let (sender, receiver) = mpsc::channel();
        std::thread::Builder::new()
            .name("hara-runtime-broker".into())
            .stack_size(RUNTIME_BROKER_STACK_SIZE)
            .spawn(move || {
                run(
                    receiver,
                    root,
                    native_sockets,
                    allow_process,
                    allow_postgres,
                    bootstrap,
                )
            })
            .map_err(|error| format!("runtime broker failed: {error}"))?;
        Ok(Self {
            handle: Arc::new(BrokerHandle { sender }),
        })
    }

    pub fn eval(&self, session: &str, source: &str) -> Result<String, String> {
        self.call(|reply| Request::Eval {
            session: session.into(),
            source: source.into(),
            reply,
        })
    }

    pub fn namespace(&self, session: &str) -> Result<String, String> {
        self.call(|reply| Request::Namespace {
            session: session.into(),
            reply,
        })
    }

    pub fn complete(&self, session: &str, prefix: &str) -> Result<Vec<String>, String> {
        self.call(|reply| Request::Complete {
            session: session.into(),
            prefix: prefix.into(),
            reply,
        })
    }

    pub fn documentation(&self, session: &str, symbol: &str) -> Result<Documentation, String> {
        self.call(|reply| Request::Doc {
            session: session.into(),
            symbol: symbol.into(),
            reply,
        })
    }

    pub fn create(&self, session: &str) -> Result<String, String> {
        self.call(|reply| Request::Create {
            session: session.into(),
            reply,
        })
    }

    pub fn close(&self, session: &str) -> Result<String, String> {
        self.call(|reply| Request::Close {
            session: session.into(),
            reply,
        })
    }

    pub fn list(&self) -> Result<Vec<String>, String> {
        self.call(|reply| Request::List { reply })
    }

    pub fn info(&self, session: &str) -> Result<String, String> {
        self.call(|reply| Request::Info {
            session: session.into(),
            reply,
        })
    }

    pub fn register_resource(&self, name: &str, source: &str) -> Result<(), String> {
        self.call(|reply| Request::RegisterResource {
            name: name.into(),
            source: source.into(),
            reply,
        })
    }

    pub fn remove_resource(&self, name: &str) -> Result<(), String> {
        self.call(|reply| Request::RemoveResource {
            name: name.into(),
            reply,
        })
    }

    pub fn resources(&self) -> Result<Vec<String>, String> {
        self.call(|reply| Request::ListResources { reply })
    }

    pub fn install_module(
        &self,
        session: &str,
        manifest: &str,
        module: &crate::wasmtime_provider::CompiledWasmModule,
    ) -> Result<String, String> {
        self.call(|reply| Request::InstallModule {
            session: session.into(),
            manifest: manifest.into(),
            module: module.clone(),
            reply,
        })
    }

    pub fn invoke_hta(
        &self,
        session: &str,
        qualified_var: &str,
        arguments: &[u8],
    ) -> Result<Vec<u8>, InvokeHtaError> {
        let (reply, response) = mpsc::channel();
        self.handle
            .sender
            .send(Request::InvokeHta {
                session: session.into(),
                qualified_var: qualified_var.into(),
                arguments: arguments.into(),
                reply,
            })
            .map_err(|_| InvokeHtaError::BrokerClosed)?;
        response.recv().map_err(|_| InvokeHtaError::BrokerStopped)?
    }

    pub fn invoke_module(
        &self,
        session: &str,
        namespace: &str,
        export: &str,
        arguments: &[u8],
    ) -> Result<Vec<u8>, String> {
        self.call(|reply| Request::InvokeModule {
            session: session.into(),
            namespace: namespace.into(),
            export: export.into(),
            arguments: arguments.into(),
            reply,
        })
    }

    fn sandbox_open(&self, spec: SandboxSpec) -> Result<SandboxId, String> {
        self.call(|reply| Request::SandboxOpen { spec, reply })
    }

    fn sandbox_eval_receiver(
        &self,
        sandbox: SandboxId,
        source: &str,
    ) -> Result<mpsc::Receiver<Result<String, String>>, String> {
        let (reply, response) = mpsc::channel();
        self.handle
            .sender
            .send(Request::SandboxEval {
                sandbox,
                source: source.into(),
                reply,
            })
            .map_err(|_| "runtime broker is closed".to_owned())?;
        Ok(response)
    }

    fn sandbox_call_receiver(
        &self,
        sandbox: SandboxId,
        callable: &str,
        arguments: &[u8],
    ) -> Result<mpsc::Receiver<Result<Vec<u8>, String>>, String> {
        let (reply, response) = mpsc::channel();
        self.handle
            .sender
            .send(Request::SandboxCall {
                sandbox,
                callable: callable.into(),
                arguments: arguments.into(),
                reply,
            })
            .map_err(|_| "runtime broker is closed".to_owned())?;
        Ok(response)
    }

    fn sandbox_cancel(&self, sandbox: SandboxId) -> Result<bool, String> {
        self.call(|reply| Request::SandboxCancel { sandbox, reply })
    }

    fn sandbox_status(&self, sandbox: SandboxId) -> Result<SandboxStatus, String> {
        self.call(|reply| Request::SandboxStatus { sandbox, reply })
    }

    fn sandbox_close(&self, sandbox: SandboxId) -> Result<(), String> {
        self.call(|reply| Request::SandboxClose { sandbox, reply })
    }

    fn call<T>(
        &self,
        request: impl FnOnce(mpsc::Sender<Result<T, String>>) -> Request,
    ) -> Result<T, String> {
        let (reply, response) = mpsc::channel();
        self.handle
            .sender
            .send(request(reply))
            .map_err(|_| "runtime broker is closed".to_owned())?;
        response
            .recv()
            .map_err(|_| "runtime broker stopped without a response".to_owned())?
    }
}

fn runtime(
    root: Option<&PathBuf>,
    native_sockets: bool,
    allow_process: bool,
    allow_postgres: bool,
    bootstrap: RuntimeBootstrap,
) -> Runtime {
    let mut runtime = match bootstrap {
        RuntimeBootstrap::Full => Runtime::new(),
        RuntimeBootstrap::Core => Runtime::core(),
    };
    if let Some(root) = root {
        runtime.install_native_file_provider(root.to_string_lossy().as_ref());
    }
    if native_sockets {
        runtime.install_native_socket_provider();
    }
    if allow_process {
        runtime.install_native_process_provider();
    }
    if allow_postgres {
        runtime
            .install_native_module(hara_db_postgres::module())
            .expect("db.postgres native module must install once per runtime");
    }
    runtime
}

fn run(
    receiver: mpsc::Receiver<Request>,
    root: Option<PathBuf>,
    native_sockets: bool,
    allow_process: bool,
    allow_postgres: bool,
    bootstrap: RuntimeBootstrap,
) {
    let runtime_root = root.clone();
    let runtime_factory: Rc<dyn Fn() -> Runtime> = Rc::new(move || {
        runtime(
            runtime_root.as_ref(),
            native_sockets,
            allow_process,
            allow_postgres,
            bootstrap,
        )
    });
    let root_runtime = runtime_factory();
    let mut kernel = SessionKernel::with_runtime_factory(root_runtime, runtime_factory);
    kernel.register_sandbox_provider(Rc::new(InProcessSandboxProvider));
    while let Ok(request) = receiver.recv() {
        match request {
            Request::Eval {
                session,
                source,
                reply,
            } => {
                let result = broker_session_id(&session).and_then(|id| {
                    kernel
                        .session_mut(&id)?
                        .runtime_mut()?
                        .eval_native_traced(&source)
                });
                let _ = reply.send(result);
            }
            Request::Namespace { session, reply } => {
                let result =
                    broker_session_id(&session).and_then(|id| kernel.session_namespace(&id));
                let _ = reply.send(result);
            }
            Request::Complete {
                session,
                prefix,
                reply,
            } => {
                let result = broker_session_id(&session).and_then(|id| {
                    kernel.session(&id)?.runtime().map(|runtime| {
                        let mut symbols = runtime
                            .visible_symbols()
                            .into_iter()
                            .filter(|symbol| symbol.starts_with(&prefix))
                            .collect::<Vec<_>>();
                        symbols.sort();
                        symbols.dedup();
                        symbols
                    })
                });
                let _ = reply.send(result);
            }
            Request::Doc {
                session,
                symbol,
                reply,
            } => {
                let result = broker_session_id(&session)
                    .and_then(|id| documentation(kernel.session(&id)?.runtime()?, &symbol));
                let _ = reply.send(result);
            }
            Request::Create { session, reply } => {
                let result = SessionId::parse(&session)
                    .map_err(|_| format!("Session already exists or is invalid: {session}"))
                    .and_then(|id| {
                        kernel
                            .create_session(id)
                            .map_err(|_| format!("Session already exists or is invalid: {session}"))
                    })
                    .map(|_| session);
                let _ = reply.send(result);
            }
            Request::Close { session, reply } => {
                let result = broker_session_id(&session)
                    .and_then(|id| kernel.close_session(&id))
                    .map(|_| session)
                    .map_err(|error| match error.as_str() {
                        "ROOT_CANNOT_CLOSE" => "ROOT cannot be closed".into(),
                        _ if error.starts_with("NO_SESSION ") => {
                            format!("No session: {}", error.trim_start_matches("NO_SESSION "))
                        }
                        _ => error,
                    });
                let _ = reply.send(result);
            }
            Request::List { reply } => {
                let names = kernel
                    .session_names()
                    .into_iter()
                    .map(|id| id.to_string())
                    .collect();
                let _ = reply.send(Ok(names));
            }
            Request::Info { session, reply } => {
                let result = broker_session_id(&session)
                    .and_then(|id| kernel.session_namespace(&id))
                    .map(|namespace| format!("{session} {namespace}"));
                let _ = reply.send(result);
            }
            Request::RegisterResource {
                name,
                source,
                reply,
            } => {
                kernel.register_resource(&name, &source);
                let _ = reply.send(Ok(()));
            }
            Request::RemoveResource { name, reply } => {
                kernel.remove_resource(&name);
                let _ = reply.send(Ok(()));
            }
            Request::ListResources { reply } => {
                let _ = reply.send(Ok(kernel.resource_names()));
            }
            Request::InstallModule {
                session,
                manifest,
                module,
                reply,
            } => {
                let result = broker_session_id(&session).and_then(|id| {
                    let runtime = kernel.session_mut(&id)?.runtime_mut()?;
                    let provider = module.provider();
                    let parsed =
                        crate::extension::ExtensionManifest::parse(&manifest, "MODULE PUT")?;
                    let namespace = parsed.namespace.clone();
                    runtime.install_wasm_extension(&manifest, "MODULE PUT", provider)?;
                    Ok(namespace)
                });
                let _ = reply.send(result);
            }
            Request::InvokeModule {
                session,
                namespace,
                export,
                arguments,
                reply,
            } => {
                let result = broker_session_id(&session).and_then(|id| {
                    let runtime = kernel.session_mut(&id)?.runtime_mut()?;
                    let arguments = crate::hta::decode(&arguments)?;
                    let arguments: Vec<crate::extension::Value> = match arguments {
                        crate::extension::Value::Vector(values) => values.iter().cloned().collect(),
                        crate::extension::Value::Tuple(values) => values.iter().cloned().collect(),
                        other => {
                            return Err(format!(
                                "hta/arguments: expected vector, got {}",
                                other.display()
                            ))
                        }
                    };
                    let result = runtime.invoke_wasm_extension(&namespace, &export, &arguments)?;
                    crate::hta::encode(&result)
                });
                let _ = reply.send(result);
            }
            Request::InvokeHta {
                session,
                qualified_var,
                arguments,
                reply,
            } => {
                let result = SessionId::parse(&session)
                    .map_err(|_| InvokeHtaError::SessionMissing(session.clone()))
                    .and_then(|id| {
                        kernel
                            .session_mut(&id)
                            .map_err(|_| InvokeHtaError::SessionMissing(session.clone()))?
                            .runtime_mut()
                            .map_err(InvokeHtaError::Execution)?
                            .invoke_hta(&qualified_var, &arguments)
                    });
                let _ = reply.send(result);
            }
            Request::SandboxOpen { spec, reply } => {
                let _ = reply.send(kernel.open_sandbox(spec).map_err(|error| error.to_string()));
            }
            Request::SandboxEval {
                sandbox,
                source,
                reply,
            } => match kernel.sandbox_eval(sandbox, &source) {
                Ok(pending) => {
                    std::thread::spawn(move || {
                        let _ = reply.send(pending.wait().map_err(|error| error.to_string()));
                    });
                }
                Err(error) => {
                    let _ = reply.send(Err(error.to_string()));
                }
            },
            Request::SandboxCall {
                sandbox,
                callable,
                arguments,
                reply,
            } => match kernel.sandbox_call(sandbox, &callable, &arguments) {
                Ok(pending) => {
                    std::thread::spawn(move || {
                        let _ = reply.send(pending.wait().map_err(|error| error.to_string()));
                    });
                }
                Err(error) => {
                    let _ = reply.send(Err(error.to_string()));
                }
            },
            Request::SandboxCancel { sandbox, reply } => {
                let _ = reply.send(
                    kernel
                        .cancel_sandbox(sandbox)
                        .map_err(|error| error.to_string()),
                );
            }
            Request::SandboxStatus { sandbox, reply } => {
                let _ = reply.send(
                    kernel
                        .sandbox_status(sandbox)
                        .map_err(|error| error.to_string()),
                );
            }
            Request::SandboxClose { sandbox, reply } => {
                let _ = reply.send(
                    kernel
                        .close_sandbox(sandbox)
                        .map_err(|error| error.to_string()),
                );
            }
            Request::Shutdown => break,
        }
    }
}

fn broker_session_id(session: &str) -> Result<SessionId, String> {
    SessionId::parse(session).map_err(|_| format!("No session: {session}"))
}

fn documentation(runtime: &Runtime, symbol: &str) -> Result<Documentation, String> {
    documentation::lookup(runtime, symbol)
}

/// Installs the generic native driver behind `std.native.Kernel/*`.
/// Command policy remains in Hara; this adapter only multiplexes isolated
/// evaluator sessions and transfers portable values across the boundary.
pub fn install_native_kernel(runtime: &mut Runtime, broker: RuntimeBroker) {
    runtime.install_native_kernel_provider(Rc::new(move |operation, arguments| {
        kernel_call(&broker, &operation, &arguments)
    }));
}

fn kernel_call(
    broker: &RuntimeBroker,
    operation: &str,
    arguments: &[Value],
) -> Result<Value, String> {
    match operation {
        "sandbox-open" => {
            let spec = sandbox_spec_argument(arguments, operation)?;
            let promise = Promise::new();
            match broker.sandbox_open(spec) {
                Ok(id) => {
                    promise.resolve(Value::Number(id.get() as i64));
                }
                Err(error) => {
                    promise.reject(error);
                }
            }
            Ok(Value::Promise(promise))
        }
        "sandbox-eval" => {
            let sandbox = sandbox_id_argument(arguments, 0, operation)?;
            let source = string_argument(arguments, 1, operation)?;
            let receiver = broker.sandbox_eval_receiver(sandbox, source)?;
            Ok(Value::Promise(sandbox_string_promise(
                broker.clone(),
                sandbox,
                receiver,
            )))
        }
        "sandbox-call" => {
            let sandbox = sandbox_id_argument(arguments, 0, operation)?;
            let callable = string_argument(arguments, 1, operation)?;
            let supplied = arguments
                .get(2)
                .ok_or_else(|| format!("{operation}: missing argument vector"))?;
            let normalized = match supplied {
                Value::Vector(_) => supplied.clone(),
                Value::Tuple(values) => Value::Vector(values.iter().cloned().collect()),
                _ => return Err(format!("{operation}: expected an argument vector")),
            };
            let encoded = crate::hta::encode(&normalized)?;
            let receiver = broker.sandbox_call_receiver(sandbox, callable, &encoded)?;
            Ok(Value::Promise(sandbox_hta_promise(
                broker.clone(),
                sandbox,
                receiver,
            )))
        }
        "sandbox-cancel" => {
            let promise = Promise::new();
            match broker.sandbox_cancel(sandbox_id_argument(arguments, 0, operation)?) {
                Ok(cancelled) => {
                    promise.resolve(Value::Bool(cancelled));
                }
                Err(error) => {
                    promise.reject(error);
                }
            }
            Ok(Value::Promise(promise))
        }
        "sandbox-status" => broker
            .sandbox_status(sandbox_id_argument(arguments, 0, operation)?)
            .map(sandbox_status_value),
        "sandbox-close" => {
            let promise = Promise::new();
            match broker.sandbox_close(sandbox_id_argument(arguments, 0, operation)?) {
                Ok(()) => {
                    promise.resolve(Value::Nil);
                }
                Err(error) => {
                    promise.reject(error);
                }
            }
            Ok(Value::Promise(promise))
        }
        "package-check" => {
            let (identity, version) = crate::package::check_path(std::path::Path::new(
                string_argument(arguments, 0, operation)?,
            ))?;
            Ok(Value::Map(
                [
                    (keyword("identity"), Value::String(identity)),
                    (keyword("version"), Value::String(version)),
                ]
                .into_iter()
                .collect(),
            ))
        }
        "package-build" => {
            let input = std::path::Path::new(string_argument(arguments, 0, operation)?);
            let output =
                optional_string_argument(arguments, 1, operation)?.map(std::path::Path::new);
            Ok(Value::String(
                crate::package::build_path(input, output)?
                    .to_string_lossy()
                    .into_owned(),
            ))
        }
        "package-inspect" => Ok(Value::String(crate::package::inspect_path(
            std::path::Path::new(string_argument(arguments, 0, operation)?),
        )?)),
        "package-install" => Ok(Value::String(
            crate::package::install_path(std::path::Path::new(string_argument(
                arguments, 0, operation,
            )?))?
            .to_string_lossy()
            .into_owned(),
        )),
        "package-publish" => Ok(Value::String(crate::package::publish_path(
            std::path::Path::new(string_argument(arguments, 0, operation)?),
            string_argument(arguments, 1, operation)?,
            boolean_argument(arguments, 2, operation)?,
        )?)),
        "package-registry-verify" => {
            let request = std::path::Path::new(string_argument(arguments, 0, operation)?);
            let identity = std::path::Path::new(string_argument(arguments, 1, operation)?);
            crate::package::verify_registry_request_paths(request, identity)?;
            Ok(Value::String(format!(
                "registry request verified: {}",
                request.display()
            )))
        }
        "tap-config-root" => Ok(Value::String(
            crate::tap::config_root().to_string_lossy().into_owned(),
        )),
        "tap-add" => {
            let root = std::path::Path::new(string_argument(arguments, 0, operation)?);
            let name = string_argument(arguments, 1, operation)?;
            let tap = crate::tap::Tap {
                name: name.into(),
                registry: strings_argument(arguments, 2, operation)?,
                identity: strings_argument(arguments, 3, operation)?,
                identity_key: string_argument(arguments, 4, operation)?.into(),
                trust: crate::tap::TrustMode::SignedRoot,
            };
            crate::tap::add(root, tap.clone())?;
            Ok(tap_value(&tap))
        }
        "tap-bootstrap" => Ok(tap_value(&crate::tap::bootstrap(
            std::path::Path::new(string_argument(arguments, 0, operation)?),
            string_argument(arguments, 1, operation)?,
        )?)),
        "tap-remove" => {
            crate::tap::remove(
                std::path::Path::new(string_argument(arguments, 0, operation)?),
                string_argument(arguments, 1, operation)?,
            )?;
            Ok(Value::Nil)
        }
        "tap-list" => Ok(Value::Vector(
            crate::tap::load(std::path::Path::new(string_argument(
                arguments, 0, operation,
            )?))?
            .values()
            .map(tap_value)
            .collect(),
        )),
        "tap-mirror-add" => Ok(tap_value(&crate::tap::add_mirror(
            std::path::Path::new(string_argument(arguments, 0, operation)?),
            string_argument(arguments, 1, operation)?,
            optional_string_argument(arguments, 2, operation)?.map(str::to_owned),
            optional_string_argument(arguments, 3, operation)?.map(str::to_owned),
        )?)),
        "tap-initialize" => {
            let initialized = crate::tap::initialize(
                string_argument(arguments, 1, operation)?,
                std::path::Path::new(string_argument(arguments, 2, operation)?),
                std::path::Path::new(string_argument(arguments, 3, operation)?),
                string_argument(arguments, 4, operation)?,
            )?;
            crate::tap::add(
                std::path::Path::new(string_argument(arguments, 0, operation)?),
                initialized.tap.clone(),
            )?;
            Ok(Value::Map(
                [
                    (keyword("tap"), tap_value(&initialized.tap)),
                    (
                        keyword("fingerprint"),
                        Value::String(initialized.fingerprint),
                    ),
                ]
                .into_iter()
                .collect(),
            ))
        }
        "tap-verify" => {
            let name = string_argument(arguments, 1, operation)?;
            let policy = crate::tap::verify_trusted(
                std::path::Path::new(string_argument(arguments, 0, operation)?),
                name,
            )?;
            Ok(Value::Map(
                [
                    (keyword("name"), Value::String(name.into())),
                    (keyword("revision"), Value::String(policy.revision)),
                ]
                .into_iter()
                .collect(),
            ))
        }
        "snapshot-build" => Ok(Value::String(crate::snapshot_tool::build_paths(
            std::path::Path::new(string_argument(arguments, 0, operation)?),
            std::path::Path::new(string_argument(arguments, 1, operation)?),
        )?)),
        "snapshot-verify" => Ok(Value::String(crate::snapshot_tool::verify_paths(
            std::path::Path::new(string_argument(arguments, 0, operation)?),
            optional_string_argument(arguments, 1, operation)?.map(std::path::Path::new),
        )?)),
        "snapshot-inspect" => Ok(Value::String(crate::snapshot_tool::inspect_path(
            std::path::Path::new(string_argument(arguments, 0, operation)?),
        )?)),
        "snapshot-diff" => Ok(Value::String(crate::snapshot_tool::diff_paths(
            std::path::Path::new(string_argument(arguments, 0, operation)?),
            std::path::Path::new(string_argument(arguments, 1, operation)?),
        )?)),
        "session-create" => {
            broker.create(string_argument(arguments, 0, operation)?)?;
            Ok(Value::Nil)
        }
        "session-close" => {
            broker.close(string_argument(arguments, 0, operation)?)?;
            Ok(Value::Nil)
        }
        "session-list" => Ok(strings_value(broker.list()?)),
        "session-info" => {
            let name = string_argument(arguments, 0, operation)?;
            let info = broker.info(name)?;
            let namespace = info
                .split_once(' ')
                .map(|(_, namespace)| namespace)
                .unwrap_or("user");
            Ok(Value::Map(
                [
                    (keyword("name"), Value::String(name.into())),
                    (
                        keyword("namespace"),
                        Value::Symbol(Symbol::parse(namespace)),
                    ),
                    (keyword("state"), keyword("idle")),
                    (keyword("filesystem"), Value::Nil),
                ]
                .into_iter()
                .collect(),
            ))
        }
        "session-eval" => {
            let output = broker.eval(
                string_argument(arguments, 0, operation)?,
                string_argument(arguments, 1, operation)?,
            )?;
            let form = crate::kernel::parse(&output)?;
            crate::core::form_to_value(&form)
        }
        "session-namespace" => Ok(Value::Symbol(Symbol::parse(
            &broker.namespace(string_argument(arguments, 0, operation)?)?,
        ))),
        "session-complete" => Ok(strings_value(broker.complete(
            string_argument(arguments, 0, operation)?,
            string_argument(arguments, 1, operation)?,
        )?)),
        "resource-register" => {
            broker.register_resource(
                string_argument(arguments, 0, operation)?,
                string_argument(arguments, 1, operation)?,
            )?;
            Ok(Value::Nil)
        }
        "resource-remove" => {
            broker.remove_resource(string_argument(arguments, 0, operation)?)?;
            Ok(Value::Nil)
        }
        "resource-list" => Ok(strings_value(broker.resources()?)),
        "capabilities" => Ok(Value::Map(
            [
                (keyword("sessions"), Value::Bool(true)),
                (keyword("resources"), Value::Bool(true)),
                (keyword("filesystems"), Value::Bool(false)),
            ]
            .into_iter()
            .collect(),
        )),
        operation if operation.starts_with("filesystem-") => {
            Err(format!("{operation} is unavailable in the runtime broker"))
        }
        _ => Err(format!("unknown foundation.kernel operation: {operation}")),
    }
}

fn sandbox_id_argument(
    arguments: &[Value],
    index: usize,
    operation: &str,
) -> Result<SandboxId, String> {
    match arguments.get(index) {
        Some(Value::Number(value)) if *value > 0 => {
            SandboxId::parse(*value as u64).map_err(|error| error.to_string())
        }
        _ => Err(format!(
            "{operation}: sandbox id must be a positive integer"
        )),
    }
}

fn sandbox_spec_argument(arguments: &[Value], operation: &str) -> Result<SandboxSpec, String> {
    let entries = match arguments.first() {
        Some(Value::Map(entries)) => entries.iter().collect::<Vec<_>>(),
        Some(Value::OrderedMap(entries)) => {
            entries.iter().map(|(key, value)| (key, value)).collect()
        }
        _ => return Err(format!("{operation}: expected a SandboxSpec map")),
    };
    let allowed = [
        "protocol",
        "provider",
        "runtime",
        "entry-namespace",
        "bundles",
        "mount",
        "provider-options",
        "limits",
    ];
    for (key, _) in &entries {
        let Value::Keyword(key) = key else {
            return Err(format!("{operation}: SandboxSpec keys must be keywords"));
        };
        if !allowed.contains(&key.as_str()) {
            return Err(format!(
                "{operation}: unknown SandboxSpec key :{}",
                key.as_str()
            ));
        }
    }
    let lookup = |name: &str| {
        entries
            .iter()
            .find(|(key, _)| matches!(key, Value::Keyword(value) if value.as_str() == name))
            .map(|(_, value)| *value)
    };
    let text = |name: &str, fallback: &str| -> Result<String, String> {
        match lookup(name) {
            None | Some(Value::Nil) => Ok(fallback.into()),
            Some(Value::String(value)) => Ok(value.clone()),
            Some(Value::Keyword(value)) => Ok(value.as_str().into()),
            Some(Value::Symbol(value)) => Ok(value.as_str().into()),
            _ => Err(format!("{operation}: :{name} must be text-like")),
        }
    };
    SandboxSpec::new(
        text("protocol", crate::SANDBOX_SPEC_PROTOCOL)?,
        text("provider", "in-process")?,
        text("runtime", "hara.standard/0-alpha")?,
        text("entry-namespace", "user")?,
        crate::SandboxLimits::default(),
    )
    .map_err(|error| error.to_string())
}

fn sandbox_string_promise(
    broker: RuntimeBroker,
    sandbox: SandboxId,
    receiver: mpsc::Receiver<Result<String, String>>,
) -> Promise {
    let promise = Promise::new();
    let waiting = Rc::new(RefCell::new(Some(receiver)));
    let settled = promise.clone();
    promise.set_waiter(Rc::new(move || {
        let Some(receiver) = waiting.borrow_mut().take() else {
            return;
        };
        match receiver.recv() {
            Ok(Ok(value)) => {
                let form =
                    crate::kernel::parse(&value).and_then(|form| crate::core::form_to_value(&form));
                match form {
                    Ok(value) => {
                        settled.resolve(value);
                    }
                    Err(error) => {
                        settled.reject(error);
                    }
                }
            }
            Ok(Err(error)) => {
                settled.reject(error);
            }
            Err(_) => {
                settled.reject("sandbox provider dropped the evaluation result");
            }
        }
    }));
    promise.set_cancel_hook(Rc::new(move || {
        let _ = broker.sandbox_cancel(sandbox);
    }));
    promise
}

fn sandbox_hta_promise(
    broker: RuntimeBroker,
    sandbox: SandboxId,
    receiver: mpsc::Receiver<Result<Vec<u8>, String>>,
) -> Promise {
    let promise = Promise::new();
    let waiting = Rc::new(RefCell::new(Some(receiver)));
    let settled = promise.clone();
    promise.set_waiter(Rc::new(move || {
        let Some(receiver) = waiting.borrow_mut().take() else {
            return;
        };
        match receiver.recv() {
            Ok(Ok(value)) => match crate::hta::decode(&value) {
                Ok(value) => {
                    settled.resolve(value);
                }
                Err(error) => {
                    settled.reject(error);
                }
            },
            Ok(Err(error)) => {
                settled.reject(error);
            }
            Err(_) => {
                settled.reject("sandbox provider dropped the call result");
            }
        }
    }));
    promise.set_cancel_hook(Rc::new(move || {
        let _ = broker.sandbox_cancel(sandbox);
    }));
    promise
}

fn sandbox_status_value(status: SandboxStatus) -> Value {
    let error = status.error.map_or(Value::Nil, |error| {
        Value::Map(
            [
                (
                    keyword("code"),
                    keyword(&format!("{:?}", error.code).to_ascii_lowercase()),
                ),
                (keyword("message"), Value::String(error.message)),
            ]
            .into_iter()
            .collect(),
        )
    });
    Value::Map(
        [
            (keyword("id"), Value::Number(status.id.get() as i64)),
            (keyword("provider"), Value::String(status.provider)),
            (keyword("state"), keyword(status.state.as_str())),
            (keyword("secure"), Value::Bool(status.secure)),
            (
                keyword("evaluation-active"),
                Value::Bool(status.evaluation_active),
            ),
            (keyword("error"), error),
        ]
        .into_iter()
        .collect(),
    )
}

#[cfg(test)]
mod tests;
