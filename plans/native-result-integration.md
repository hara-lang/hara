# Native Result boundary integration

Issue: #641

## Objective

Use native `Result` as Hara's completed boundary outcome without turning it into a promise, computation gateway, aggregate report, optional-value wrapper, or monad. The runtime implementation remains `std.native.Result`; the conventional HAL-facing API is the direct `std.foundation` `res-*` family.

A completed operation has one native `Result[status data error context]`. Nested explanations and aggregate summaries remain ordinary portable data inside or around that Result.

## Invariants

- `status` is exactly `:success` or `:error`.
- Success contains data and no error; error contains a normalized native `:hara/Error`.
- Context is diagnostic and adapter-owned; it does not affect equality or hashing.
- Dereference returns success data or throws the contained native Error.
- Synchronization dereferences once, never flattens nested Results, and owns common timeout/cancellation behavior.
- `:display` is local-only context and is removed before transport. Other nonportable context is rejected.
- Low-level handles, streams, pending promises, progress, lifecycle state, event streams, and aggregate reports remain in their existing forms.
- JavaScript, Python, Lua, Dart, browser, and XTalk APIs retain native promises/envelopes. Native Hara creates Result only after a foreign operation has completed.
- Do not revive `std.lib.result`, `IReturn`, or archived structural wrappers.

## Stack

| Tranche | State | Deliverable |
| --- | --- | --- |
| Native value model | merged | Rust `Value::Result` and Java `HaraResult`, identity, hashing, dereference, and stable display |
| Synchronization and facade | PR #662 | `Result/synchronize`, common timeout/cancellation semantics, direct `std.foundation/res-*` API |
| Transport | in progress | HTA STRUCT reconstruction first; JSON/RPC envelope next |
| Typed comparison | queued | recursive typed failures and one `Result<boolean>` comparison outcome |
| Boundary adapters | queued | substrate, native HTTP, lint, completed process, and per-item work outcomes |
| Remaining completed operations | queued | CLI/deploy/host/compiler/database/package/agent/tool boundaries and Wrapped-style cleanup |

## 1. Native transport

### HTA STRUCT — current tranche

Touchpoints:

- `core/rust/src/core/native_result.rs`
- `core/rust/src/hta.rs`
- `core/java/src/main/java/hara/truffle/HaraResult.java`
- `core/java/src/main/java/hara/truffle/HtaValueCodec.java`
- `core/java/src/test/java/hara/truffle/HtaValueCodecTest.java`
- focused Rust HTA tests in `core/rust/src/hta.rs`

Wire representation:

```text
STRUCT
name   = "hara/Result"
fields = ["status", "data", "error", "context"]
```

The exact name and field order reconstruct a native Result. Any other struct remains generic. No new HTA tag or wire version is introduced. Encoding strips top-level `:display`; all other context must pass the existing portable-value encoder.

Required proof:

- success and error round trips in Rust and Java
- status/data/error/context preservation
- exact-structure recognition and generic fallback for nonexact structs
- `:display` stripping
- explicit failure for another nonportable context value

### JSON and RPC projection — next transport tranche

Touchpoints:

- `core/rust/src/json.rs`
- `core/java/src/main/java/hara/truffle/StdJson.java`
- `core/java/src/test/java/hara/truffle/StdJsonTest.java`
- native HTTP/RPC adapters discovered under the HTTP boundary audit

Envelope:

```json
{
  "$hara": "result",
  "status": "success",
  "data": {},
  "error": null,
  "context": {}
}
```

No wire version is included. The tranche must define or reuse one canonical native Error JSON projection rather than inventing adapter-specific error shapes.

## 2. Typed checking and test comparison

Touchpoints:

- `core/lib/src/std/typed/schema.hal`
- `core/lib/src/std/typed/infer.hal`
- remaining `core/lib/src/std/typed/**` checker and explanation code
- public source conversion under `core/lib/src/tool/lint/**`
- Rust native Test implementation in `core/rust/src/core.rs`
- Java native Test implementation in `core/java/src/main/java/hara/truffle/HaraContext.java`
- `core/lib/src/code/test/**`, including checker, evaluation, executive, and process plumbing
- canonical and generated HAL mirrors affected by public Foundation/Test definitions

Current audit invariant: the active `std.typed` tree has no direct `std.block` import. Preserve that hard boundary; source-block conversion belongs in the lint adapter.

Deliverables:

- `Test/compare` returns one `Result<boolean>`
- ordinary mismatch is `Result/success false`, never `Result/error`
- checker crashes, timeouts, or unexpected evaluation failures are `Result/error`
- throws checkers consume a captured native Error as actual input
- strict recursive portable Failure maps with every required field
- deterministic depth-first leaf traversal and leaf count
- `Test/result` enriches an existing comparison Result without recomputation
- aggregate facts, namespaces, runs, events, and summaries remain ordinary maps containing comparison Results

## 3. Substrate and native HTTP

### Substrate

Touchpoints:

- `core/lib/src/std/substrate.hal`
- `core/lib/src/std/substrate/**`
- `core/lib/src-lang/xt/substrate/**`
- `core/spec/std/substrate-parity.edn`
- `core/spec/xt/substrate-parity.edn`

A reply frame becomes the Result itself. Correlation and delivery metadata live under `:context :substrate`. Unknown actions, authorization failures, validation failures, handler failures, malformed replies, closed transport, and local timeout become error Results. Pending promises remain internal.

### Native HTTP RPC

Touchpoints:

- native Hara HTTP/RPC adapters identified by the HTTP audit
- `core/lib/src-lang/xt/net/http_fetch.hal`
- `core/lib/src-lang/xt/net/http_util.hal`
- corresponding XTalk tests and parity specifications

A delivered application error remains a valid Result envelope, normally HTTP 2xx with `application/vnd.hara.result+json`. HTTP/proxy/decode/gateway failure before an envelope exists becomes a local transport error Result. HTTP context is allowlisted and must exclude credentials and authorization headers. `xt.net.http-fetch` itself remains Promise/envelope based.

## 4. Lint, process, and work

### Lint

Touchpoints:

- `core/lib/src/tool/lint.hal`
- `core/lib/src/tool/lint/**`
- `core/lib/src/tool/cli/lint.hal`

Internal findings remain ordinary data. Public lint entry points return `Result<report>`. Error-severity findings are still a successful completed report; parser/analyzer/linter crashes are error Results. Display and source provenance belong in context.

### Completed process commands

Touchpoints:

- low-level `std.native.Process` implementations in Rust and Java
- `core/lib/src/tool/sh.hal` and shell command adapters
- `core/lib/src/code/test/**` process timeout compatibility code

Handles, streams, stdout/stderr access, and waiting remain Promise based. The completed command interface uses `Result/synchronize`. Exit zero returns a success Result containing the output record. Nonzero exit returns an error Result retaining argv, status, stdout, and stderr in Error data and namespaced process context. The common timeout path replaces `TimeoutValue`-style compatibility state.

### Work and build

Touchpoints:

- `core/lib/src/std/work.hal`
- `core/lib/src/std/work/command.hal`
- `core/lib/src/std/work/protocol.hal`
- `core/lib/src/std/work/report.hal`
- `core/lib/src/std/work/runtime.hal`
- `core/lib/src/std/work/runtime/**`
- `core/lib/src/std/work/provider/**`
- work command, CLI, task, selector, recipe, template, and report adapters

Per-item completed outcomes become Results. Warnings remain successful data with severity in context. Batch summaries, progress, lifecycle state, and event streams remain ordinary aggregate structures.

## 5. Additional completed boundaries

Audit and migrate outcome-oriented completion surfaces where Result's dereference and contextual display semantics are appropriate:

- REPL presentation and completed shell/tool results
- pointer, host, and extension invocation completion
- completed CLI and deploy operations
- parser/compiler/emitter public entry points
- database RPC and service commands
- package steps
- agent/tool execution
- Wrapped-style outcome presentation

Do not replace optional or missing values, structural zipper/sentinel values, mutable state, progress, event streams, or pure internal helper return values.

## Sequencing

1. Land PR #662.
2. Land exact HTA STRUCT round-trip support.
3. Add canonical JSON/Error projection and native Result RPC envelope.
4. Implement typed Failure data and native Test `Result<boolean>` APIs.
5. Migrate code.test and remove superseded timeout/result-map compatibility.
6. Adapt substrate and native HTTP boundaries.
7. Adapt lint, completed process commands, and per-item work outcomes.
8. Audit remaining completed CLI/deploy/host/compiler/database/package/agent/tool surfaces.
9. Refresh generated HAL mirrors and run focused plus relevant full suites at every tranche.

## Validation gates

Every tranche must include runtime parity tests and explicit negative cases. Every `.hal` edit follows the repository development gate: evaluate the full candidate in a fresh native process, run the focused test before writing, write the source, evaluate the written file in another fresh process, and rerun the focused test. Generated source mirrors must remain byte-for-byte aligned where required.
