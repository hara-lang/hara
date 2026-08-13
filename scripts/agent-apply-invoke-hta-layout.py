#!/usr/bin/env python3
from pathlib import Path

ROOT = Path("core/rust")


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    if text.count(old) != 1:
        raise SystemExit(f"{path}: expected exactly one replacement")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


# Move RuntimeBroker construction and HTA dispatch out of native_cli.rs.
native = ROOT / "src/native_cli.rs"
text = native.read_text(encoding="utf-8")
text = text.replace(
    "mod arguments;\nmod documentation;\n",
    "mod arguments;\nmod bootstrap;\nmod documentation;\nmod invoke_hta;\nuse bootstrap::RuntimeBootstrap;\n",
    1,
)
text = text.replace(
    "#[derive(Clone, Copy)]\nenum RuntimeBootstrap {\n    Full,\n    Core,\n}\n\n",
    "",
    1,
)
start = text.index("    pub fn start() -> Result<Self, String> {\n")
end = text.index("    pub fn eval(&self, session: &str, source: &str) -> Result<String, String> {\n", start)
bootstrap_methods = text[start:end]
text = text[:start] + text[end:]
start = text.index("    pub fn invoke_hta(\n")
end = text.index("    pub fn invoke_module(\n", start)
invoke_method = text[start:end]
text = text[:start] + text[end:]
native.write_text(text, encoding="utf-8")

native_dir = ROOT / "src/native_cli"
native_dir.mkdir(exist_ok=True)
(native_dir / "bootstrap.rs").write_text(
    "use super::*;\n\n"
    "#[derive(Clone, Copy)]\n"
    "pub(super) enum RuntimeBootstrap {\n    Full,\n    Core,\n}\n\n"
    "impl RuntimeBroker {\n"
    + "".join(line[4:] if line.startswith("    ") else line for line in bootstrap_methods.splitlines(True))
    + "}\n",
    encoding="utf-8",
)
(native_dir / "invoke_hta.rs").write_text(
    "use super::*;\n\nimpl RuntimeBroker {\n"
    + "".join(line[4:] if line.startswith("    ") else line for line in invoke_method.splitlines(True))
    + "}\n",
    encoding="utf-8",
)

# Move the inline HTA codec tests to a child module.
hta = ROOT / "src/hta.rs"
text = hta.read_text(encoding="utf-8")
text = text.replace(
    "#[cfg(test)]\nuse crate::lang::data::{Tuple as PTuple, Vector as PVector};\n",
    "",
    1,
)
marker = "#[cfg(test)]\nmod tests {\n"
start = text.index(marker)
module = text[start + len(marker):]
if not module.rstrip().endswith("}"):
    raise SystemExit("HTA test module must end at EOF")
module = module.rstrip()[:-1].rstrip() + "\n"
module = module.replace(
    "    use super::*;\n",
    "use super::*;\nuse crate::lang::data::{Tuple as PTuple, Vector as PVector};\n",
    1,
)
module = "".join(line[4:] if line.startswith("    ") else line for line in module.splitlines(True))
hta.write_text(text[:start] + '#[cfg(test)]\n#[path = "hta/tests.rs"]\nmod tests;\n', encoding="utf-8")
hta_dir = ROOT / "src/hta"
hta_dir.mkdir(exist_ok=True)
(hta_dir / "tests.rs").write_text(module, encoding="utf-8")

# Move the artifact Reader and Writer machinery to a child module.
artifact = ROOT / "src/vm/artifact.rs"
text = artifact.read_text(encoding="utf-8")
start_marker = "#[derive(Default)]\nstruct Writer {"
end_marker = '#[cfg(test)]\n#[path = "artifact/tests.rs"]\nmod tests;'
start = text.index(start_marker)
end = text.index(end_marker, start)
section = text[start:end]
section = section.replace("struct Writer {", "pub(super) struct Writer {", 1)
section = section.replace("    bytes: Vec<u8>,", "    pub(super) bytes: Vec<u8>,", 1)
section = section.replace("struct Reader<'a> {", "pub(super) struct Reader<'a> {", 1)
section = section.replace("    fn ", "    pub(super) fn ")
artifact.write_text(
    text[:start] + '#[path = "artifact/io.rs"]\nmod io;\nuse io::{Reader, Writer};\n\n' + text[end:],
    encoding="utf-8",
)
artifact_dir = ROOT / "src/vm/artifact"
artifact_dir.mkdir(exist_ok=True)
(artifact_dir / "io.rs").write_text("use super::*;\n\n" + section, encoding="utf-8")

# Move compiler terminal helpers to a child module.
compiler = ROOT / "src/vm/compiler.rs"
text = compiler.read_text(encoding="utf-8")
anchor = '#[path = "compiler/functions.rs"]\nmod functions;\n'
text = text.replace(
    anchor,
    anchor + '#[path = "compiler/helpers.rs"]\nmod helpers;\nuse helpers::{constant_form, internal, literal_collection_form, unquote_argument};\n',
    1,
)
start_marker = "fn literal_collection_form(form: &Form) -> bool {"
end_marker = '#[cfg(test)]\n#[path = "compiler/tests.rs"]\nmod tests;'
start = text.index(start_marker)
end = text.index(end_marker, start)
section = text[start:end]
section = section.replace("\nfn ", "\npub(super) fn ")
if section.startswith("fn "):
    section = "pub(super) " + section
compiler.write_text(text[:start] + text[end:], encoding="utf-8")
compiler_dir = ROOT / "src/vm/compiler"
compiler_dir.mkdir(exist_ok=True)
(compiler_dir / "helpers.rs").write_text("use super::*;\n\n" + section, encoding="utf-8")

# Move terminal machine execution and trace-cache helpers to a child module.
machine = ROOT / "src/vm/machine.rs"
text = machine.read_text(encoding="utf-8")
anchor = '#[path = "machine/globals.rs"]\nmod globals;\n'
text = text.replace(
    anchor,
    anchor
    + '#[path = "machine/execution.rs"]\nmod execution;\n'
    + 'use execution::{constant_string, constant_string_vector};\n'
    + 'pub use execution::{execute_program, execute_program_with_globals};\n'
    + '#[cfg(all(test, feature = "tracing-jit"))]\n'
    + 'pub(crate) use execution::{active_compiled_trace_count, active_jit_telemetry, cached_trace_count};\n'
    + '#[cfg(feature = "tracing-jit")]\n'
    + 'pub(crate) use execution::cached_jit_telemetry;\n',
    1,
)
start_marker = "/// Reads a string constant (the global-name operands).\nfn constant_string"
start = text.index(start_marker)
section = text[start:]
section = section.replace("fn constant_string(", "pub(super) fn constant_string(", 1)
section = section.replace("fn constant_string_vector(", "pub(super) fn constant_string_vector(", 1)
machine.write_text(text[:start], encoding="utf-8")
machine_dir = ROOT / "src/vm/machine"
machine_dir.mkdir(exist_ok=True)
(machine_dir / "execution.rs").write_text("use super::*;\n\n" + section, encoding="utf-8")
