# Replace grammar-api with the original pipeline

User decision: replace the remaining grammar-api architecture with Foundation's
original architecture. This takes priority over management presentation work.
Do not preserve the rejected architecture behind an alias or new facade.

Latest runtime evidence: [original JS bootstrap against installed models](js-original-bootstrap-installed-model-audit.md).
Both emitted return-eval and the original emit-generated socket client execute
six exact Node payloads without staged model overlays. The installed process
namespace still fails on obsolete `runtime/install-type`; the full original
process/basic dependency cutover remains required.

Authority: Foundation revision
`fe54cc866473a888bfbd4ce2d3de6d8011f09f72`. Read its `src/tahto` and `test/tahto`
files from Git objects only; do not modify those working trees.

## Replacement boundaries

| Legacy boundary | Original owner/shape |
| --- | --- |
| grammar-api profile/template/definition/source objects | Literal feature/template groups and the original grammar build/to-reserved/grammar composition |
| Catalog coordinates, selections and facets | Original Books, library installation and snapshots |
| Model install-source/install/select wrappers | Original model grammar, Book and script installation definitions |
| Legacy base.compiler/core.compile integration | Original core compilation, preprocessing and emission path; audit each call rather than mechanically renaming symbols |

Retain `lang.core.registry`: it owns original runtime/Book lookup, not the added
grammar catalog in `lang.base.registry`.

Preserve each target's original composition and group names. The `v1` path alone
does not identify a legacy implementation: some files there have already been
replaced with original-style code. Inspect imports and implementations.

## Verified cutover

The JavaScript root at `src/lang/model/v1/spec_js.hal` now uses the staged
original definitions instead of grammar-api, the legacy registry and the
js-grammar catalog model. Its path-matched test file is replaced with the
original-contract candidate. Obsolete catalog-specific tests and source are
retained as complete before-images in
`lang_model_js_original_root_cutover.edn` under Foundation migration candidates.

Fresh written-file validation: 23 JavaScript root facts pass, including Book
installation and emission through the original core implementation. All 18
unchanged emit-template facts now pass; previously their import failed on
`lang.model.v1.spec-xtalk/install`. Native code.manage inventory is clean for
the written definitions. Scaffold preview/write still fail with the existing
CLI stack overflow (exit 134).

This is not complete JS model parity: the original emit-html wrapper and its
HTML dependency remain unported. The historical assertion is retained in the
parent migration evidence/test comment and is not counted as passing. Do not
use the clean native inventory to obscure that original-source obligation.

## Remaining work

1. Re-audit callers of grammar-api, base.registry, base.compiler and
   core.compile. Replace consumers in dependency order with their pinned
   originals, preserving original tests and explicit before-images.
2. Finish target roots and required helpers; the old Go/Dart/Hara/PostgreSQL
   model layers still contain rejected catalog constructs. JS/Lua/Python names
   and paths alone do not prove full target parity.
3. Replace obsolete compiler/registry tests with the original pipeline tests,
   retaining relevant semantic contracts and recording architecture-only test
   dispositions. Do not make legacy tests green by expanding grammar-api.
4. Retain the corrected book-module boundary regression. Its synthetic
   `demo.inherited` fixture now intercepts materialization and verifies the
   inherited language and brief source view, restoring the intercepted Var's
   value and metadata. This is not end-to-end inherited-entry proof.
5. After all executable consumers are migrated, remove obsolete grammar-api
   and related catalog files with recovery evidence; verify no executable
   references remain. Four orphaned model grammar wrappers and two catalog
   test files have been removed; the main grammar-api remains in use.
6. Rerun dependency-ordered base/core/model tests and actual JS/Lua/Python
   emitted bootstrap/process tests, then finish full source/test/replay parity.

## Baseline audit

`lang_base_installed_dependency_validation.edn` records fresh per-file native
checks in declared source dependency order: 33 path-matched files, 29 passing
and four failing before the JS root cutover. emit-template is now repaired;
book-module's corrected unit suite passes 22 facts. The original core registry
now passes eight facts after correcting XTalk/Lua/Python model paths. The
catalog/compiler failures remain. Four additional test-only files are inventoried separately
and were not run in that pass.

The older `plans/lang-migration-dependency-audit.edn` contains literal output
truncation text and is unreadable EDN. It was not used as test-order authority.
The new baseline is validation evidence, not proof of full pinned-source parity.

## Recovery and validation update — 2026-09-14

The removed source wrappers are `spec_js/js_grammar.hal`,
`spec_xtalk/xtalk_grammar.hal`, `spec_lua/lua_grammar.hal`, and
`spec_python/python_grammar.hal` beneath `src/lang/model/v1`. The removed
path-matched tests are the JS and XTalk wrappers; Lua/Python had no separate
wrapper tests. All six before-images exist at Hara commit
`c89bc9b95b9c0895456db0bb17ef08b539f0f935`. Their catalog assertions are obsolete
by the explicit architecture decision. JS array/JSX behavior was transferred
to the original root test as `original-array-jsx-boundary`, with exact output
and a deliberately incorrect expectation observed failing.

Only the four wrapper entries were removed from the project source manifest
and migration policy reference inventory. Other manifest edits, including the
native version pin, are independent work and must not be reverted wholesale.
The migration policy still generates grammar-api-dependent material elsewhere;
inventory cleanup does not complete policy migration.

Written per-file checks after wrapper removal passed: JS 23, XTalk 6, Lua 38,
Python 43, core registry 8, emit-template 18, book-module 22, migration policy
core 52. These are separate focused checks, not a full pipeline parity claim.
Scaffold preview/write attempts still failed with stack overflow (exit 134).

`work.flow.make.artifact/compile-resolve` now uses the original resolution order:
dereference native Vars/pointers, resolve symbols, call only `function?`
producers, and leave other values intact. Recovery before-images and staged
outputs are retained in Foundation migration candidate
`lang_core_compile_resolve_pending.edn`; its pending filename is not status
evidence. On 2026-09-14 a fresh native readback compared both entire written
files to those outputs without whitespace normalization: two assertions passed.
The written artifact suite was rerun: 49 passed, zero assertion failures, one
error in the old compiler consumer (`lang.protocol.compiler/INormalizer`
unbound). That error existed before this resolver change. The seven resolver
facts previously detected incorrect Var/pointer invocation and unresolved-symbol
behavior before the change and pass with the original behavior.

Artifact-wide test correspondence is not complete: older `Test/check` cases
are not indexed like the added `fact` blocks by the current unit inventory.
Do not report a clean namespace-wide correspondence gate or suppress the old
compiler error. Next replace the original core compiler as a complete pair,
including script/schema/type registrations and its three consumer boundaries:
`lang.core`, artifact module dispatch, and PostgreSQL client compilation.

## Original C root installed — 2026-09-14

`src/lang/model/v1/spec_c.hal` and its path-matched test now use the pinned
original grammar, Book, and `script/install` pipeline. Removed the catalog
definition/selection helpers and the invented XTalk parent. Original literal
feature/template groups, callbacks, function names and documentation are
retained; existing public function/grammar/Book/init markers are preserved.

Fresh written suite: 38 passed, zero failures/errors: 20 historical facts,
ten retained native behavior checks, and eight added grammar/Book/flat-layout/
metadata contracts. All assertions have negative evidence (37 deliberately
incorrect expectations fail, and the metadata regression fails without the
three Var markers). Native written-file emission, metadata and correspondence
checks pass; incomplete/unchecked/scaffold-new are empty. Core registry's
eight checks also pass.

Foundation candidates `lang_model_c_original_root_cutover.edn` and
`lang_model_c_original_root_installed.edn` retain immutable inputs, full native
before-images, structural adapters, exact written outputs and verification.
Replay matches both written files byte-for-byte; a second generation matches.
The three obsolete catalog-only tests remain as comments/recovery data, not
passing tests. No native user behavior tests were discarded.

CLI scaffold preview/write still abort with stack overflow (134). CLI run
rejects `:config :role` in the existing book-module dependency (exit 1);
the fresh native runtime source and test checks above are separate evidence.
Do not claim those CLI gates pass or that other grammar-api consumers are gone.

## Original Go XTalk prerequisite installed — 2026-09-14

`spec_xtalk/fn_go.hal` now retains the original Var-valued macro entries and
literal four-group merge topology. Its path-matched test preserves all 17
historical facts and 17 native behavior checks, adding five exact group contracts.
Candidate and written suites pass 39 tests on release native 0.1.33, matching the
externally updated project pin. Deliberately incorrect expectations fail all 39
with zero errors. Four group contracts also fail against the old function-valued
tables. Written-source Var dispatch and correspondence pass two checks; missing,
unchecked and scaffold-new inventories are empty.

The staged original Go root passes all seven historical facts using the installed
helper (no helper overlay). The root itself remains uninstalled. Recovery,
immutable units, structural adapters, hand-authored additions and exact outputs
are in `lang_model_xtalk_fn_go_original_owner.edn`; its adjacent `_installed.edn`
records verification and hashes. Saved replay matches both written files exactly
and a second generation is identical.

CLI scaffold preview/write still exit 134; CLI source-run still rejects the
existing namespace role (exit 1). Native project-aware checks pass independently;
these CLI gaps are not waived. Next is the original Go root source/test cutover,
preserving existing wrapper behavior and declaration-metadata assertions.

## Original Go root installed — 2026-09-14

`spec_go.hal` now owns the pinned original five helpers, literal feature/template
groups, grammar, Book and script installation. The catalog definition/selection
layer and invented XTalk parent are removed. The superseded `go_grammar.hal`
source/test pair is removed, with full before-images verified in Foundation
`lang_model_go_original_root_cutover.edn`; the source manifest entry is removed.
The existing Go typed sidecar and external runtime-pin change are preserved.

All seven original facts and seven native wrapper checks now run in the root
test file, alongside eight hand-authored contracts: 22 pass after wrapper removal.
The native declaration-metadata fixture now uses original BookEntry form-symbol
metadata, retaining its exact emitted return-type output. All 18 native pipeline
examples retain their expected strings and use original `impl/emit-as`.
Four obsolete catalog-only root checks remain in recovery, not as passing tests.

All 22 tests fail with deliberately incorrect expectations and zero errors.
Written expression/statement/tagged-field emission and correspondence pass two
checks; incomplete, unchecked and scaffold-new are empty. The Go XTalk helper
passes 39 regression checks and core registry passes eight. Saved structural
replay matches source/test bytes exactly; second generation is identical.
Adjacent `lang_model_go_original_root_installed.edn` records hashes and results.

Validation used release native 0.1.33. CLI scaffold still exits 134 and CLI run
still rejects the book-module dependency's namespace role (exit 1); those gates
remain red. Dart, Hara and PostgreSQL roots still import grammar-api. The native
migration policy's stale wrapper preservation entries/catalog generation remain
part of the full-goal audit; this cutover does not claim that policy is repaired.

## Dart original dependency candidates — 2026-09-14

No Dart production source/test files changed in this stage. Installed baselines:
`spec_xtalk/fn_dart_test.hal` passes 142 tests; `spec_dart/rewrite_test.hal`
passes 45. Those results do not prove original source parity.

The staged original root fails with the installed helper because the previous
port inserted `:x-str-last-index-of` into the string override group. The pinned
source has 12 string entries and 112 merged operators, not the installed 13/113;
the native extension must remain separate. Original macro group entries are Vars.
The previous port also changed `dart-call` from variadic arguments to one vector,
and incorrectly made apply/construct emit `(Function.apply f [args])` instead of
the original `(Function.apply f args)`.

Pinned helper and root candidates now load together. Existing structural template
lowering, reader-discard handling and a single-arity adaptation are reused.
Native predicate/index boundaries are explicit: form? for syntax forms,
not-nil for some?, and `(first (drop 2 obj))` for the original optional index.
The original root retains its actual XTalk parent, unlike C/Go.

All 71 active historical helper facts execute through the staged original root:
70 pass, one fails, zero errors. Empty historical facts remain recovery comments,
not passing tests. The remaining failure expects `(-1.2).ceil()` and `.floor()`;
actual output contains `((double -1.2)).ceil()`/`.floor()`. The test source still
contains literal -1.2. A focused three-layer probe produced:

* `emit-helper/default-emit-fn`: `-1.2` (correct)
* `emit/emit-main`: `(double -1.2)` (incorrect)
* `impl/emit-as`: `(double -1.2)` (incorrect)

`emit-common/emit-token` still falls back to `pr-str token` instead of the existing
numeric formatter. Fix this shared fallback through its owning source/test and
replay rule before further Dart root work. Do not weaken the historical regex.

Comparing the staged original helper to unchanged native tests before the final
optional-index fix gives 128 passes, 11 failures, three errors. Most remaining
differences concern original versus replacement form shapes, argument wrapping,
and group sizes; each requires semantic reconciliation, not blanket expectation
replacement. The full root tests also require the typed `xt.event.base-route`
optional-argument fixture and have not yet run.

Foundation records `lang_model_dart_original_helper_pending.edn` and
`lang_model_dart_original_root_pending.edn` retain immutable inputs, before-images,
structural adapters and exact candidates. Saved-record equality and helper replay
were verified after chunked transport. Release native 0.1.33 was used throughout.

## Numeric token fallback corrected — 2026-09-14

`emit-common/emit-token` now delegates its default token formatting to the existing
`emit-helper/default-emit-fn`. This preserves Foundation decimal spelling instead
of exposing native `(double ...)` constructor notation. Explicit custom/as/emit
precedence is unchanged. The production source diff is one line.

The active Clojure owner `code.migrate.lang/rewrite-emit-token-form` contains the
same scoped rule. Three hand-authored native regression facts are registered in
the emit-common target's native additions in `resources/code/migrate/lang.edn`.
Generate with that explicit language catalog, not the generic default catalog.
The isolated rule namespace `code.migrate.token-test` checks fallback routing,
idempotence and preservation of unrelated functions/quoted data; its written
test passes. The full `code.migrate.lang-test` namespace load timed out and was
interrupted; after restarting the REPL, only the isolated test was validated.
The project nREPL is now on port 58936.

The baseline suite passed 133; the numeric regression failed on the old source.
Generated and written source/test pairs pass 136. All three new tests fail with
deliberately incorrect expectations and zero errors. Fresh written-source and
correspondence checks pass two assertions; incomplete/unchecked/scaffold-new are
empty. Both written files match fresh pinned generation byte-for-byte, and a
second generation is identical. Source/test generation diagnostics are empty.

The staged original Dart helper/root now passes all 71 active historical helper
facts with unchanged expected results, including negative decimal receivers.
Installed JS passes 23 and Python 43. Lua passes 37 with one exception-table
ordering assertion failure; the exact pre-change emitter produces the identical
37/1 result on the same 0.1.33 host. This is a retained issue, not a green Lua gate.
Dart production files remain unchanged pending native test reconciliation and
full original root/typed-fixture tests.

`lang_base_emit_token_numeric_installed.edn` records the replay recipe, checksums,
catalog recovery and verified native before-images at Hara commit
`c89bc9b95b9c0895456db0bb17ef08b539f0f935`. CLI scaffold still exits 134 and CLI run
still rejects the existing context.space namespace role (exit 1). No commits or
publication were performed.

## Dart template-order reconciliation — 2026-09-14

The retained exact classifier-form assertion exposed a migration-order error:
generic Clojure numeric equality rewriting changed literal target `==` to `=`
inside `template/$` before template lowering protected those symbols. Lowering
the original templates before the generic plan preserves the original operator;
the assertion remains exact, normalizing only generated local names.

The corrected staged helper with the staged original Dart root passes 213 facts:
71 historical plus all 142 retained native facts. Deliberately incorrect retained
expectations fail all 142 with zero errors; restoring the expectations passes all
213 again in a fresh release 0.1.33 process. The 13 explicitly reconciled native
facts and the separate native extension are preserved in
`lang_model_dart_template_order_reconciliation.edn`, which links the immutable
source/test authority and full native before-images in the earlier helper record.

This remains staged, not installed. Real target-runtime checks, production
installation/post-write validation, and original root fixtures remain.
The generic migration pipeline still needs the corresponding ordering audit;
this staged correction does not establish that other models are unaffected.

Saved-record readback matches the evaluated record. Two complete replays produce
identical source and retained test bytes (the initial 10-second observation was
interrupted; a subsequent 60-second invocation completed both generations).
Source SHA-256: `79a81ce3e23550ea8578afad22cdb453475948d15b017f1c2f44b0f73e47a369`.
Retained-test SHA-256: `aa3cdd3d8107d6eb8185c201b89568bb4c57cd1ee37e4cc3645cfe855bc650a3`.

The subsequent fresh native `code.manage.unit` inventory passes: incomplete,
unchecked, and scaffold-new are all empty for the combined historical/retained
source-test candidate, including the separately preserved native extension.
This supersedes the saved staging record's inventory-pending entry; it does not
resolve the previously failing CLI scaffold command.

## Catalog template-order regression — 2026-09-14

The shared `code.migrate/plan-unit` dispatch already applies catalog-selected
`:foundation/lang-model-templates` before generic source rewriting. The staged
Dart recipe had bypassed that dispatch; no shared engine change was necessary.
The current language catalog selects this path for the Lua and Python helpers.

Added Foundation `test/code/migrate/template_test.clj`, exercising both owners
with target `==` and `Array.isArray` literals beside an unquoted executable host
`==`. It asserts the complete resulting definition, empty diagnostics, original
input/checksum provenance, unchanged comments, byte-identical repeat generation,
and the selected template rule. The written test passes one fact. A deliberately
generic-first planner makes the written fact fail (one failure, zero errors or
timeouts); restoring the planner and running it alongside the numeric-token
regression passes both facts. No historical language source/test was modified.
This covers the catalog dispatch order, not complete installed model parity.

## Full Dart root fixture stage — 2026-09-14

Staged all 21 active original root facts and preserved the seven originally empty
facts explicitly as pending contracts. No historical assertion or required
`xt.event.base-route` fixture was removed. Native adaptations use `binding` for
the dynamic emitter callback and `satisfies` for predicate checkers. The generic
test migration's top-level `Test/run` wrapper is rejected by the current runner;
the staged pair retains native `fact` blocks instead.

Applied original root adaptation before generic rewriting, preserving literal
template operators. Two generations of source and tests are byte-identical.
`lang_model_dart_root_full_fixture_stage.edn` links the immutable units, root and
helper recovery records, adapter, checksums, and exact validation failure.

The complete fixture load fails before facts execute: `xt.event.base-route`
requires `lang.core`, whose old facade still publishes the absent
`lang.core.runtime/bootstrap`. This is reproduced with the template-first root
and helper on native release 0.1.33 (zero passes, one load error). Restoring the
original full core facade requires its audited dependencies, including
type-notify and compiler/management publication targets; adding a bootstrap
compatibility alias or omitting the fixture would not satisfy the original port.
No Dart production source/test was installed in this stage.

## Notification state dependency installed — 2026-09-14

Added `src/lang/core/type_notify.hal` and its path-matched test with six original
state functions: has-sink?, get-sink, clear-sink, add-listener, remove-listener,
and process-capture. This is a partial namespace, not a completed notification
server. The complete immutable source/test units and remaining function inventory
are retained in `lang_core_type_notify_state_stage.edn`; installation evidence is
in `lang_core_type_notify_state_installed.edn`.

The migration structurally selects the original functions and routes atomic
swap-return and watch operations to existing Foundation primitives. Listener key
normalization preserves existing keyword keys because native keyword construction
does not have Clojure's identity case. Six historical facts retain their original
expectations; setup is explicit at the first assertion, with isolated state
fixtures. Seven additional contracts cover presence, shared sink identity,
idempotent clearing/recreation, keyed callback replacement/removal, and capture
ordering/dynamic binding.

Candidate and written tests pass 13 facts. Deliberately wrong expectations fail
all 13 with zero errors. The written source runs through the CLI successfully,
and a fresh repeat of the path-matched tests passes 13. Saved-record readback and
two replays are exact; installed source/test bytes match regeneration.
Native candidate correspondence reports no incomplete, unchecked, or new scaffold
obligations; the same inventory passes against the written files. CLI scaffold
preview/write find zero records before the source exists,
but stack-overflow (exit 134) once installed; that gate remains unresolved.

Native watch self-removal and Promise settlement cleanup also pass two independent
seam probes. HTTP transport has no inspected native HTTP-server owner; TCP is
available through Socket. Neither transport, one-shot timers, the original server
type nor default resource lifecycle has been replaced with a placeholder.

## Notification one-shot registry installed — 2026-09-14

Added the original get-oneshot-id, remove-oneshot-id and clear-oneshot-sinks to the
same notification owner, bringing it to nine original functions. The complete
pair now passes 19 facts (nine historical, ten additional), including registered
ID membership, short-ID format, repeated removal, and retaining persistent or
unregistered sinks during cleanup. All 19 deliberately wrong expectations fail
with zero errors. A deterministic UUID-injection probe confirms collision retry
and exact Foundation base-36 IDs for two fixed UUIDs.

The existing native oneshot constructor owns the reused short-ID conversion; no
new sid facade or future library was introduced. Source/test before-images,
immutable authority, structural adapters, oracle, and replay are saved in
`lang_core_type_notify_oneshot_registry_stage.edn`, with written verification in
`lang_core_type_notify_oneshot_registry_installed.edn`. Source loads through the
CLI (exit 0), both fresh written test runs pass 19, and regeneration matches both
files byte-for-byte. Candidate and written correspondence are clean. CLI scaffold preview and
write still stack-overflow (134), separately from passing native tests.

Next: watch-oneshot's Promise boundary, then message dispatch and transport/server
lifecycle. The pinned future:timeout rejects its internal future, but the original
on:complete callback ignores the error and returns ret; the outward watch-oneshot
future therefore resolves nil after timeout cleanup. Preserve this behavior, not
a superficially similar Promise/finally chain that propagates rejection.

## Notification Promise watch installed — 2026-09-14

Installed watch-oneshot using native Promise/new and Promise/delay. The returned
Promise retains message values (including false/nil), resolves nil on timeout,
and registers idempotent cleanup before either a watch or deadline can complete.
Cleanup removes the keyed watch, registered ID and sink, and cancels the deadline.
Cancellation and partial initialization errors also clean up. Zero/negative
timeouts and a recreated sink surviving the previous watch's deadline are tested.

The full notification pair passes 27 facts: ten historical contracts and seventeen
additional checks. All 27 deliberately wrong expectations fail with zero errors.
The written source runs through the CLI (exit 0), and the path-matched written
tests pass 27. Original docs and full before-images are retained in
`lang_core_type_notify_watch_stage.edn`; serialized replay matches written source
and tests byte-for-byte. `lang_core_type_notify_watch_installed.edn` records the
installation evidence. Ten original functions are now installed, not the complete
notification namespace.

The subsequent fresh repeat passes all 27 tests; written-file native inventory
reports no incomplete, unchecked, or new scaffold obligations.

An intermediate source layout introduced clojure.core/deref inside a native
callback. The callback failed without settling its Promise, leaving a test process
spinning. That exact process was confirmed and terminated (exit 143); the adapter
now normalizes deref before layout. Tests use bounded waits, and the defective
candidate now terminates with 22 passes, two failures and three errors. Native
promise/cancel returns a Promise and reports :cancelled, which is asserted directly.

Message dispatch, printing, HTTP/TCP transports, server type/lifecycle, default
resource registration, and full core facade integration remain. CLI scaffold
preview/write still stack-overflow (134); this has not been relabeled green.

## PostgreSQL current-layout revalidation — 2026-09-14

Current model source and tests are under `lang.model.v1.spec-postgres`, not
the earlier unversioned paths in migration records. Fresh native 0.1.33 runs
using `test --project . --file <path>` establish:

- `test/lang/model/v1/spec_postgres/postgres_form_deftype_test.hal`: 30 passed.
- `test/lang/model/v1/spec_postgres/postgres_form_defn_test.hal`: 9 passed.
- `test/lang/model/v1/spec_postgres_test.hal`: 34 passed, 2 errors, exit 1.

The two root errors are function metadata formatting and PL/pgSQL function
emission, both calling `postgres-native/pg-defn-format`. That legacy owner
uses `(nth normalized 2)`; the migrated `postgres-form-defn` owner explicitly
normalizes the formatted form to a vector and passes its typed-input tests.
The reported root error is unsupported `INth/nth` receiver. The root and its
`postgres-grammar` dependency still retain the catalog/grammar-api architecture;
passing helper tests do not prove original pipeline integration.

Next integration work must reconcile current versioned ownership with the
immutable-source migration records, preserve the existing root assertions,
and route the original grammar through its original formatter/emitter owners.
Do not patch the duplicate legacy formatter merely to make this suite green
while leaving the rejected root architecture intact. Schema reference and
original hydration/application dependencies remain prerequisites for the full
root cutover. No source changes or test expectation changes were made during
this revalidation.

## Notification printing written — 2026-09-14

The structural migration adapter now appends original `process-print` with its
docstring and `:added` metadata. String output preserves numbered lines, CRLF
normalization, trailing-empty-line removal, empty-string behavior and numbers
beyond 99. Optional namespace/id/location/data headers are tested. Non-string
values deliberately use the native pretty-printer's representation and layout;
this is documented, not a claim of universal Clojure pprint byte parity.

The complete notification pair passes 31 tests in the candidate and twice after
writing. Eleven original functions are now present. Four printing tests include
the original historical assertion; all four fail with deliberately incorrect
expectations while the previous 27 continue passing. Written native inventory
reports no incomplete, unchecked, or new scaffold obligations. Source CLI run
exits 0. Scaffold preview and write still stack-overflow (134).

Full before-images, structural adapter, candidate, tests and serialized replay
are in `lang_core_type_notify_print_pair_stage.edn`; written evidence is in
`lang_core_type_notify_print_written.edn`. Both written files match replay
byte-for-byte and a second generation is identical. Message dispatch is next;
transports and the full notification namespace are not complete.

## Notification message dispatch written — 2026-09-14

Appended original `process-message` using immutable source and the owned
structural adapter. Native Json/read and str/starts-with? replace the original
host calls; dispatch ordering and sink conditions are unchanged. The original
listener assertion is preserved with local state/cleanup. Its writer input uses
string keys because native Json/write rejects keyword keys, while the serialized
message remains the same.

The complete pair passes 36 tests in the candidate and twice after writing.
Five new tests cover the historical listener, print/capture dispatch, ignored
IDs, persistent updates, and actual one-shot Promise settlement/cleanup followed
by a late message. All five deliberately wrong expectations fail with zero
errors, leaving the prior 31 passing. Source CLI run exits 0 and written native
correspondence reports no incomplete, unchecked or new scaffold obligations.

`lang_core_type_notify_message_stage.edn` retains immutable source/test units,
before-images, adapters, tests, hashes and replay. Both written files match
serialized replay and second generation byte-for-byte; installation evidence is
in `lang_core_type_notify_message_written.edn`. Twelve original functions are
now present. Scaffold CLI preview/write still stack-overflow (134). Transport,
server type/lifecycle and full core facade integration remain incomplete.

## Notification TCP written — 2026-09-15

Installed original handle-notify-socket, start-notify-socket and
stop-notify-socket through the owned structural adapter, retaining docstrings
and metadata. Native subscribed Promise work replaces Java threads/futures;
no future library or public socket API was added. Bytes accumulate until EOF
before UTF-8 decoding. Guarded cleanup handles completion, cancellation and
repeated server stop. Native listener close was corrected separately to release
the port before returning; see hara-native/docs/socket-listener-close.md.

The full pair passes 44 tests in the candidate and twice after writing, including
the historical start/stop assertions plus split UTF-8, malformed input, successive
connections without manual polling, active cancellation/peer EOF, same-port
restart and failed-bind cleanup. All eight deliberately wrong TCP expectations
fail with zero errors while the prior 36 pass. The failed-bind fixture reserves
the same wildcard address as the implementation; a loopback-only reservation did
not reliably produce that failure condition.

Written native correspondence is clean and the CLI source run exits 0. Scaffold
preview/write still stack-overflow (134). Immutable units, before-images, source
and test adapters, and exact replay are in lang_core_type_notify_tcp_stage.edn;
written evidence is in lang_core_type_notify_tcp_written.edn. Both files and a
second generation are byte-identical. Supplementary Unicode is escaped in the
serialized record after detecting an emoji corrupted during transport; decoded
record equality is explicitly checked.

Fifteen original notification functions are now present. HTTP, server type,
combined start/stop, default resource registration and full core facade remain.
