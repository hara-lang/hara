# Bytecode VM milestone 4 closeout

Issue: `hara-lang/hara#223`

This note records the implementation boundary and verification expected before
closing the globals, namespaces, and multi-arity milestone.

## Delivered surface

The Rust bytecode VM now provides the following through the public `Runtime`
entry points:

- registry-direct global reads and writes through `def`, `var`, and `set!`;
- real late-bound Vars for `defn`, including same-unit forward discovery and
  redefinition through the shared namespace registry;
- fixed, variadic, and named multi-arity function dispatch;
- destructuring expansion for `let` and `loop` bindings;
- persistent `defstruct` and reference-identity `defmutable` named values,
  including same-unit `Name`, `->Name`, and `map->Name` constructor visibility;
- mutable field reads and field-place `set!`;
- macro definitions and compile-time macro expansion;
- direct compilation of the Foundation source body into a validated HBC0
  artifact.

## Namespace boundary decision

`ns`, `in-ns`, `require`, aliasing, referring, and module loading remain
runtime/loader responsibilities. They are intentionally not represented as VM
instructions. A host selects and prepares the namespace before compiling a
source unit or lowers a HALC module after applying its namespace declaration.
The VM then resolves global operands through the prepared
`NamespaceRegistry`.

This keeps module discovery, loading failure retention, reload policy, and
project configuration outside process-local bytecode.

## Arity boundary decision

Named multi-arity `defn` is supported. Anonymous multi-arity `fn` remains a
compile error because it is not part of the current portable evaluator
surface. Anonymous variadic `fn` is supported. This is a language-surface
boundary, not a missing dispatcher capability.

## Closeout evidence

`core/rust/tests/vm_milestone_4.rs` exercises the milestone through public
runtime methods rather than private compiler helpers. It covers globals,
late-bound Vars, named multi-arity dispatch, variadic closures, destructuring,
named values, mutable fields, and the explicit namespace host boundary.

The existing VM unit and corpus suites additionally cover:

- namespace-owned Var protection and declaration semantics;
- exception interactions with named values;
- Foundation bytecode compilation and loading;
- HBC0 validation and round trips;
- differential behavior against `Runtime::eval_native`.

Recommended closeout commands:

```sh
cargo test --manifest-path core/rust/Cargo.toml --test vm_milestone_4
cargo test --manifest-path core/rust/Cargo.toml --lib vm::
cargo build --manifest-path core/rust/Cargo.toml --target wasm32-unknown-unknown --lib
```

The milestone does not make the bytecode VM the universal evaluator and does
not move module loading into bytecode. Suspension and resumability remain
tracked separately by issue #204.
