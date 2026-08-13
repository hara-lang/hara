#!/usr/bin/env python3
"""Split persistent named-value code out of oversized Rust modules."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()

EXPECTED_BLOBS = {
    "core/rust/src/hta.rs": "5d270ddf5d25fa9af43fc23fa9144201d0820b91",
    "core/rust/src/vm/compiler.rs": "75b1e027f660b96a02f3115bfc26d54b66cd7ba5",
    "core/rust/src/vm/compiler/globals.rs": "60835bbbdc08cb5a8ba902d25cd00be0188c2763",
    "core/rust/src/vm/execution_tests.rs": "bbbb99db2378cf95a3e33135ce169e35f99a0e7d",
    "core/rust/src/vm/machine.rs": "f5def7bfbc4730662ca8af0bd7c52ac5f47fa1fb",
    "core/rust/scripts/layout-baseline.txt": "aeb1a5a25e3e994cb968115bf3de3c3d6f8fd637",
}


def read(relative: str) -> str:
    return (ROOT / relative).read_text()


def write(relative: str, text: str) -> None:
    path = ROOT / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


def blob(relative: str) -> str:
    return subprocess.check_output(
        ["git", "-C", str(ROOT), "hash-object", relative], text=True
    ).strip()


for path, expected in EXPECTED_BLOBS.items():
    actual = blob(path)
    if actual != expected:
        raise SystemExit(f"unexpected source for {path}: {actual}, expected {expected}")

# HTA codec tests belong in a test-only child module.
hta_path = "core/rust/src/hta.rs"
hta = read(hta_path).replace(
    "use crate::core::Value;\n#[cfg(test)]\n"
    "use crate::lang::data::{Tuple as PTuple, Vector as PVector};\n",
    "use crate::core::Value;\n",
    1,
)
marker = "\n#[cfg(test)]\nmod tests {\n"
start = hta.index(marker)
body = hta[start + len(marker) :]
if not body.endswith("}\n") or not body.startswith("    use super::*;\n"):
    raise SystemExit("unexpected HTA test module shape")
body = body[len("    use super::*;\n") : -2]
dedented = "".join(
    line[4:] if line.startswith("    ") else line
    for line in body.splitlines(keepends=True)
)
write(
    "core/rust/src/hta/tests.rs",
    "use super::*;\n"
    "use crate::lang::data::{Tuple as PTuple, Vector as PVector};\n\n"
    + dedented,
)
write(hta_path, hta[:start] + '\n#[cfg(test)]\n#[path = "hta/tests.rs"]\nmod tests;\n')

# Compiler lowering for named definitions and mutable fields gets its own module.
globals_path = "core/rust/src/vm/compiler/globals.rs"
globals = read(globals_path)
start_marker = "    /// Defines one immutable or mutable named-value family, its parallel\n"
end_marker = "    /// `(defn name ...)` / `(defn- name ...)`: interns a real var holding\n"
start = globals.index(start_marker)
end = globals.index(end_marker, start)
methods = globals[start:end]
write(globals_path, globals[:start] + globals[end:])
write(
    "core/rust/src/vm/compiler/named_values.rs",
    "use crate::kernel::{Form, Position, Span};\n\n"
    "use super::{Child, Compiler};\n"
    "use crate::vm::error::{CompileError, CompileErrorKind};\n"
    "use crate::vm::opcode::Instruction;\n\n"
    "impl Compiler {\n"
    + methods
    + "}\n\n"
    "fn unsupported(message: impl Into<String>, position: Position) -> CompileError {\n"
    "    CompileError::new(\n"
    "        CompileErrorKind::UnsupportedForm,\n"
    "        message.into(),\n"
    "        Some(position),\n"
    "    )\n"
    "}\n",
)
compiler_path = "core/rust/src/vm/compiler.rs"
compiler = read(compiler_path)
needle = '#[path = "compiler/globals.rs"]\nmod globals;\n'
write(
    compiler_path,
    compiler.replace(
        needle,
        needle + '#[path = "compiler/named_values.rs"]\nmod named_values;\n',
        1,
    ),
)

# Keep named-value end-to-end tests together without growing the legacy test module.
execution_path = "core/rust/src/vm/execution_tests.rs"
execution = read(execution_path)
start_marker = "#[test]\nfn defstruct_forms_issue_223() {"
end_marker = "#[test]\nfn variadic_and_multi_arity_issue_223() {"
start = execution.index(start_marker)
end = execution.index(end_marker, start)
tests = execution[start:end].rstrip() + "\n"
execution = execution[:start] + execution[end:]
needle = '#[path = "execution_tests/bindings.rs"]\nmod bindings;\n'
write(
    execution_path,
    execution.replace(
        needle,
        needle + '#[path = "execution_tests/named_values.rs"]\nmod named_values;\n',
        1,
    ),
)
write(
    "core/rust/src/vm/execution_tests/named_values.rs",
    "use super::eval;\n\n" + tests,
)

# Route named-value instructions through a focused machine extension.
machine_path = "core/rust/src/vm/machine.rs"
machine = read(machine_path)
needle = '#[path = "machine/globals.rs"]\nmod globals;\n'
machine = machine.replace(
    needle,
    needle + '#[path = "machine/named_values.rs"]\nmod named_values;\n',
    1,
)
old_dispatch = """            Instruction::DefStruct { name, fields } => {
                guarded!(self.exec_def_struct(program, *name, *fields));
            }
            Instruction::DefMutable { name, fields } => {
                guarded!(self.exec_def_mutable(program, *name, *fields));
            }
            Instruction::MutableFieldGet(index) => {
                guarded!(self.exec_mutable_field_get(program, *index));
            }
            Instruction::MutableFieldSet(index) => {
                guarded!(self.exec_mutable_field_set(program, *index));
            }
            Instruction::InstanceOf => {
                guarded!(self.exec_instance_of());
            }
"""
new_dispatch = """            Instruction::DefStruct { .. }
            | Instruction::DefMutable { .. }
            | Instruction::MutableFieldGet(_)
            | Instruction::MutableFieldSet(_)
            | Instruction::InstanceOf => {
                guarded!(self.exec_named_value_instruction(program, instruction));
            }
"""
if old_dispatch not in machine:
    raise SystemExit("unexpected named-value machine dispatch shape")
machine = machine.replace(old_dispatch, new_dispatch, 1)
machine = machine.replace(
    "pub mod observation;\n\n/// Terminal state of a machine run.",
    "pub mod observation;\n/// Terminal state of a machine run.",
    1,
)
write(machine_path, machine)
write(
    "core/rust/src/vm/machine/named_values.rs",
    """use super::Machine;
use crate::vm::opcode::Instruction;
use crate::vm::program::Program;

impl Machine {
    pub(super) fn exec_named_value_instruction(
        &mut self,
        program: &Program,
        instruction: &Instruction,
    ) -> Result<(), String> {
        match instruction {
            Instruction::DefStruct { name, fields } => {
                self.exec_def_struct(program, *name, *fields)
            }
            Instruction::DefMutable { name, fields } => {
                self.exec_def_mutable(program, *name, *fields)
            }
            Instruction::MutableFieldGet(index) => {
                self.exec_mutable_field_get(program, *index)
            }
            Instruction::MutableFieldSet(index) => {
                self.exec_mutable_field_set(program, *index)
            }
            Instruction::InstanceOf => self.exec_instance_of(),
            _ => unreachable!("named-value instruction dispatch"),
        }
    }
}
""",
)

baseline_path = "core/rust/scripts/layout-baseline.txt"
baseline = read(baseline_path)
entry = "lines src/vm/compiler/globals.rs 721\n"
if entry not in baseline:
    raise SystemExit("compiler globals layout baseline was already changed")
write(baseline_path, baseline.replace(entry, "", 1))
