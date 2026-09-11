# Original entry pipeline: installed, migration incomplete

Subsequent state-owner cutover: the suite still has 27 passes, but the color
integration test now stops at the old `library-raw-snapshot` caller before
reaching the earlier compilation limit. See `state-owner-cutover.md` for the
current integration boundary; the results below record the initial entry cutover.

`src/lang/core/impl_entry.hal` now contains the complete original entry emission
and cache implementation, adapted through `code.migrate` from Foundation commit
`fe54cc866473a888bfbd4ce2d3de6d8011f09f72`. Native `entry-symbol` is retained.
The original recent-cache policy uses the approved object-identity comparison
and provides idempotent reset plus snapshot/restore boundaries.

All existing path-matched tests are preserved; 12 tests were added for cache
identity, force/bypass bindings, state storage/restoration, and native Var
transforms. Native scaffold correspondence reports no missing or unchecked tests.
All 12 deliberately incorrect expectations failed with zero execution errors.

The written test was run with:

```text
../hara-native/core/rust/target/release/hara-native test --project . --file test/lang/core/impl_entry_test.hal
```

Result: 27 passed, zero assertion failures, one pre-existing error. Before this
change, the same suite had 15 passes and the identical error:
`xt.lang.common-color` compilation exceeds the 255-argument limit. No test was
skipped. `git diff --check` passes.

The separate installed `hara --project . --offline run` launcher rejects the
existing namespace `:role` option. The workspace native runtime loads and tests
the written source; launcher validation must not be reported as passing.

An additional installed-dependency run used the same native test command with
`test/lang/base/book_test.hal` and `test/lang/base/emit_template_test.hal`.
Book passed all 64 facts. Template tests failed before executing any facts:
`lang.model.v1.spec-js` loads `lang.model.v1.spec-xtalk`, whose Book construction
throws `Meta required`. This is a model-loading failure, not a passing template
regression suite. The process terminated with exit 1; no test job remains live.

## Reproduction and remaining work

The validated source regenerates byte-exactly from
`reference/foundation-base/resources/code/migrate/candidates/lang_core_impl_entry_full_stage.edn`
using its source unit and source target with the loaded catalog/specs.
Exact before-images and the new authored tests are preserved in
`reference/foundation-base/resources/code/migrate/recovery/lang_core_impl_entry_native_install.edn`.

The user explicitly approved the Foundation catalog and migration evidence
update after safety review. The catalog now selects the complete original
entry source rule, rather than the older constructor-only target.

Historical model-backed entry tests remain in the full-stage recipe, not the
installed test file. Installed Lua/XTalk models still use the rejected
`grammar-api` path and fail Book construction with `Meta required`. Continue the
original Snapshot/Library/script/model cutover; do not add a compatibility shim.
