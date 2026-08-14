# Bounded live evaluator snapshots

Issue: `hara-lang/hara#403`

This slice builds the first portable evidence projection on top of the merged
live `EvalFiber` pause/resume kernel. It observes the actual retained CPS
continuation; it does not replay an Evaluation Journal and does not invoke the
bytecode VM.

## Documents

The public evaluator methods emit JSON-safe Hara values using:

- `hal.interpreter-live-snapshot/0-alpha`
- `hal.interpreter-live-boundary/0-alpha`

A snapshot contains:

- caller-supplied source identity;
- live status (`running`, `paused`, `suspended`, `returned`, `failed`, or
  `cancelled`);
- whether a production continuation is retained;
- deterministic, name-sorted lexical/environment bindings;
- explicit binding count, limit, and omission evidence;
- bounded value displays with truncation flags;
- pending promise settlement state without promise identity;
- bounded terminal result or error evidence.

A boundary contains the bounded state before and after exactly one
`step_observed` or `resume_observed` call and classifies the result as
`evaluation/continue`, `evaluation/suspend`, `evaluation/resume`,
`evaluation/return`, `evaluation/fail`, or `evaluation/noop`.

## Safety boundary

Documents never contain executable `Value` instances, closures, continuations,
promise handles, mutable collection cells, iterator identities, or extension
handles. Functions, coroutines, promises, iterators, extensions, mutable values,
and host-backed collections use explicit redacted displays while remaining
owned by the fiber.

## Public surface

```text
EvalFiber::snapshot_observed_value
EvalFiber::snapshot_observed_value_with_limits
EvalFiber::step_observed_value
EvalFiber::step_observed_value_with_limits
EvalFiber::resume_observed_value
EvalFiber::resume_observed_value_with_limits
```

The caller owns sequencing and retention. A browser/session wrapper can add
stable boundary IDs and bounded history without changing the live evaluator
kernel.

## Deliberate limits

The current CPS `Step::Continue` variant has no semantic label or source-form
payload. These snapshots therefore expose an authoritative generic
`evaluation/continue` boundary plus the real environment delta. Attaching
current form, source span, macroexpansion origin, lexical-frame nesting, and
namespace mutation labels requires instrumenting the continuation producers and
remains the next #403 slice.

## Focused checks

Tests cover deterministic binding order, binding and text truncation, extension
handle redaction, terminal before/after evidence, JSON serialization, promise
state without identity, and one-boundary promise resumption.

The existing `Code VM live interpreter` workflow compiles these methods for
native Rust and browser Wasm.
