from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one marker, found {count}")
    return text.replace(old, new, 1)


def checkout_fixture(repository: str, revision: str, destination: Path) -> None:
    """Materialize a pinned public fixture repository for native tests."""
    if destination.exists():
        return
    destination.mkdir(parents=True)
    subprocess.run(["git", "-C", str(destination), "init"], check=True)
    subprocess.run(
        [
            "git",
            "-C",
            str(destination),
            "remote",
            "add",
            "origin",
            f"https://github.com/{repository}.git",
        ],
        check=True,
    )
    subprocess.run(
        [
            "git",
            "-C",
            str(destination),
            "fetch",
            "--depth",
            "1",
            "origin",
            revision,
        ],
        check=True,
    )
    subprocess.run(
        ["git", "-C", str(destination), "checkout", "--detach", "FETCH_HEAD"],
        check=True,
    )


def link_fixture(source: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.is_symlink():
        destination.unlink()
    elif destination.exists():
        raise SystemExit(f"fixture destination already exists: {destination}")
    destination.symlink_to(source, target_is_directory=True)


# Compiler bindings split.
globals_path = ROOT / "core/rust/src/vm/compiler/globals.rs"
globals_source = globals_path.read_text()
globals_source = replace_once(
    globals_source,
    "    fn require_owned_global(&self, _name: &str, _span: &Span) -> Result<(), CompileError> {",
    "    pub(super) fn require_owned_global(&self, _name: &str, _span: &Span) -> Result<(), CompileError> {",
    "owned-global visibility",
)
globals_source = replace_once(
    globals_source,
    "    fn var_metadata(&mut self, metadata: Option<Rc<Metadata>>) -> Option<u16> {",
    "    pub(super) fn var_metadata(&mut self, metadata: Option<Rc<Metadata>>) -> Option<u16> {",
    "var-metadata visibility",
)
globals_source = replace_once(
    globals_source,
    '            .map_err(|message| unsupported(format!("{message}"), children[1].span.start))?;',
    '            .map_err(|message| unsupported(message, children[1].span.start))?;',
    "globals useless format",
)
bindings_start = globals_source.index(
    "    /// `(def name init)`: interns the value in the current namespace and\n"
)
bindings_end = globals_source.index(
    "    /// `(declare name ...)`: interns a nil var per name without resetting\n"
)
bindings_methods = globals_source[bindings_start:bindings_end].rstrip() + "\n"
globals_path.write_text(globals_source[:bindings_start] + globals_source[bindings_end:])

bindings_source = """//! Compilation of immutable and mutable global bindings.
//!
//! This child owns `def`, `set!`, and `var`; namespace declaration,
//! callable publication, structures, fields, and functions remain in
//! `globals.rs`. The split is structural only and preserves the emitted
//! instruction sequences.

use crate::core::binding_symbol;
use crate::kernel::{Form, Span};
use crate::vm::error::{CompileError, CompileErrorKind};
use crate::vm::opcode::Instruction;

use super::{Child, Compiler};

impl Compiler {
""" + bindings_methods + """}

fn unsupported(message: impl Into<String>, position: crate::kernel::Position) -> CompileError {
    CompileError::new(
        CompileErrorKind::UnsupportedForm,
        message.into(),
        Some(position),
    )
}
"""
(ROOT / "core/rust/src/vm/compiler/bindings.rs").write_text(bindings_source)

compiler_path = ROOT / "core/rust/src/vm/compiler.rs"
compiler_source = compiler_path.read_text()
compiler_marker = '#[path = "compiler/calls.rs"]\nmod calls;\n'
compiler_path.write_text(
    replace_once(
        compiler_source,
        compiler_marker,
        '#[path = "compiler/bindings.rs"]\nmod bindings;\n' + compiler_marker,
        "compiler child module",
    )
)

# Machine dispatch and constant-decoding split.
machine_path = ROOT / "core/rust/src/vm/machine.rs"
machine_source = machine_path.read_text()
machine_source = replace_once(
    machine_source,
    "                    let mut next_ip = ip;\n",
    '                    #[cfg(feature = "tracing-jit")]\n'
    "                    let mut next_ip = ip;\n"
    '                    #[cfg(not(feature = "tracing-jit"))]\n'
    "                    let next_ip = ip;\n",
    "feature-correct next-ip mutability",
)

enum_start = machine_source.index(
    "/// Result of executing one instruction. Call actions only carry their\n"
)
enum_end = machine_source.index(
    "/// A synchronous interpreter for one function of a validated [`Program`].\n"
)
dispatch_enum = machine_source[enum_start:enum_end].rstrip() + "\n"
dispatch_enum = replace_once(
    dispatch_enum,
    "enum Dispatch {",
    "pub(super) enum Dispatch {",
    "dispatch visibility",
)
machine_source = machine_source[:enum_start] + machine_source[enum_end:]

dispatch_start = machine_source.index(
    "    /// Executes one instruction, returning where the `run` loop\n"
)
dispatch_end = machine_source.index(
    "    /// Pops the callee and arguments for a Call instruction.\n"
)
dispatch_method = machine_source[dispatch_start:dispatch_end].rstrip() + "\n"
dispatch_method = replace_once(
    dispatch_method,
    "    fn dispatch(\n",
    "    pub(super) fn dispatch(\n",
    "dispatch method visibility",
)
machine_source = machine_source[:dispatch_start] + machine_source[dispatch_end:]

constants_start = machine_source.index(
    "/// Reads a string constant (the global-name operands).\n"
)
constants_end = machine_source.index(
    "fn run_entry(program: Rc<Program>) -> Result<Value, VmError> {\n"
)
constants_methods = machine_source[constants_start:constants_end].rstrip() + "\n"
constants_methods = replace_once(
    constants_methods,
    "fn constant_string(",
    "pub(super) fn constant_string(",
    "string constant visibility",
)
constants_methods = replace_once(
    constants_methods,
    "fn constant_string_vector(",
    "pub(super) fn constant_string_vector(",
    "string-vector constant visibility",
)
machine_source = machine_source[:constants_start] + machine_source[constants_end:]

module_marker = (
    '#[path = "machine/coroutine_runtime.rs"]\n'
    "mod coroutine_runtime;\n"
    '#[path = "machine/globals.rs"]\n'
    "mod globals;\n"
)
module_replacement = (
    '#[path = "machine/coroutine_runtime.rs"]\n'
    "mod coroutine_runtime;\n"
    '#[path = "machine/constants.rs"]\n'
    "mod constants;\n"
    "use constants::{constant_string, constant_string_vector};\n"
    '#[path = "machine/dispatch.rs"]\n'
    "mod dispatch;\n"
    "use dispatch::Dispatch;\n"
    '#[path = "machine/globals.rs"]\n'
    "mod globals;\n"
)
machine_path.write_text(
    replace_once(
        machine_source,
        module_marker,
        module_replacement,
        "machine child modules",
    )
)

constants_source = """//! Decoding of typed VM constant-pool operands.

use super::Program;
use crate::core::Value;

""" + constants_methods
(ROOT / "core/rust/src/vm/machine/constants.rs").write_text(constants_source)

dispatch_source = """//! Single-instruction VM dispatch.
//!
//! The outer `Machine::run` loop remains in `machine.rs`; this child only
//! decodes and executes one validated instruction and returns the next
//! control action.

use super::*;

""" + dispatch_enum + "\nimpl Machine {\n" + dispatch_method + "}\n"
(ROOT / "core/rust/src/vm/machine/dispatch.rs").write_text(dispatch_source)

# Match Core CI's external fixture layout. These repositories are transport-only
# inputs and are not staged into the clean product commit.
specs_checkout = ROOT / "hara-specs-registry"
checkout_fixture("hara-lang/hara-specs-registry", "main", specs_checkout)
link_fixture(specs_checkout, ROOT.parent / "hara-specs-registry")
link_fixture(specs_checkout, Path("/tmp/hara-specs-registry"))

benchmarks_checkout = ROOT / "hara-benchmarks"
checkout_fixture(
    "hara-lang/hara-benchmarks",
    "05234295ac1c706eb1adee505873b10783d42163",
    benchmarks_checkout,
)
link_fixture(
    benchmarks_checkout,
    ROOT.parent.parent / "website/hara-benchmarks",
)

print("applied the mechanical #562 compiler and machine layout split")
