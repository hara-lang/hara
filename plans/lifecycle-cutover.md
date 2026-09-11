# Original lifecycle emission: installed native cutover

Authority: Foundation `fe54cc866473a888bfbd4ce2d3de6d8011f09f72`.
Status: source/test pair installed locally; full historical/model parity remains incomplete.

## Installed and verified

- `src/lang/core/impl_lifecycle.hal` follows the original tuple-based emission
  context, setup sections, native/link/export hooks and teardown sequence.
- Its path-matched tests have 16 runnable facts: two historical assertions and
  14 explicit native assertions. Nineteen historical assertions remain deferred
  with their source retained in migration evidence.
- Candidate and written lifecycle runs: 16 passed, zero failures/errors.
- Written `impl` regression run: 42 passed, zero failures/errors.
- Both written runs loaded installed impl and lifecycle code, without overlays
  for either implementation or their installed compiler dependencies.
- Original model construction candidates and explicit Lua fixture loading are
  still required. This is not a standalone project or runtime.basic pass.
- Source and tests regenerate byte-for-byte from the written catalog.
- Scaffold preview/apply and incomplete/unchecked audit: one passing check.
- Wrong expectations: all 16 facts fail, zero runtime errors.
- Exact module setup output and fully suppressed empty output now supplement
  the older adopted type-only assertion.

## Fidelity and retained work

The previous flat-layout link suppression was not original behavior and has
not been carried forward. Normalized equal graph paths use the original empty
relative string, producing `/dep`, rather than the previous `./dep`.
The former `emit-lifecycle-form` helper is replaced by the original calls to
`impl/emit-direct`; no compatibility wrapper was added.

Section ordering and join boundaries remain tested. Original normalized graph
paths, alias handling, import overrides/suppression, export controls and exact
Lua setup/teardown are covered. Old JS fixture-specific assertions are pending,
not claimed as identical parity merely because corresponding Lua tests pass.

Displaced files and per-test dispositions:
`resources/code/migrate/recovery/lang_core_lifecycle_native_cutover.edn`
in Foundation. Detailed validation:
`candidates/lang_core_lifecycle_native_cutover.edn`.

Three original trailing-whitespace lines remain at source lines 30, 277 and 281;
`git diff --check` is not clean for this pair.

Next: the dependency-free original core registry, then remaining script/model
integration and restoration of deferred tests.
