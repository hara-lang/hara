# Core facade management prerequisite — 2026-09-15

The installed lang.core facade still publishes runtime/bootstrap and other
names absent from the faithfully ported runtime. The immutable original
tahto.core facade does not justify restoring those compatibility aliases.
Its management publication targets include lib-module-overview,
lib-module-entries, lib-module-purge, and lib-module-unused. The production
manage owner still has its older four-function native surface instead.

The saved lang_core_manage_native_work.edn candidate was revalidated against
the current installed native Work adapter, without the old helper overlay:

- Complete staged historical-output test namespace: 17 passed, zero failures
  or errors; state cleanup prints :manage-work-cleanup-verified.
- Real-library integration namespace: two passed, zero failures or errors.
- Bulk contracts namespace: three passed, zero failures or errors.
- Installed test/lang/core/manage_task_test.hal: 29 passed, zero failures or
  errors (including the subsequent adapter fixes).

These are separate test runs. An initial concatenation of all test namespaces
ran only the last namespace's two tests and is not evidence for the others.
No full-core or model fixture success is claimed. Staged historical tests
explicitly use direct library owners instead of the still-invalid core facade.

The older task-package record's assertion that all native invocation/selection
is unimplemented is stale. The saved Work adaptation deliberately replaces
callable Foundation Task values with native Work bundles and task/invoke; the
original configurations and arglists are retained as provenance. Do not port
std.lib.invoke to undo that approved boundary.

Production management replacement remains pending: merge and classify the
existing native test contracts, inventory all original helper/task tests, and
finish presentation semantics against the original authority before claiming
full parity. Then replace the owner through its structural recipe with source
and test recovery images and written-file/regeneration validation. The original
full core facade additionally requires notification default/lifecycle owners,
compiler publication targets and eager model initialization; management alone
will not make the facade complete.

Authority is Foundation fe54cc866473a888bfbd4ce2d3de6d8011f09f72. No production
source/test, immutable source, scheduling contract, or public compatibility API
was changed in this revalidation. Temporary stdin symlink removed. No commits
or publication; the full migration goal remains active.

## Summary presentation implemented — 2026-09-15

The native Work adapter now honors :print {:summary true}, matching the pinned
std.task.bulk/bulk-summary selection and ordering contract: numeric zero fields
are excluded only from display, durations use the existing format-ms, the
summary is bold with the original surrounding newlines, and it precedes
deferred notices and result projection. Returned summary data is unchanged.
Map text deliberately uses native representation, not universal Clojure map
printer byte parity. No new public function or renderer API was introduced.

The new path-matched behavioral regression checks exact output framing, false
and nil retention, zero-field preservation in data, and printing order. Old
source: 29 passed, one failed, zero errors. Complete candidate: 30 passed.
Written file: 30 passed twice. Native incomplete/unchecked inventory is empty.

Foundation's lang_core_manage_summary_native_written.edn preserves source/test
before-images, insertion recipe, hashes and validation. Its native replay was
read back and reconstructs both written files byte-for-byte on repeated runs.
Direct native reading outside the Hara project mount was correctly denied;
readback used ordinary read-only transport into the native verifier instead.
This adapter edit and replay use native Hara source processing, not new
Clojure tooling. Item/result/warning/error presentation and production
management owner replacement remain unfinished.

Installed launcher scaffold preview/write still exit 134, and source run still
rejects :config :role; these gates remain unresolved. Scoped whitespace check
passes. Temporary stdin symlink removed after terminal verification.

## Original renderer padding dependency staged — 2026-09-15

The existing native table renderer uses different terminal widths/truncation
and cannot silently replace Foundation's original multiline report renderer.
The six required padding/alignment functions from pinned
std.print.format.common were structurally selected and adapted through
code.migrate.lang/rewrite-owned-source (std.block), preserving their docstrings,
metadata and six original historical facts. They are staged under internal
lang.core.manage-print; no production namespace or general std.print API was
added. The original authority source/test and replay evidence are retained in
Foundation's lang_core_manage_print_padding_stage.edn.

Six authored boundary facts exposed a real translation defect: native repeat
rejects negative counts whereas the original returns an empty sequence. This
broke over-width strings and surplus rows. The local structural rule clamps
repeat counts at zero without changing native global semantics or truncating
content. JVM long-of-division is adapted to quot for truncation toward zero.

Initial historical suite: six passed. Before the repeat fix, the six new
boundary facts error. Fixed full candidate: 12 passed. Wrong expectations:
zero passed, 12 failed, zero errors. Saved-record replay is exact and the rule
is idempotent; fresh replayed native run passes all 12. Padding/custom-char,
odd/even centering, default alignment, over-width preservation, surplus rows
and padding idempotence are checked.

Next dependencies are original lines:elements, lines:row-basic, lines:row,
report:header, report:row and report:column. Result formatter callbacks must
run only in the original printing branch and in the original order. Full
item/result/warning/error rendering and management installation remain pending.
