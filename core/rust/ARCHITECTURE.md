# Rust runtime architecture

The Rust runtime follows the responsibility boundaries of the Java runtime
without copying its class hierarchy. The public Rust surface remains flat:
existing paths such as `hara_wasm::Runtime` and `hara_wasm::core::Value`
continue to work.

## Compatibility facades

`src/core.rs` and `src/lib.rs` are stable entry points. They declare the
runtime's dependencies and include responsibility-focused implementation
fragments:

- `src/core/` contains values, environment and protocol state, native
  operations and providers, asynchronous values, primitives, forms, namespace
  loading, and evaluation.
- `src/runtime/` contains the embedding model, sessions, runtime bootstrap,
  bytecode integration, native evaluation, WebAssembly bridges, and runtime
  tests.

The fragments use `include!` deliberately. They are compiled in their
facade's module rather than as nested Rust modules, preserving visibility,
symbol paths, macro scope, and the direct inclusion of `core.rs` by the raw
runtime crates. This makes the reorganization structural rather than
behavioral.

Dependencies point inward: embedding and runtime code may use core facilities;
core facilities do not depend on the embedding facade. Experimental bytecode
and WebAssembly adapters stay at the runtime boundary.

## Live execution and compiler products

`LIVE_SESSION.md` defines the backend-neutral live-session contract, its
ownership beneath Sandbox-private Sessions, source replacement semantics, and
the distinction between HBC, whole-Wasm, runtime-host Wasm, and extension Wasm
products. New interactive execution and compiler-target work must preserve
those boundaries rather than adding evaluator behaviour to `Sandbox`.

## Layout policy

Most Rust files remain subject to the repository's line-count gate. The
`core` and `runtime` facade trees are exempt because their first constraint
is API and raw-crate compatibility. Their files are grouped by responsibility,
and can be converted into encapsulated modules later as those compatibility
constraints are retired.

New module trees should use `module.rs` with `module/*.rs`; do not add new
`mod.rs` files.
