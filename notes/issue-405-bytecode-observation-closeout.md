# Bytecode observation closeout

Issue: `hara-lang/hara#405`

This note records the final contract decisions for the production bytecode
observation surface.

## Feature boundary

The main `hara-wasm` crate enables `bytecode-vm` in its default feature set.
That does not make observation ambient: `bytecode-observation` remains a
separate opt-in feature, and the ordinary `Machine::run` path remains unchanged
when observation is not enabled.

Compiler-free deployments continue to consume validated HBC artifacts through
the dedicated `vm-runtime` crate without carrying the source compiler.
Feature-minimal crates may omit `bytecode-vm` entirely.

## Global-state boundary

The executable `Machine` does not own semantic globals. Global Vars, namespace
identity, dynamic metadata, and redefinition state live in the shared
`NamespaceRegistry`. Keeping that state out of `MachineSnapshot` preserves the
registry-neutral machine and the compiler-free VM boundary.

The public `BytecodeObservationSession::snapshot_value` projection now joins
the machine snapshot with a deterministic, bounded semantic global projection:

- only Vars owned by the current namespace are included;
- referred Foundation Vars are excluded rather than duplicated;
- bindings are sorted by qualified symbol;
- values use the same bounded display contract as machine slots;
- dynamic, macro, and origin metadata are scalar evidence;
- at most 64 bindings are retained and the omitted count is explicit.

The browser session and raw Wasm observation facade already consume
`snapshot_value`, so the same projection crosses those boundaries without
exposing executable Vars, namespace handles, or host identities.

## Pre-machine failures

Compilation, HALC lowering, artifact decoding, and program validation happen
before a live machine exists. They therefore remain outside the instruction
stepping contract in this issue. Typed production handoff evidence belongs to
#404, while normalization across Rust, JVM, and Wasm startup failures belongs
to the shared corpus and report in #406.

This separation keeps #405 focused on observable execution after the existing
validation safety gate and avoids inventing a synthetic machine for a program
that was never admitted.

## Closeout evidence

Focused session tests cover deterministic ordering, current-namespace
ownership, truncation, omitted counts, bounded value display, and exclusion of
referred Foundation Vars.

Recommended checks:

```sh
cargo test --manifest-path core/rust/Cargo.toml \
  --features bytecode-observation vm::session:: --lib
cargo clippy --manifest-path core/rust/Cargo.toml \
  --features bytecode-observation --lib --no-deps
cargo build --manifest-path core/rust/Cargo.toml \
  --target wasm32-unknown-unknown --features bytecode-observation --lib
```
