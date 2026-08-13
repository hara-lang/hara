#!/usr/bin/env python3
from pathlib import Path

path = Path("core/rust/src/native_cli.rs")
text = path.read_text(encoding="utf-8")

old = "mod arguments;\nmod documentation;\n"
new = (
    "mod arguments;\n"
    "mod bootstrap;\n"
    "mod documentation;\n"
    "mod invoke_hta;\n"
    "use bootstrap::RuntimeBootstrap;\n"
)
if text.count(old) != 1:
    raise SystemExit("native_cli module header changed unexpectedly")
text = text.replace(old, new, 1)

runtime_bootstrap = """#[derive(Clone, Copy)]
enum RuntimeBootstrap {
    Full,
    Core,
}

"""
if text.count(runtime_bootstrap) != 1:
    raise SystemExit("RuntimeBootstrap declaration changed unexpectedly")
text = text.replace(runtime_bootstrap, "", 1)

start = text.index("    pub fn start() -> Result<Self, String> {\n")
end = text.index(
    "    pub fn eval(&self, session: &str, source: &str) -> Result<String, String> {\n",
    start,
)
bootstrap_methods = text[start:end]
text = text[:start] + text[end:]

start = text.index("    pub fn invoke_hta(\n")
end = text.index("    pub fn invoke_module(\n", start)
invoke_method = text[start:end]
text = text[:start] + text[end:]
path.write_text(text, encoding="utf-8")

module_dir = Path("core/rust/src/native_cli")
module_dir.mkdir(exist_ok=True)

bootstrap = (
    "use super::*;\n\n"
    "#[derive(Clone, Copy)]\n"
    "pub(super) enum RuntimeBootstrap {\n"
    "    Full,\n"
    "    Core,\n"
    "}\n\n"
    "impl RuntimeBroker {\n"
    + bootstrap_methods
    + "}\n"
)
(module_dir / "bootstrap.rs").write_text(bootstrap, encoding="utf-8")

invoke = "use super::*;\n\nimpl RuntimeBroker {\n" + invoke_method + "}\n"
(module_dir / "invoke_hta.rs").write_text(invoke, encoding="utf-8")
