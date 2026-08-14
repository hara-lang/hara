# Live evaluator pause and resume kernel

Issue: `hara-lang/hara#403`

This slice introduces a real, opt-in stepping boundary in the production Rust
CPS evaluator. It does not replay a completed Evaluation Journal and does not
invoke the bytecode VM.

## Boundary

`EvalFiber::start_observed` retains the evaluator's initial continuation before
executing source. Each `step_observed` call consumes at most one existing
`Step::Continue` trampoline boundary. If the next production step returns
another continuation it is retained in the same fiber; terminal values and
errors become the existing `EvalFiberState` outcomes.

The ordinary `EvalFiber::start` and `drive_sync` behavior is unchanged and
continues to drain trampoline continuations at full speed.

## Suspension

When the evaluator reaches a real pending promise, the observed fiber retains:

- the original `Promise` identity;
- the original CPS resume closure;
- the shared lexical environment;
- the existing `Suspended` state.

`resume_observed` applies exactly one promise settlement and stops again at the
next continuation boundary instead of draining the rest of evaluation.
Cancellation releases either a paused continuation or a pending promise through
the existing fiber lifecycle.

## Deliberate limits

This is the live pause/resume kernel only. A `Step::Continue` boundary currently
has no portable semantic label, form path, source span, lexical-frame delta, or
expansion origin. Those projections belong in the next #403 slice. Coroutine
`yield` outside a coroutine remains the existing evaluator error; this slice
does not redefine coroutine ownership.

## Focused checks

The module tests cover:

- a source that remains paused until explicitly stepped;
- multiple retained continuation boundaries and the final value;
- real promise identity and one-boundary resumption;
- cancellation of a paused continuation;
- unchanged full-speed `EvalFiber` behavior.

Recommended command:

```sh
cargo test --manifest-path core/rust/Cargo.toml \
  core::fiber::coroutine::observation --lib
```
