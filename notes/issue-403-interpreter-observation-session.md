# Isolated browser interpreter observation session

Issue: #403

This slice owns the production live evaluator behind an on-demand raw-Wasm ABI.
It follows the existing bytecode observation runtime pattern while retaining a
real `EvalFiber`; it does not compile to bytecode or replay an Evaluation
Journal.

## Ownership

Each opaque handle owns:

- one isolated namespace registry and protocol environment;
- one real observed `EvalFiber` and any pending promise identity;
- source/session identity and reset generation;
- caller-selected binding and display projection limits;
- bounded before/after boundary history with explicit dropped counts.

The module is a standalone crate outside the main runtime workspace. Ordinary
Hara evaluation therefore carries no interpreter-session registry, history, or
raw JSON ABI cost.

## Raw operations

The JSON request ABI provides:

```text
start
info
snapshot
step
run
resume
resolve-suspension
reject-suspension
suspension-state
history
reset
cancel
set-observation-limits
set-retention-limits
result-display
error-message
dispose
dispose-all
```

Each `step` executes or publishes at most one authoritative live boundary. A
terminal evaluator may still own queued semantic events; the session remains
running until those events are drained into bounded history.

Promise settlement acts on the exact pending promise retained by `EvalFiber`.
`resume` applies that promise's real state to the retained continuation and
returns the resulting before/after boundary.

`reset` drops the old fiber and registry, creates a fresh isolated substrate,
and increments the generation. `cancel` releases the paused or suspended
continuation. `dispose` removes all runtime and history ownership behind the
opaque handle.

## Evidence schemas

The session wraps the existing live schemas without copying executable values:

```text
hal.interpreter-live-snapshot/0-alpha
hal.interpreter-live-boundary/0-alpha
hal.interpreter-observation-session/0-alpha
hal.interpreter-observation-entry/0-alpha
hal.interpreter-observation-history/0-alpha
hal.interpreter-observation-run/0-alpha
```

## Proofs

Native raw-ABI tests cover:

- nested arithmetic with exact source path and bounded retained history;
- nested closure frames containing captured and argument values;
- real pending promise settlement and continuation resumption;
- a deep loop with bounded history and explicit dropped evidence;
- fresh reset generation and isolated Var state;
- cancellation and deterministic handle disposal.

The dedicated build also compiles the standalone crate for
`wasm32-unknown-unknown` and emits `hara_interpreter_observation_raw.wasm`.
