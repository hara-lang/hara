# XTalk seed compatibility

## Matrix

Validate each affected canonical fact in this order:

1. Native Hara evaluation of the complete candidate.
2. Focused canonical test under `core/lib/test-lang/xt`.
3. JavaScript emitted form and runtime fact.
4. Python, Dart, and Lua generated facts when marked compatible.
5. A second generation with no unexplained diff.

## Classification

- Canonical failure: fix portable source or shared semantics.
- Generator failure: fix seed metadata, staging, or generation logic.
- Runtime failure: fix the target runtime implementation.
- Rewrite failure: use a target adapter or transform for a genuine target
  restriction.

Do not infer a failure from unequal check counts alone. Inspect suppressions and
transforms first. Do not add suppression when an implementation, portable API,
or adapter can preserve the fact.

## Access semantics

Keep keyed and indexed access distinct through staging. Use explicit semantic
operations where receiver shape is known; report an unknown shape instead of
letting an individual backend guess differently.
