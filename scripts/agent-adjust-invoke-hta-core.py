#!/usr/bin/env python3
from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one replacement, found {count}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


native = "core/rust/src/native_cli.rs"
replace_once(
    native,
    "enum Request {\n",
    "#[derive(Clone, Copy)]\n"
    "enum RuntimeBootstrap {\n"
    "    Full,\n"
    "    Core,\n"
    "}\n\n"
    "enum Request {\n",
)

replace_once(
    native,
    "impl RuntimeBroker {\n"
    "    pub fn start() -> Result<Self, String> {\n"
    "        Self::start_with(None, false, false, false)\n"
    "    }\n\n"
    "    pub fn start_with(\n"
    "        root: Option<PathBuf>,\n"
    "        native_sockets: bool,\n"
    "        allow_process: bool,\n"
    "        allow_postgres: bool,\n"
    "    ) -> Result<Self, String> {\n"
    "        let (sender, receiver) = mpsc::channel();\n"
    "        std::thread::Builder::new()\n"
    "            .name(\"hara-runtime-broker\".into())\n"
    "            .stack_size(RUNTIME_BROKER_STACK_SIZE)\n"
    "            .spawn(move || {\n"
    "                run(\n"
    "                    receiver,\n"
    "                    root,\n"
    "                    native_sockets,\n"
    "                    allow_process,\n"
    "                    allow_postgres,\n"
    "                )\n"
    "            })\n"
    "            .map_err(|error| format!(\"runtime broker failed: {error}\"))?;\n"
    "        Ok(Self {\n"
    "            handle: Arc::new(BrokerHandle { sender }),\n"
    "        })\n"
    "    }\n",
    "impl RuntimeBroker {\n"
    "    pub fn start() -> Result<Self, String> {\n"
    "        Self::start_with_bootstrap(\n"
    "            None,\n"
    "            false,\n"
    "            false,\n"
    "            false,\n"
    "            RuntimeBootstrap::Full,\n"
    "        )\n"
    "    }\n\n"
    "    /// Starts an isolated broker with the portable L0 runtime.\n"
    "    ///\n"
    "    /// This is intended for small embedding surfaces and focused tests\n"
    "    /// that do not require the language-level Foundation bundle.\n"
    "    pub fn start_core() -> Result<Self, String> {\n"
    "        Self::start_with_bootstrap(\n"
    "            None,\n"
    "            false,\n"
    "            false,\n"
    "            false,\n"
    "            RuntimeBootstrap::Core,\n"
    "        )\n"
    "    }\n\n"
    "    pub fn start_with(\n"
    "        root: Option<PathBuf>,\n"
    "        native_sockets: bool,\n"
    "        allow_process: bool,\n"
    "        allow_postgres: bool,\n"
    "    ) -> Result<Self, String> {\n"
    "        Self::start_with_bootstrap(\n"
    "            root,\n"
    "            native_sockets,\n"
    "            allow_process,\n"
    "            allow_postgres,\n"
    "            RuntimeBootstrap::Full,\n"
    "        )\n"
    "    }\n\n"
    "    fn start_with_bootstrap(\n"
    "        root: Option<PathBuf>,\n"
    "        native_sockets: bool,\n"
    "        allow_process: bool,\n"
    "        allow_postgres: bool,\n"
    "        bootstrap: RuntimeBootstrap,\n"
    "    ) -> Result<Self, String> {\n"
    "        let (sender, receiver) = mpsc::channel();\n"
    "        std::thread::Builder::new()\n"
    "            .name(\"hara-runtime-broker\".into())\n"
    "            .stack_size(RUNTIME_BROKER_STACK_SIZE)\n"
    "            .spawn(move || {\n"
    "                run(\n"
    "                    receiver,\n"
    "                    root,\n"
    "                    native_sockets,\n"
    "                    allow_process,\n"
    "                    allow_postgres,\n"
    "                    bootstrap,\n"
    "                )\n"
    "            })\n"
    "            .map_err(|error| format!(\"runtime broker failed: {error}\"))?;\n"
    "        Ok(Self {\n"
    "            handle: Arc::new(BrokerHandle { sender }),\n"
    "        })\n"
    "    }\n",
)

replace_once(
    native,
    "fn runtime(\n"
    "    root: Option<&PathBuf>,\n"
    "    native_sockets: bool,\n"
    "    allow_process: bool,\n"
    "    allow_postgres: bool,\n"
    ") -> Runtime {\n"
    "    let mut runtime = Runtime::new();\n",
    "fn runtime(\n"
    "    root: Option<&PathBuf>,\n"
    "    native_sockets: bool,\n"
    "    allow_process: bool,\n"
    "    allow_postgres: bool,\n"
    "    bootstrap: RuntimeBootstrap,\n"
    ") -> Runtime {\n"
    "    let mut runtime = match bootstrap {\n"
    "        RuntimeBootstrap::Full => Runtime::new(),\n"
    "        RuntimeBootstrap::Core => Runtime::core(),\n"
    "    };\n",
)

replace_once(
    native,
    "fn run(\n"
    "    receiver: mpsc::Receiver<Request>,\n"
    "    root: Option<PathBuf>,\n"
    "    native_sockets: bool,\n"
    "    allow_process: bool,\n"
    "    allow_postgres: bool,\n"
    ") {\n",
    "fn run(\n"
    "    receiver: mpsc::Receiver<Request>,\n"
    "    root: Option<PathBuf>,\n"
    "    native_sockets: bool,\n"
    "    allow_process: bool,\n"
    "    allow_postgres: bool,\n"
    "    bootstrap: RuntimeBootstrap,\n"
    ") {\n",
)

replace_once(
    native,
    "        runtime(root.as_ref(), native_sockets, allow_process, allow_postgres),\n",
    "        runtime(\n"
    "            root.as_ref(),\n"
    "            native_sockets,\n"
    "            allow_process,\n"
    "            allow_postgres,\n"
    "            bootstrap,\n"
    "        ),\n",
)

replace_once(
    native,
    "                    let mut created =\n"
    "                        runtime(root.as_ref(), native_sockets, allow_process, allow_postgres);\n",
    "                    let mut created = runtime(\n"
    "                        root.as_ref(),\n"
    "                        native_sockets,\n"
    "                        allow_process,\n"
    "                        allow_postgres,\n"
    "                        bootstrap,\n"
    "                    );\n",
)

integration = Path("core/rust/tests/invoke_hta.rs")
text = integration.read_text(encoding="utf-8")
if text.count("Runtime::new()") != 3:
    raise SystemExit("invoke_hta integration tests: unexpected Runtime::new count")
text = text.replace("Runtime::new()", "Runtime::core()")
text = text.replace("RuntimeBroker::start().expect(\"broker\")", "RuntimeBroker::start_core().expect(\"broker\")")
start = text.index("#[test]\nfn runtime_settles_fulfilled_promises_and_bounds_results()")
end = text.index("#[test]\nfn broker_keeps_invoke_hta_session_isolated()", start)
text = text[:start] + text[end:]
integration.write_text(text, encoding="utf-8")

benchmark = Path("core/rust/src/bin/hara-invoke-hta-benchmark.rs")
text = benchmark.read_text(encoding="utf-8")
if text.count("Runtime::new()") != 1:
    raise SystemExit("invoke HTA benchmark: unexpected Runtime::new count")
benchmark.write_text(text.replace("Runtime::new()", "Runtime::core()"), encoding="utf-8")

invoke = "core/rust/src/invoke_hta.rs"
replace_once(
    invoke,
    "        let result = settle_result(result)?;\n"
    "        let encoded = hta::encode(&result).map_err(InvokeHtaError::UnsupportedResult)?;\n"
    "        if encoded.len() > MAX_INVOKE_HTA_RESULT_BYTES {\n"
    "            return Err(InvokeHtaError::ResultTooLarge {\n"
    "                actual: encoded.len(),\n"
    "                maximum: MAX_INVOKE_HTA_RESULT_BYTES,\n"
    "            });\n"
    "        }\n"
    "        Ok(encoded)\n",
    "        encode_result(settle_result(result)?)\n",
)

path = Path(invoke)
text = path.read_text(encoding="utf-8")
text += r'''

fn encode_result(result: Value) -> Result<Vec<u8>, InvokeHtaError> {
    let encoded = hta::encode(&result).map_err(InvokeHtaError::UnsupportedResult)?;
    if encoded.len() > MAX_INVOKE_HTA_RESULT_BYTES {
        return Err(InvokeHtaError::ResultTooLarge {
            actual: encoded.len(),
            maximum: MAX_INVOKE_HTA_RESULT_BYTES,
        });
    }
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fulfilled_and_rejected_promises_are_typed() {
        let fulfilled = core::Promise::new();
        assert!(fulfilled.resolve(Value::Number(42)));
        assert_eq!(
            settle_result(Value::Promise(fulfilled)),
            Ok(Value::Number(42))
        );

        let rejected = core::Promise::new();
        assert!(rejected.reject("no"));
        assert_eq!(
            settle_result(Value::Promise(rejected)),
            Err(InvokeHtaError::PromiseRejected("no".to_owned()))
        );
    }

    #[test]
    fn encoded_results_are_bounded() {
        let result = Value::String("x".repeat(MAX_INVOKE_HTA_RESULT_BYTES));
        assert!(matches!(
            encode_result(result),
            Err(InvokeHtaError::ResultTooLarge {
                maximum: MAX_INVOKE_HTA_RESULT_BYTES,
                ..
            })
        ));
    }
}
'''
path.write_text(text, encoding="utf-8")
