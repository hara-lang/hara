# Staged Rust bytecode VM: design closeout and go/no-go decision

Issue: `hara-lang/hara#195`

Status: architecture decision complete; implementation and promotion continue in focused issues.

## Decision

### Go

Continue the Rust bytecode VM as an explicit, opt-in execution tier and as the
prepared-execution substrate for Rust-native embedding, HBC0 artifacts,
tracing, and whole-function compilation.

The staged design has been validated beyond the original proof point:

- typed bytecode and abstract-interpretation validation;
- numeric lexical slots and an explicit guest frame stack;
- closures, by-value captures, recursion, variadic functions, named
  multi-arity dispatch, and destructuring;
- registry-direct global Vars, namespace-owned mutation, macros, protocols,
  multimethods, persistent `defstruct`, and reference-identity `defmutable`;
- static exception tables with cross-frame catch/finally unwind;
- process-local suspension, `await`, `yield`, async result Promises, host
  Promise interop, and cancellation propagation;
- checksummed HBC0 persistence and HBX standard-library bundles;
- optional checked tracing, native tracing, and whole-function Wasm tiers;
- public embedding APIs that can compile once and return immutable Hara
  values directly.

### No-go for default replacement today

Do **not** make bytecode the universal source evaluator, silently route
unsupported source to the tree evaluator, or retire `EvalFiber` and the
existing evaluator yet.

Those are separate promotion decisions. They require complete supported-
language conformance, source/HALC/HBC/native/browser/module parity, an explicit
fallback policy, and evidence that default startup, diagnostics, dynamic
forms, host capabilities, and coroutine behavior remain correct. The current
feature gates are intentional product boundaries, not unfinished architecture.

## Acceptance-criteria mapping

| Original criterion | Delivered evidence |
| --- | --- |
| Written VM specification covering instructions, frames, closures, globals, calls, exceptions, suspension, source maps, validation, and tracing | `hara-specs-registry/01-lang/010-bytecode/draft/hal-bytecode-vm.edn`, its suspension extension, the machine-checked corpora, and the working implementation note `notes/rust-bytecode-vm.md` |
| Representative compiled listings | Deterministic `HbcDisassembler`, disassembly unit tests, compiler execution tests, and decomposed bytecode benchmark programs |
| Native and Wasm memory, code-size, and runtime costs | The milestone benchmark records, bytecode/trace executable-size measurements, native and browser build lanes, and the reproducible benchmark coordinators under `core/lib/bench` and `core/rust` |
| Differential, malformed-bytecode, conformance, and benchmark matrices | VM differential tests, structural validator tests, HBC0 invalid-artifact tests, language and VM conformance corpora, startup and prepared-execution benchmark lanes |
| Explicit staged implementation issues | Closures `#202`, exceptions `#203`, globals/namespaces/arity `#223`, suspension `#204`, execution performance `#226`/`#253`, and whole-function compilation `#256` |
| Explicit go/no-go decision before default evaluator replacement | This document: go for continued opt-in VM development; no-go for universal default replacement or evaluator retirement today |

## Final architecture boundary

```text
HAL source or HALC
  -> namespace/module preparation at the Runtime boundary
  -> macro expansion and name resolution
  -> bytecode compilation
  -> structural validation
  -> Program
       constants
       normalized schema facts
       function prototypes
       numeric locals/captures
       explicit call frames
       source positions
       static exception handlers
  -> Machine / VmFiber
       Return | Error | Suspend | Yield
  -> immutable Value or async result Promise
```

### Portable contracts

- HAL source semantics, language conformance, schemas, HALC, HTA, host
  capabilities, and Evaluation Journal events remain shared contracts.
- Namespace syntax and module loading remain Runtime/loader responsibilities;
  the VM does not encode project discovery or module I/O as instructions.
- Host Promise adapters settle the shared Hara Promise value. JavaScript or
  native callback identity is never persisted in HBC0.

### Rust-private implementation contracts

- `Program`, typed opcodes, numeric slot layouts, handler tables, explicit
  frames, HBC0, tracing IR, JIT caches, and whole-function MIR are Rust
  implementation details.
- A parked `VmFiber` is process-local state. It may retain Vars, Promises,
  provider handles, scheduler references, and JIT state; it is not a durable
  or transferable continuation artifact.
- Checked and native optimization tiers must retain guarded fallback to exact
  dynamic Hara semantics.

## Stage assessment

1. **Program model, validation, source maps, and disassembly — complete.**
2. **Synchronous language core and ordinary calls — complete.**
3. **Closures, exceptions, globals, namespaces, arity, named values, and
   destructuring — complete for the current supported VM surface.**
4. **Promises, await, yield, cancellation, and resumable machine state —
   implemented and specified as process-local execution.**
5. **Differential and benchmark evidence — established and continuously
   extended.** Full-language promotion remains a release gate rather than a
   prerequisite for closing this design issue.
6. **Switch synchronous source execution to bytecode — not authorized by this
   decision.** Track as a dedicated promotion issue when the gates below are
   satisfied.
7. **Retire duplicate evaluator paths — explicitly deferred.** The tree
   evaluator remains the compatibility and bootstrap path.

The original issue was an architecture and sequencing decision, not a promise
to complete stages 6 and 7 inside the parent ticket. Keeping it open until
runtime retirement would obscure ownership now that focused implementation
and performance issues exist.

## Promotion gates that remain outside #195

A future default-on proposal must attach evidence for all of the following:

1. the full language corpus supported by the proposed default path, including
   namespace, reload, macro, protocol, named-value, error, and coroutine cases;
2. source, HALC, HBC0, native, Wasm/browser, extension, and host-capability
   parity for that surface;
3. an explicit unsupported-form and dynamic-code policy with no accidental
   per-expression evaluator mixing;
4. cold startup, warm compilation, prepared execution, allocation, retained
   state, and artifact-size evidence;
5. browser host-Promise and cancellation integration, not merely a Wasm build;
6. diagnostics and Evaluation Journal parity;
7. a rollback-compatible release plan before removing any evaluator path.

## Verification entry points

Focused architecture and runtime checks include:

```sh
cargo test --manifest-path core/rust/Cargo.toml --features bytecode-vm --lib vm::
cargo test --manifest-path core/rust/Cargo.toml --test vm_milestone_4
cargo test --manifest-path core/rust/Cargo.toml --test vm_suspension
cargo build --manifest-path core/rust/Cargo.toml \
  --target wasm32-unknown-unknown --features bytecode-vm --lib
```

Artifact, tracing, whole-Wasm, Java HALC, and shared conformance workflows add
the cross-tier evidence. Broad-suite exclusions must be reported against the
current `main` baseline rather than hidden in a design closeout.

## Closure action

After this note lands, close #195 as completed and leave the feature gates in
place. New default-evaluator or evaluator-retirement work should begin with a
fresh issue whose acceptance criteria are the promotion gates above, not by
reopening the staged architecture decision.
