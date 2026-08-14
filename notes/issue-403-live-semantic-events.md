# Explicit live evaluator semantic events

Issue: #403

This slice extends the production-backed live evaluator context from source and
frame evidence into an ordered semantic event stream.

## Queue contract

Authoritative evaluator seams enqueue events while one real continuation is
executing. The observed driver publishes at most one queued event per host
step. If one continuation performs several semantically relevant operations,
subsequent host steps drain those events without executing another
continuation.

This preserves order without changing the ordinary `Step` enum or replaying
source. Completed and failed fibers may still have bounded semantic events to
drain; snapshots expose the remaining count explicitly.

## Events

The first explicit event set is:

```text
call/enter
effect/var-define
effect/var-set
effect/field-set
error/raise
error/catch
form/return
value/return
```

Call events are emitted only after the production evaluator has resolved the
callable and evaluated the actual arguments. Arguments use the existing safe,
bounded value projection.

Var and mutable-field events are emitted only after the authoritative mutation
succeeds. Var events include the actual previous value when present and the
committed replacement. A protected fallback `def` that performs no change does
not emit a commit event.

Raised errors are attached to the form boundary that propagated the actual
runtime error. The selected `catch` clause is recorded separately after its
binding has been installed. Adjacent duplicate propagation records are
suppressed without changing the runtime error.

## Proofs

Focused tests establish:

- `call/enter` for `(* 2 3)` precedes its `form/return` result `6`;
- `(def counter 1)` and `(set! counter 42)` produce ordered explicit effects
  with before/after evidence;
- division by zero produces `error/raise` with normalized category and exact
  source focus;
- the selected catch clause produces a later `error/catch` event;
- queued events can be drained after evaluator termination without advancing
  language execution.

## Remaining #403 work

The remaining major slice is isolated raw-Wasm/browser session ownership:
start, step, resume, snapshot/history, reset, cancel, and dispose. Expansion
origin and richer call-stack labels can then extend this same event stream.
