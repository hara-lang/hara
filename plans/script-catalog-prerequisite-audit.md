# Script catalog prerequisite audit — 2026-09-15

The GLSL catalog target requires installed lang.core.script, but the original
script owner is not yet a catalog target. Do not register its current tests as
full historical parity simply because the native suite passes.

Immutable authority fe54cc866473a888bfbd4ce2d3de6d8011f09f72 contains 28 facts
and 38 assertions in test/tahto/core/script_test.clj. The complete test file was
read during this audit. The prior reconciliation explicitly retains five full
original facts, empty-macro behavior and configuration assertions; runtime and
process assertions remain deferred in the original authority. Native contract
tests supplement those assertions, not replace them.

Executed the existing original, pipeline and correspondence source replay
stages. Each exactly matches its recorded intermediate source; the final
correspondence stage exactly matches the installed script source. The later
historical configuration replay also matched both installed files before the
test addition below. No production script implementation changed in this turn.

## Restored historical JavaScript preparation assertion

Added the original script-test-prep :js {} assertion to the path-matched native
test. Its expected module is routed from tahto.core.script-test to
lang.core.script-test. Original partial-map matching is represented by
select-keys; the original input and expected module are preserved. The existing
isolated Library/space/resource fixture and nested finally cleanup are reused.

The structural adapter and parent-linked recovery are saved in Foundation
resources/code/migrate/candidates/lang_core_script_historical_prep_written.edn.
The parent content is checksum-checked. Replay reconstructs the added fact
through rewrite-owned-source and verifies the complete source/test checksums.
The saved record was read back and replayed twice against the installed files;
both files match byte-for-byte. The earlier source reconstruction chain was
also executed, rather than relying solely on cached output strings.

Fresh native 0.1.33 evidence:

- Before: 52 checks passed.
- Complete candidate: 53 passed, zero failures/errors.
- Wrong-module negative: 52 passed, one failed, zero errors.
- Written test: 53 passed twice in separate processes.

Those 53 checks include three imported Lua preparation-fixture checks; the
script file itself runs 50 facts. This is one restored historical assertion,
not three newly migrated script tests. No assertion was removed or weakened.

## Remaining work

The original runtime setup, Lua annex execution, and common-data fixture tests
still need their full source/test dependency closure. In particular, preserve
the original return-1 and arr-map results and registered/active annex tag sets.
Do not register the current partial test assembly with an expected count that
hides those outstanding assertions. Promote the verified source reconstruction
and a fully accounted test adaptation into catalog handlers; retain all native
tests and the added historical preparation assertion. Full core facade and
runtime/process migration remain active work.
