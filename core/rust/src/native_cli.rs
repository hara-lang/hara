#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{mpsc, Arc};

use crate::core::Value;
use crate::lang::data::Symbol;
use crate::Runtime;

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
        Self::start_with(None, false, false, false)
    }

    pub fn start_with(
        root: Option<PathBuf>,
        native_sockets: bool,
        allow_process: bool,
        allow_postgres: bool,
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
) -> Runtime {
    let mut runtime = Runtime::new();
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
            .expect("std.db.postgres native module must install once per runtime");
    }
    runtime
}

fn run(
    receiver: mpsc::Receiver<Request>,
    root: Option<PathBuf>,
    native_sockets: bool,
    allow_process: bool,
    allow_postgres: bool,
) {
    let mut resources = HashMap::<String, String>::new();
    let mut sessions = HashMap::from([(
        "ROOT".to_owned(),
        runtime(root.as_ref(), native_sockets, allow_process, allow_postgres),
    )]);
    while let Ok(request) = receiver.recv() {
        match request {
            Request::Eval {
                session,
                source,
                reply,
            } => {
                let result = sessions
                    .get_mut(&session)
                    .ok_or_else(|| format!("No session: {session}"))
                    .and_then(|runtime| runtime.eval_native_traced(&source));
                let _ = reply.send(result);
            }
            Request::Namespace { session, reply } => {
                let result = sessions
                    .get(&session)
                    .map(Runtime::current_namespace)
                    .ok_or_else(|| format!("No session: {session}"));
                let _ = reply.send(result);
            }
            Request::Complete {
                session,
                prefix,
                reply,
            } => {
                let result = sessions
                    .get(&session)
                    .map(|runtime| {
                        let mut symbols = runtime
                            .visible_symbols()
                            .into_iter()
                            .filter(|symbol| symbol.starts_with(&prefix))
                            .collect::<Vec<_>>();
                        symbols.sort();
                        symbols.dedup();
                        symbols
                    })
                    .ok_or_else(|| format!("No session: {session}"));
                let _ = reply.send(result);
            }
            Request::Doc {
                session,
                symbol,
                reply,
            } => {
                let result = sessions
                    .get(&session)
                    .ok_or_else(|| format!("No session: {session}"))
                    .and_then(|runtime| documentation(runtime, &symbol));
                let _ = reply.send(result);
            }
            Request::Create { session, reply } => {
                let result = if session.is_empty() || sessions.contains_key(&session) {
                    Err(format!("Session already exists or is invalid: {session}"))
                } else {
                    let mut created =
                        runtime(root.as_ref(), native_sockets, allow_process, allow_postgres);
                    for (name, source) in &resources {
                        created.register_resource(name, source);
                    }
                    sessions.insert(session.clone(), created);
                    Ok(session)
                };
                let _ = reply.send(result);
            }
            Request::Close { session, reply } => {
                let result = if session == "ROOT" {
                    Err("ROOT cannot be closed".into())
                } else if sessions.remove(&session).is_some() {
                    Ok(session)
                } else {
                    Err(format!("No session: {session}"))
                };
                let _ = reply.send(result);
            }
            Request::List { reply } => {
                let mut names = sessions.keys().cloned().collect::<Vec<_>>();
                names.sort();
                let _ = reply.send(Ok(names));
            }
            Request::Info { session, reply } => {
                let result = sessions
                    .get(&session)
                    .map(|runtime| format!("{session} {}", runtime.current_namespace()))
                    .ok_or_else(|| format!("No session: {session}"));
                let _ = reply.send(result);
            }
            Request::RegisterResource {
                name,
                source,
                reply,
            } => {
                for runtime in sessions.values_mut() {
                    runtime.register_resource(&name, &source);
                }
                resources.insert(name, source);
                let _ = reply.send(Ok(()));
            }
            Request::RemoveResource { name, reply } => {
                resources.remove(&name);
                let _ = reply.send(Ok(()));
            }
            Request::ListResources { reply } => {
                let mut names = resources.keys().cloned().collect::<Vec<_>>();
                names.sort();
                let _ = reply.send(Ok(names));
            }
            Request::InstallModule {
                session,
                manifest,
                module,
                reply,
            } => {
                let result = sessions
                    .get_mut(&session)
                    .ok_or_else(|| format!("No session: {session}"))
                    .and_then(|runtime| {
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
                let result = sessions
                    .get_mut(&session)
                    .ok_or_else(|| format!("No session: {session}"))
                    .and_then(|runtime| {
                        let arguments = crate::hta::decode(&arguments)?;
                        let arguments: Vec<crate::extension::Value> = match arguments {
                            crate::extension::Value::Vector(values) => {
                                values.iter().cloned().collect()
                            }
                            crate::extension::Value::Tuple(values) => {
                                values.iter().cloned().collect()
                            }
                            other => {
                                return Err(format!(
                                    "hta/arguments: expected vector, got {}",
                                    other.display()
                                ))
                            }
                        };
                        let result =
                            runtime.invoke_wasm_extension(&namespace, &export, &arguments)?;
                        crate::hta::encode(&result)
                    });
                let _ = reply.send(result);
            }
            Request::Shutdown => break,
        }
    }
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

#[cfg(test)]
mod tests;
