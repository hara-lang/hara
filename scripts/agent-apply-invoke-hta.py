#!/usr/bin/env python3
from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one replacement, found {count}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


replace_once(
    "core/rust/src/lib.rs",
    '#[cfg(not(target_arch = "wasm32"))]\npub mod hta;\n',
    '#[cfg(not(target_arch = "wasm32"))]\npub mod hta;\n'
    '#[cfg(not(target_arch = "wasm32"))]\npub mod invoke_hta;\n'
    '#[cfg(not(target_arch = "wasm32"))]\npub use invoke_hta::{InvokeHtaError, MAX_INVOKE_HTA_RESULT_BYTES};\n',
)

replace_once(
    "core/rust/src/core.rs",
    "pub fn completion_symbols() -> &'static [&'static str] {\n"
    "    fiber::completion_symbols()\n"
    "}\n",
    "pub fn completion_symbols() -> &'static [&'static str] {\n"
    "    fiber::completion_symbols()\n"
    "}\n\n"
    "pub(crate) fn invoke_function_sync(\n"
    "    function: Rc<Function>,\n"
    "    arguments: Vec<Value>,\n"
    ") -> Result<Value, String> {\n"
    "    fiber::invoke_function_sync(function, arguments)\n"
    "}\n",
)

fiber = Path("core/rust/src/fiber.rs")
fiber_text = fiber.read_text(encoding="utf-8")
marker = "\n#[cfg(test)]\nmod tests {"
index = fiber_text.rfind(marker)
if index < 0:
    raise SystemExit("core/rust/src/fiber.rs: test module marker not found")
helper = r'''

pub(crate) fn invoke_function_sync(
    function: Rc<Function>,
    arguments: Vec<Value>,
) -> Result<Value, String> {
    let env = Rc::new(RefCell::new(HashMap::new()));
    let mut fiber = EvalFiber {
        env,
        pending: None,
        resume: None,
        state: EvalFiberState::Running,
    };
    fiber.accept(call(function, arguments, Box::new(Step::Done)));
    fiber.drive_sync()
}
'''
fiber.write_text(fiber_text[:index] + helper + fiber_text[index:], encoding="utf-8")

Path("core/rust/src/invoke_hta.rs").write_text(r'''//! Binary-safe invocation of already-loaded, fully qualified Hara Vars.
//!
//! This boundary deliberately does not parse, compile, macroexpand, load, or
//! evaluate source text. Embedding hosts remain responsible for a closed Var
//! allowlist before calling it.

use crate::core::{self, PromiseState, Value};
use crate::lang::data::Symbol;
use crate::{hta, Runtime};
use std::error::Error;
use std::fmt;
use std::rc::Rc;

pub const MAX_INVOKE_HTA_RESULT_BYTES: usize = 256 * 1024;
const MAX_PROMISE_UNWRAP_DEPTH: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvokeHtaError {
    InvalidQualifiedVar,
    MalformedInput(String),
    NoncanonicalInput,
    ArgumentsNotVector,
    NamespaceMissing(String),
    VarMissing(String),
    VarNotCallable(String),
    Execution(String),
    PromiseRejected(String),
    PromisePending,
    PromiseDepthExceeded,
    UnsupportedResult(String),
    ResultTooLarge { actual: usize, maximum: usize },
    SessionMissing(String),
    BrokerClosed,
    BrokerStopped,
}

impl InvokeHtaError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidQualifiedVar => "invoke-hta/qualified-var-invalid",
            Self::MalformedInput(_) => "invoke-hta/input-malformed",
            Self::NoncanonicalInput => "invoke-hta/input-noncanonical",
            Self::ArgumentsNotVector => "invoke-hta/arguments-not-vector",
            Self::NamespaceMissing(_) => "invoke-hta/namespace-missing",
            Self::VarMissing(_) => "invoke-hta/var-missing",
            Self::VarNotCallable(_) => "invoke-hta/var-not-callable",
            Self::Execution(_) => "invoke-hta/execution-failed",
            Self::PromiseRejected(_) => "invoke-hta/promise-rejected",
            Self::PromisePending => "invoke-hta/promise-pending",
            Self::PromiseDepthExceeded => "invoke-hta/promise-depth-exceeded",
            Self::UnsupportedResult(_) => "invoke-hta/result-unsupported",
            Self::ResultTooLarge { .. } => "invoke-hta/result-too-large",
            Self::SessionMissing(_) => "invoke-hta/session-missing",
            Self::BrokerClosed => "invoke-hta/broker-closed",
            Self::BrokerStopped => "invoke-hta/broker-stopped",
        }
    }
}

impl fmt::Display for InvokeHtaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.code())?;
        match self {
            Self::MalformedInput(detail)
            | Self::Execution(detail)
            | Self::PromiseRejected(detail)
            | Self::UnsupportedResult(detail) => write!(formatter, ": {detail}"),
            Self::NamespaceMissing(namespace) => write!(formatter, ": {namespace}"),
            Self::VarMissing(path) | Self::VarNotCallable(path) => {
                write!(formatter, ": {path}")
            }
            Self::ResultTooLarge { actual, maximum } => {
                write!(formatter, ": {actual} exceeds {maximum} bytes")
            }
            Self::SessionMissing(session) => write!(formatter, ": {session}"),
            _ => Ok(()),
        }
    }
}

impl Error for InvokeHtaError {}

impl Runtime {
    pub fn invoke_hta(
        &mut self,
        qualified_var: &str,
        arguments_hta: &[u8],
    ) -> Result<Vec<u8>, InvokeHtaError> {
        let (namespace_name, var_name) = split_qualified_var(qualified_var)?;
        let decoded = hta::decode(arguments_hta).map_err(InvokeHtaError::MalformedInput)?;
        let canonical = hta::encode(&decoded).map_err(InvokeHtaError::MalformedInput)?;
        if canonical != arguments_hta {
            return Err(InvokeHtaError::NoncanonicalInput);
        }
        let arguments = match decoded {
            Value::Vector(values) => values.iter().cloned().collect::<Vec<_>>(),
            _ => return Err(InvokeHtaError::ArgumentsNotVector),
        };

        let namespace = self
            .namespace_registry
            .find(namespace_name)
            .ok_or_else(|| InvokeHtaError::NamespaceMissing(namespace_name.to_owned()))?;
        let symbol = Symbol::parse(var_name);
        let var = namespace
            .resolve(&symbol)
            .ok_or_else(|| InvokeHtaError::VarMissing(qualified_var.to_owned()))?;
        let function = match var.deref_value() {
            Value::Function(function) => function,
            _ => return Err(InvokeHtaError::VarNotCallable(qualified_var.to_owned())),
        };

        let result = self
            .invoke_loaded_function(function, arguments)
            .map_err(InvokeHtaError::Execution)?;
        let result = settle_result(result)?;
        let encoded = hta::encode(&result).map_err(InvokeHtaError::UnsupportedResult)?;
        if encoded.len() > MAX_INVOKE_HTA_RESULT_BYTES {
            return Err(InvokeHtaError::ResultTooLarge {
                actual: encoded.len(),
                maximum: MAX_INVOKE_HTA_RESULT_BYTES,
            });
        }
        Ok(encoded)
    }

    fn invoke_loaded_function(
        &mut self,
        function: Rc<core::Function>,
        arguments: Vec<Value>,
    ) -> Result<Value, String> {
        let namespace_source = self.namespace_source();
        core::with_capability_providers(
            self.providers.file(),
            self.providers.socket(),
            self.providers.process(),
            self.providers.kernel(),
            || {
                core::with_promise_provider(self.providers.promise(), || {
                    core::with_macros(self.macros.clone(), || {
                        core::with_namespace_registry(&self.namespace_registry, || {
                            core::with_namespace_source(namespace_source, || {
                                core::with_protocols(&self.protocols, || {
                                    if let Some(handler) = &self.native_host_handler {
                                        return core::with_host_calls(handler.clone(), || {
                                            core::invoke_function_sync(function, arguments)
                                        });
                                    }
                                    core::invoke_function_sync(function, arguments)
                                })
                            })
                        })
                    })
                })
            },
        )
    }
}

fn split_qualified_var(value: &str) -> Result<(&str, &str), InvokeHtaError> {
    let Some((namespace, name)) = value.split_once('/') else {
        return Err(InvokeHtaError::InvalidQualifiedVar);
    };
    if namespace.is_empty()
        || name.is_empty()
        || name.contains('/')
        || value.bytes().any(|byte| byte.is_ascii_whitespace())
    {
        return Err(InvokeHtaError::InvalidQualifiedVar);
    }
    Ok((namespace, name))
}

fn settle_result(mut value: Value) -> Result<Value, InvokeHtaError> {
    for _ in 0..MAX_PROMISE_UNWRAP_DEPTH {
        let Value::Promise(promise) = value else {
            return Ok(value);
        };
        value = match promise.wait_state() {
            PromiseState::Fulfilled(value) => value,
            PromiseState::Rejected(error) => {
                return Err(InvokeHtaError::PromiseRejected(
                    error.message().to_owned(),
                ))
            }
            PromiseState::Pending => return Err(InvokeHtaError::PromisePending),
        };
    }
    Err(InvokeHtaError::PromiseDepthExceeded)
}
''', encoding="utf-8")

replace_once(
    "core/rust/src/native_cli.rs",
    "use crate::core::Value;\nuse crate::lang::data::Symbol;\nuse crate::Runtime;\n",
    "use crate::core::Value;\nuse crate::invoke_hta::InvokeHtaError;\n"
    "use crate::lang::data::Symbol;\nuse crate::Runtime;\n",
)

replace_once(
    "core/rust/src/native_cli.rs",
    "    InvokeModule {\n"
    "        session: String,\n"
    "        namespace: String,\n"
    "        export: String,\n"
    "        arguments: Vec<u8>,\n"
    "        reply: mpsc::Sender<Result<Vec<u8>, String>>,\n"
    "    },\n"
    "    Shutdown,\n",
    "    InvokeModule {\n"
    "        session: String,\n"
    "        namespace: String,\n"
    "        export: String,\n"
    "        arguments: Vec<u8>,\n"
    "        reply: mpsc::Sender<Result<Vec<u8>, String>>,\n"
    "    },\n"
    "    InvokeHta {\n"
    "        session: String,\n"
    "        qualified_var: String,\n"
    "        arguments: Vec<u8>,\n"
    "        reply: mpsc::Sender<Result<Vec<u8>, InvokeHtaError>>,\n"
    "    },\n"
    "    Shutdown,\n",
)

replace_once(
    "core/rust/src/native_cli.rs",
    "    pub fn invoke_module(\n",
    "    pub fn invoke_hta(\n"
    "        &self,\n"
    "        session: &str,\n"
    "        qualified_var: &str,\n"
    "        arguments: &[u8],\n"
    "    ) -> Result<Vec<u8>, InvokeHtaError> {\n"
    "        let (reply, response) = mpsc::channel();\n"
    "        self.handle\n"
    "            .sender\n"
    "            .send(Request::InvokeHta {\n"
    "                session: session.into(),\n"
    "                qualified_var: qualified_var.into(),\n"
    "                arguments: arguments.into(),\n"
    "                reply,\n"
    "            })\n"
    "            .map_err(|_| InvokeHtaError::BrokerClosed)?;\n"
    "        response\n"
    "            .recv()\n"
    "            .map_err(|_| InvokeHtaError::BrokerStopped)?\n"
    "    }\n\n"
    "    pub fn invoke_module(\n",
)

replace_once(
    "core/rust/src/native_cli.rs",
    "            Request::Shutdown => break,\n",
    "            Request::InvokeHta {\n"
    "                session,\n"
    "                qualified_var,\n"
    "                arguments,\n"
    "                reply,\n"
    "            } => {\n"
    "                let result = sessions\n"
    "                    .get_mut(&session)\n"
    "                    .ok_or_else(|| InvokeHtaError::SessionMissing(session.clone()))\n"
    "                    .and_then(|runtime| runtime.invoke_hta(&qualified_var, &arguments));\n"
    "                let _ = reply.send(result);\n"
    "            }\n"
    "            Request::Shutdown => break,\n",
)

Path("core/rust/tests/invoke_hta.rs").write_text(r'''use hara_wasm::core::Value;
use hara_wasm::hta;
use hara_wasm::native_cli::RuntimeBroker;
use hara_wasm::{InvokeHtaError, Runtime, MAX_INVOKE_HTA_RESULT_BYTES};

fn arguments(values: Vec<Value>) -> Vec<u8> {
    hta::encode(&Value::Vector(values.into())).expect("canonical arguments")
}

fn number(bytes: &[u8]) -> i64 {
    let Value::Number(value) = hta::decode(bytes).expect("result HTA") else {
        panic!("expected integer result")
    };
    value
}

#[test]
fn runtime_invokes_only_an_already_loaded_qualified_var() {
    let mut runtime = Runtime::new();
    runtime
        .eval_native("(ns invoke.sample) (defn add [a b] (+ a b)) (def answer 42)")
        .expect("load sample namespace");

    let result = runtime
        .invoke_hta(
            "invoke.sample/add",
            &arguments(vec![Value::Number(20), Value::Number(22)]),
        )
        .expect("invoke add");
    assert_eq!(number(&result), 42);

    assert_eq!(
        runtime.invoke_hta("add", &arguments(vec![])),
        Err(InvokeHtaError::InvalidQualifiedVar)
    );
    assert_eq!(
        runtime.invoke_hta("missing.namespace/add", &arguments(vec![])),
        Err(InvokeHtaError::NamespaceMissing(
            "missing.namespace".to_owned()
        ))
    );
    assert_eq!(
        runtime.invoke_hta("invoke.sample/missing", &arguments(vec![])),
        Err(InvokeHtaError::VarMissing(
            "invoke.sample/missing".to_owned()
        ))
    );
    assert_eq!(
        runtime.invoke_hta("invoke.sample/answer", &arguments(vec![])),
        Err(InvokeHtaError::VarNotCallable(
            "invoke.sample/answer".to_owned()
        ))
    );
}

#[test]
fn runtime_rejects_malformed_noncanonical_and_non_vector_arguments() {
    let mut runtime = Runtime::new();
    runtime
        .eval_native("(ns invoke.input) (defn identity* [value] value)")
        .expect("load input namespace");

    assert!(matches!(
        runtime.invoke_hta("invoke.input/identity*", b"not-hta"),
        Err(InvokeHtaError::MalformedInput(_))
    ));
    assert_eq!(
        runtime.invoke_hta(
            "invoke.input/identity*",
            &hta::encode(&Value::Number(1)).expect("scalar HTA")
        ),
        Err(InvokeHtaError::ArgumentsNotVector)
    );

    let mut noncanonical = b"HTA0".to_vec();
    noncanonical.push(11);
    noncanonical.extend_from_slice(&2_u32.to_be_bytes());
    for (key, value) in [(b'z', 1_i64), (b'a', 2_i64)] {
        noncanonical.push(6);
        noncanonical.extend_from_slice(&1_u32.to_be_bytes());
        noncanonical.push(key);
        noncanonical.push(3);
        noncanonical.extend_from_slice(&value.to_be_bytes());
    }
    assert_eq!(
        runtime.invoke_hta("invoke.input/identity*", &noncanonical),
        Err(InvokeHtaError::NoncanonicalInput)
    );
}

#[test]
fn runtime_settles_fulfilled_promises_and_bounds_results() {
    let mut runtime = Runtime::new();
    runtime
        .eval_native(
            "(ns invoke.promise)\n\
             (defn promised [value] (promise/from value))\n\
             (defn huge [] (str/repeat \"x\" 270000))",
        )
        .expect("load promise namespace");

    let result = runtime
        .invoke_hta(
            "invoke.promise/promised",
            &arguments(vec![Value::Number(42)]),
        )
        .expect("settle fulfilled promise");
    assert_eq!(number(&result), 42);

    assert!(matches!(
        runtime.invoke_hta("invoke.promise/huge", &arguments(vec![])),
        Err(InvokeHtaError::ResultTooLarge {
            maximum: MAX_INVOKE_HTA_RESULT_BYTES,
            ..
        })
    ));
}

#[test]
fn broker_keeps_invoke_hta_session_isolated() {
    let broker = RuntimeBroker::start().expect("broker");
    broker
        .eval("ROOT", "(ns invoke.broker) (defn value [] 1)")
        .expect("root function");
    broker.create("SECOND").expect("second session");
    broker
        .eval("SECOND", "(ns invoke.broker) (defn value [] 2)")
        .expect("second function");

    assert_eq!(
        number(
            &broker
                .invoke_hta("ROOT", "invoke.broker/value", &arguments(vec![]))
                .expect("root invoke")
        ),
        1
    );
    assert_eq!(
        number(
            &broker
                .invoke_hta("SECOND", "invoke.broker/value", &arguments(vec![]))
                .expect("second invoke")
        ),
        2
    );
    assert_eq!(
        broker.invoke_hta("MISSING", "invoke.broker/value", &arguments(vec![])),
        Err(InvokeHtaError::SessionMissing("MISSING".to_owned()))
    );
}
''', encoding="utf-8")

Path("core/rust/src/bin/hara-invoke-hta-benchmark.rs").write_text(r'''#![cfg(not(target_arch = "wasm32"))]

use hara_wasm::core::Value;
use hara_wasm::{hta, Runtime};
use std::time::Instant;

fn main() -> Result<(), String> {
    let iterations = std::env::var("HARA_INVOKE_HTA_ITERATIONS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(10_000);
    let mut runtime = Runtime::new();
    runtime.eval_native("(ns invoke.benchmark) (defn add [a b] (+ a b))")?;
    let arguments = hta::encode(&Value::Vector(
        vec![Value::Number(20), Value::Number(22)].into(),
    ))?;

    for _ in 0..100 {
        runtime
            .invoke_hta("invoke.benchmark/add", &arguments)
            .map_err(|error| error.to_string())?;
    }
    let started = Instant::now();
    let mut result = Vec::new();
    for _ in 0..iterations {
        result = runtime
            .invoke_hta("invoke.benchmark/add", &arguments)
            .map_err(|error| error.to_string())?;
    }
    let elapsed = started.elapsed();
    if hta::decode(&result)? != Value::Number(42) {
        return Err("invoke HTA benchmark checksum failed".into());
    }
    let nanos = elapsed.as_nanos() as f64 / iterations as f64;
    println!(
        "invoke_hta iterations={iterations} total_ns={} ns_per_call={nanos:.2}",
        elapsed.as_nanos()
    );
    Ok(())
}
''', encoding="utf-8")

Path("notes/native-invoke-hta.md").write_text(r'''# Binary-safe qualified-Var invocation

Native embedding hosts may invoke one already-loaded, fully qualified Hara Var
without constructing or evaluating source text:

```rust
Runtime::invoke_hta(qualified_var, arguments_hta)
RuntimeBroker::invoke_hta(session, qualified_var, arguments_hta)
```

The input is one canonical HTA0 vector of arguments. The result is one canonical
HTA0 value bounded to 256 KiB. The boundary rejects unqualified or missing Vars,
non-callable Vars, malformed or noncanonical input, unsupported result values,
rejected promises, and oversized results with stable typed errors.

Var resolution is direct against the prepared namespace registry. The method does
not call the parser, macroexpander, compiler, namespace loader, or source evaluator.
Native capability providers and the reviewed host callback remain active while the
already-loaded function executes. Downstream hosts must apply their own closed Var
allowlist before calling this API; it is not a general IPC evaluation surface.

Run the focused benchmark with:

```text
cargo run --manifest-path core/rust/Cargo.toml --release \
  --bin hara-invoke-hta-benchmark
```
''', encoding="utf-8")
