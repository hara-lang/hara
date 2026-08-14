# Production code.vm conformance corpus

Issue: `hara-lang/hara#406`

The `code-vm-conformance` feature generates one terminal-neutral report from
three production execution paths:

- `Runtime::eval_native_journal` for interpreter outcomes and Evaluation
  Journal events;
- `trace_halc_source_to_bytecode` for source, schema, envelope, decode, and
  bytecode-handoff evidence;
- `BytecodeObservationSession` for bounded instruction-level execution.

The checked-in EDN corpus records upstream specs-registry case identities,
source text, expected outcomes, execution limits, trace retention, and
browser-safe fixtures. Unsupported bytecode forms are typed compile failures
and retain `handoff/fallback false`; no stage silently substitutes the
interpreter.

The JSON report schema is `hal.code-vm-conformance-runtime/0-alpha`. It carries
full bounded traces, normalized checks, deterministic teaching annotations,
and an explicit runtime matrix. Rust is executed by the stable CLI, the same
library path is compiled for `wasm32-unknown-unknown`, and Truffle is reported
as unsupported until its production runner is wired.

Commands:

```sh
cargo run --manifest-path core/rust/Cargo.toml \
  --features code-vm-conformance \
  --bin hara-code-vm-conformance -- check

cargo run --manifest-path core/rust/Cargo.toml \
  --features code-vm-conformance \
  --bin hara-code-vm-conformance -- report

cargo run --manifest-path core/rust/Cargo.toml \
  --features code-vm-conformance \
  --bin hara-code-vm-conformance -- browser
```
