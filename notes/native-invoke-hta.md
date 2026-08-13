# Binary-safe qualified-Var invocation

Native embedding hosts may invoke one already-loaded, fully qualified Hara Var
without constructing or evaluating source text:

```rust
Runtime::invoke_hta(qualified_var, arguments_hta)
RuntimeBroker::invoke_hta(session, qualified_var, arguments_hta)
```

The input is one canonical HTA0 vector of arguments. The result is one canonical
HTA0 value bounded to 256 KiB. The boundary rejects unqualified or missing Vars,
non-callable Vars, malformed or noncanonical input, unsupported result values,
rejected promises, and oversized results with stable typed errors.

Var resolution is direct against the prepared namespace registry. The method does
not call the parser, macroexpander, compiler, namespace loader, or source evaluator.
Native capability providers and the reviewed host callback remain active while the
already-loaded function executes. Downstream hosts must apply their own closed Var
allowlist before calling this API; it is not a general IPC evaluation surface.

Run the focused benchmark with:

```text
cargo run --manifest-path core/rust/Cargo.toml --release \
  --bin hara-invoke-hta-benchmark
```
