# Original impl pipeline: installed native cutover

Authority: Foundation `fe54cc866473a888bfbd4ce2d3de6d8011f09f72`.
State: installed locally; script/model integration is not complete.

## Installed

`src/lang/core/impl.hal` now follows the original Library -> emission-options ->
preprocess -> dependency collection -> emit pipeline. Its path-matched test file
contains all 45 historical assertions and 12 explicit native assertions.

The native compiler-context-map implementation has been replaced by the original
five-element emission context. No grammar-api shim or obsolete Library adapter
was introduced.

## Evidence

- Registered candidate: 42 facts passed, zero failures/errors.
- Written-file run: 42 facts passed, zero failures/errors.
- Source and tests regenerate byte-for-byte from the written migration catalog.
- Scaffold preview/apply and incomplete/unchecked audit: one passing check.
- Eight added contract tests with deliberately wrong expectations: 34 unchanged
  facts passed, all eight changed facts failed, zero runtime errors.
- Two original trailing-whitespace lines remain at source lines 62 and 229;
  `git diff --check` is therefore not clean for this pair.

The post-write test process loaded the installed impl, imports, collector, Book,
emit, entry, Snapshot and Library code. It still supplied staged original
XTalk/Lua/JS model construction, the original lifecycle candidate for adopted
historical lifecycle tests, and explicit Lua test-fixture loading. This is not a
standalone project or runtime.basic pass. Full model targets remain unregistered
in the main migration catalog.

## Retained native contracts

- Original emission context retains module, layout and emit options without
  mutating caller input.
- Empty import selection and exact script-join boundaries are retained.
- Five former Library-owned contracts now live under their original impl owner:
  clone isolation, process-default identity, idempotent resource reset, runtime
  default/annex/binding precedence, and binding restoration after exceptions.
- The original default reset stops/removes the global resource; it does not
  mutate an already-held Library reference. Tests distinguish these behaviors.

Three old color integration facts (entry, dependency emission, and complete
script emission) remain pending the original script/model pipeline. Their
source is preserved, not replaced by weaker assertions.

Recovery: Foundation
`resources/code/migrate/recovery/lang_core_impl_native_cutover.edn`.
Detailed validation: `candidates/lang_core_impl_native_cutover.edn`.

Next dependency-ordered cutover: `lang.core.impl-lifecycle`, followed by the
remaining original script/model integration.
