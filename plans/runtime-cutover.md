# Original runtime cutover

The original `lang.core.runtime` source and its path-matched test are now
installed locally. This is not completion of `lang.core`, the language models,
or the real `lang.runtime.basic` processes.

Authority: Foundation `fe54cc866473a888bfbd4ce2d3de6d8011f09f72`,
`src/tahto/core/runtime.clj`. Target base:
`c89bc9b95b9c0895456db0bb17ef08b539f0f935`.

## Verified

- All 42 original named definitions, including RuntimeDefault, are represented.
- The complete candidate passes 56 facts in a fresh native process.
- The written runtime passes an installed-only smoke fact without overlays.
- The written test now passes all 56 facts with no source, model or fixture
  overlays. Registry, Space, resource and owned Library cleanup
  is checked and prints `:runtime-test-cleanup-verified`.
- Source and test regeneration from the saved evidence are byte-exact.
- The assembly postprocessor removes one identical repeated `rt-null`, is
  idempotent, and rejects conflicting duplicate definitions.
- The scoped source/test diff passes `git diff --check`.

Evidence is in Foundation
`resources/code/migrate/candidates/lang_core_runtime_installed.edn`.
The referenced `lang_core_runtime_complete_stage.edn` retains exact source
and test before-images, the original structural assembly recipe, historical
test adaptations, correspondence checks and negative-test evidence. The
previous native tests are also retained in an inactive archive comment in
the path-matched test file; they are not counted as passing coverage.

## Remaining dependency order

1. Original `lang.core.script/install` is now installed: five focused facts
   pass from disk without overlays. The rest of script is not ported; at the
   installer cutover, its existing 2 failures and 70 errors were unchanged
   (full-suite passes increased from 22 to 27).
2. Original XTalk is now installed: all six facts pass with the exact
   path-matched test command and no overlays. Original Lua is now installed:
   all 29 definitions match the authority, 37 facts pass from its exact
   path-matched file without overlays, and source/test regeneration is exact.
   Installer's five focused facts also pass without overlays. Continue with JS roots
   and their required helpers, using Book plus `script/install`, without
   grammar-api registration. Their obsolete calls to XTalk catalog APIs must
   be removed by that faithful cutover, not restored as compatibility aliases.
3. Native test-root loading is fixed in Rust `run_project_tests`: declared
   test roots are available only to the test runner, with project source roots
   taking precedence. All 76 Rust checks pass with loopback permission, and
   the release host is rebuilt. The runtime suite passes normally: 56 facts.
4. Pointer and script-control cleanup hooks are moved before all facts and
   explicitly verify restoration. Written tests pass 54 and 14 facts; a broken
   teardown fails in the corrected position but was masked in the old late
   position. Runtime-proxy also passes its 16 facts normally. These four core
   suites total 140 passing facts without overlays.
5. Continue the remaining core and real runtime.basic process/bootstrap ports.

Java inspection confirms existing native Result, callable Var/Pointer and
Pointer metadata support. No new Java helper was needed for this cutover.
No commits or publication were performed.

Installer and XTalk evidence is saved in Foundation candidates
`lang_core_script_install_installed.edn` and
`lang_model_xtalk_installed.edn`. Both record byte-exact regeneration and
recovery; the XTalk initialization negative rejects Book-only `+init+`.

Exact installed XTalk check:
`../hara-native/core/rust/target/release/hara-native test --project . --file test/lang/model/v1/spec_xtalk_test.hal`
from the Hara root: 6 passed, 0 failed, 0 errors.

Lua evidence is in Foundation `lang_model_lua_installed.edn` and
`lang_model_lua_installed_consumers.edn`. The source has no grammar-api
dependency. Historical coverage retains four helper facts (nine assertions)
and three emission facts (three assertions); the Nginx variant async-transform
assertion remains explicitly pending its owning variant. The scoped scaffold
audit has no missing/TODO/unchecked tests. Replacing Lua function bodies with
nil produces 30 failures.

Exact installed Lua check:
`../hara-native/core/rust/target/release/hara-native test --project . --file test/lang/model/v1/spec_lua_test.hal`
from the Hara root: 37 passed, 0 failed, 0 errors.

Latest integration evidence:
`lang_core_runtime_installed_integration.edn` in Foundation candidates.
It retains the cleanup before-images, reproducible structural edit recipe,
red/green teardown evidence, host validation and normal runtime results.
The older `lang_model_lua_installed_consumers.edn` records the now-resolved
test-loader failure as historical evidence, not a current blocker.

## JS helper follow-through

The original `spec-xtalk.fn-js` source/test pair now has a verified,
reproducible candidate: all 102 functions and 20 groups are represented,
142 facts pass, cleanup is verified, and scaffold reports no missing or
unchecked tests. Replacing all helper bodies with nil breaks 121 facts.
Saved reconstruction also passes 142 facts in a fresh process.

This pair is **not installed yet**: its historical emission tests use the
original JS grammar/Book candidate, while the installed JS root still uses
the old architecture. Foundation evidence is
`lang_model_fn_js_original_pair_stage.edn` and its adjacent review note.
The candidate also fixes a leftover JVM regex hint and explicitly selects
sequence traversal when the JS model emits map entries as strings.
Continue the JSX/root dependency cutover; do not count this stage as an
installed namespace or restore the removed XTalk catalog API.

## Python model and callback installed

The Python root and `spec-xtalk.fn-python` source/test pairs are now installed.
The root preserves the original grammar/Book and `script/install` load-time
initialization with no `grammar-api` dependency. Fresh exact-file native tests
pass 42 root facts and 245 callback facts, without candidate overlays.
All four written files match the validated candidates byte-for-byte; model
structural replay and callback catalog-plus-init-contract replay are exact.
Coverage audits find no missing/TODO/unchecked tests. The 25 new model
contract tests all reject deliberately incorrect expectations.

Foundation evidence and full recovery before-images are in
`lang_model_python_original_pair_cutover.edn` and its adjacent review note.
Root catalog registration remains pending. This does not complete the Python
process: its current `source-python` bootstrap is still hand-written text and
must be replaced by the original `emit-entry-deps` / `emit-as` generation path
after its script/runtime dependencies are faithfully ported.

## Original runtime options installed

`lang.runtime.basic.type-common` now contains all 15 original functions and
the original dynamic option state. Its 36 path-matched facts pass against the
installed registry without dependency overlays. All 36 wrong-expectation
controls fail; scaffold/TODO/unchecked audit is clean. Written files and saved
source replay are byte-exact, and the scoped diff check passes.

The two prior native test contracts have explicit adaptations: recursive option
merging is asserted through the original option operation, and `valid-context!`
returns nil for accepted contexts. No compatibility aliases were added. Full
before-images and replay are saved in Foundation
`lang_runtime_basic_common_installed.edn` with an adjacent review note.

This resolves the original registry/type-common dependency boundary. Runtime
type/process consumers still require coordinated original ports. A fresh
`xt.lang.common-lib` load also confirms that the old `lang.core` facade still
publishes the removed `runtime/bootstrap`; faithful script/facade publication
must replace that architecture before full registered bootstrap loading works.

## Original oneshot structure installed

`lang.runtime.basic.type-oneshot` now contains the original seven functions,
open record, and original context delegation mapped to native `IContextEval`
and `IContextLifeCycle/setup-ptr`. The new path-matched test file passes 31 facts
in a fresh exact-file run. Source/test replay and written bytes match; coverage
audit and scoped diff check pass. Two new context tests reject wrong results.

Evidence/recovery: Foundation `lang_runtime_basic_oneshot_original_pair.edn`
and its adjacent review note. This is explicitly **not full oneshot parity**:
historical pointer/bootstrap integration, real first-load process registration,
four unsupported shell options, and legacy type-twostep/basic-test consumers
remain. The source uses native Promise/Result and synchronous process cleanup;
no std.lib.future or compatibility aliases were introduced.

## Verify prerequisite staged, not installed

The original Python registration needs `verify-exec-file`; the current native
replacement lacks it. All six original `type-verify` functions now have a
generated candidate with nine passing facts, including the five historical
expectations. Negative controls fail all nine; coverage audit and saved replay
are clean. Evidence is Foundation `lang_runtime_basic_verify_mounted_stage.edn`.
Follow-through adds a tenth passing fact: argument-substitution failure now
cleans both allocated files. The test fails against the old cleanup scope and
passes against the corrected candidate; replay and coverage audit remain clean.

The candidate remains uninstalled: Foundation's default OS temp directory lies
outside the native project filesystem mount. Explicit in-mount roots work with
real checker processes. A user decision is pending between native OS-temp-file
support and a deliberate project-contained temp-file policy. The candidate also
records shell-option gaps; allocation-before-substitution cleanup is corrected. Other
core/script migration work can continue without deciding this filesystem policy.

## Script-def rule hygiene corrected; native pair staged

The std.block migration rules now keep native splicing `apply` and qualify
generated builtin calls so optional arguments cannot shadow them. Both focused
Clojure facts pass after written reload and detect the old rules. All eight
historical assertions remain; candidate regeneration is byte-exact.

The seven-fact native pair has six passes and one failure, so remains uninstalled.
The failure is isolated to Rust compiler rest destructuring: counting a rest
iterator consumes it before template emission. Java and the Rust interpreter
materialize rest vectors. A narrow Rust compiler exception has been requested;
no compiler change or semantic workaround was made. Evidence and replay are in
Foundation `lang_core_script_def_hygiene_stage.edn` and the adjacent review note.

## Original script-annex dependency staged

The original annex precedes script and has no script dependency. Its complete
16-function source, RuntimeAnnex record, and resource initialization are staged
with 18 authored facts. Scaffold inventories 17 obligations; correspondence is
clean, and all 18 wrong-expectation controls fail. Exact replay and native before-
images are in Foundation `lang_core_script_annex_original_stage.edn`.

Do not install the staged lifecycle fallback: although 17 facts pass, the real-
component stop test fails. Foundation's ordinary-Object start/stop fallback is
absent natively, and checking `satisfies? IComponent` skips partially implemented
components whose stop method is callable. Historical 16-fact/19-assertion parity,
native test reconciliation, and annex-reset return disposition remain pending.
The ordered pipeline is annex -> script -> original facade -> registered basic
processes and emit-generated bootstraps; no obsolete aliases were reinstated.

Annex follow-up adds a stop-error/state-preservation regression. The rejected
predicate fallback now has 17 passes and two assertion failures; both lifecycle
regressions pass with direct dispatch. Correspondence and byte-exact replay
remain clean. Evidence: Foundation `lang_core_script_annex_stop_dispatch.edn`.
A native method-support API decision is pending; no API or runtime change was
made, and annex remains uninstalled.

## Approved native boundaries fixed; script-def installed

The user approved the Rust rest-destructuring correction and a native method-
support API. Both are implemented: rest bindings materialize reusable vectors,
and `Base/supports-method?` inspects dispatch without invoking a method or
changing whole-protocol `satisfies?`. Java and Rust agree on the focused cases.
The release native host is rebuilt. Four Rust destructuring tests, all 36 native-
lang tests (with loopback permission), and two focused Java tests pass. Java
coverage was disabled only for the invocation because its agent rejects the
installed JDK version.

Original `lang.core.script-def` is now installed with all eight historical
assertions, all three existing native facts, and the new exact metadata/wrapper
contracts. Its written path-matched test passes seven facts; scaffold audit is
clean and seven wrong-expectation controls fail. Source/test replay is byte-exact.

The annex candidate now passes all nineteen authored facts, including actual
stop dispatch and stop-error/state preservation. Its method-support blocker is
resolved, but annex remains staged pending original historical/native test
reconciliation. Continue annex -> script -> facade -> actual process/bootstrap
integration. Earlier pending native-boundary notes above are superseded here;
the temporary-directory and model-parity decisions are not resolved by this work.

Evidence/replay: Foundation `lang_core_native_method_boundary_installed.edn`.
Native API contract: hara-native `docs/protocol-method-support.md`.

## Annex reset and historical assertions reconciled (staged)

The original reset returns the removed annex instance. The candidate now captures
that instance before native Space teardown; it does not change the Space API.
An identity assertion fails against the prior nil-return candidate (18 pass,
one failure) and the corrected candidate passes all 19 authored facts.

Historical predicates are explicitly applied, original per-fact setup is
materialized, and record-valued map checks use the corresponding native record
predicate. The original any? check evaluates its input without swallowing errors.
These adaptations remove the false comparisons and cyclic-value reporter crashes.
Dynamic book loading now evaluates a native require form with a quoted namespace
symbol, rather than treating require as a first-class function.

The final historical run has 15 passes and one error: the literal Rust first-load
assertion reaches `lang.model.annex.spec-rust`, which is not installed. All 16 facts
and 19 historical assertions remain in the staged test. The fixture currently
loads migrated Lua directly; original xt.lang fixture parity still depends on the
script/facade cutover. The existing native annex pair remains untouched pending
its test reconciliation. Do not report annex as installed or its full suite green.

Exact candidates, adapters, before-images, negative evidence, test output, and
executable replay: Foundation `lang_core_script_annex_reconciliation.edn`.

## Existing native annex tests: storage slice reconciled

Inventoried all 59 existing native Test/check blocks (61 assertions), preserving
their literal before-images and the complete installed source/test pair. Eighteen
storage blocks now have candidate adaptations: seven keep their exact expected
values; eleven use original independent registry/runtime atoms, parent Library
Var identity, original registration fields, and observable component stop hooks.
The :postgres cases retain that language and explicitly install its default
runtime context through original install-lang!, rather than changing languages.

The candidate suite passes 37 facts (19 original-contract plus 18 reconciled
native facts). Replacing each of the 18 new expectations with a wrong value gives
exactly 18 failures, 19 passes, and no errors. These are candidate checks, not an
installed-annex claim. Thirty-two native blocks belong to the original script
owner and nine assert replacement-specific architecture; both groups retain
explicit pending dispositions. The historical missing Rust model remains open.

Recovery, ownership mapping, candidate tests, and replay:
Foundation `lang_core_script_annex_native_reconciliation.edn`.

## Complete original script source staged; focused contracts verified

The immutable original script source (29 functions/macros, two key lists) now
loads against the staged original annex. All 28 original historical facts remain
in the recovery record. Five complete specialization/test-mode facts, the empty-
macro assertion, and the first runtime-config assertion are exercised alongside
additional exact configuration and macro-expansion contracts: 15 facts pass.
All 15 wrong-expectation controls fail without errors.

Adaptations use std.block source matching and existing native operations:
map entry sequences for option flattening, native Var type inspection, explicit
sequence realization, dynamic require forms, current symbol resolution and
require :refer for macro imports. Java and Rust already resolve -/ intrinsically;
the obsolete alias mutation is omitted, with shorthand and alias-table checks.
Generated macro call heads are explicitly qualified; the unqualified candidate
fails all four macro-expansion facts. No compatibility API was introduced.

This is a staged source, not a completed or installed script port. Populated macro
imports, reload behavior, script-fn-base, lifecycle/runtime historical assertions,
full correspondence, native-test ownership reconciliation, facade cutover and
real emit-generated process bootstraps remain pending. Existing installed script
source/tests are preserved as exact before-images and are not replaced here.

Full source/test authority, adapters, before-images, positive/negative evidence,
and executable source replay: Foundation `lang_core_script_original_stage.edn`.

## Original script module and tagged-runtime pipeline verified

The script candidate now passes 20 facts, including actual Lua module installation
into the supplied library, registered default-runtime creation, annex child-library
isolation, tagged-runtime reuse, exact pointer emission `return 1 + 2`, and tagged
stop/restart state transitions. Five wrong integration expectations produce exactly
five failures with the preceding fifteen facts passing and no errors.

Further adaptations lower nil-only defaults and negative conditions, use native
resolved-Var presence checks, dispatch through IContextEval/invoke-ptr, and use
native map-preserving entry transformation. Invocation tests release their parent
library override so original runtime-library precedence selects the annex child.
Cleanup stops annex runtimes explicitly before resetting Space and local libraries.

Pipeline testing found one installed pointer defect: ptr-invoke-meta retained
collection/transpose. It now uses native transpose, whose key/value inversion
matches the immutable Foundation implementation. The added exact regression checks
runtime inverse links, preservation of existing links, unchanged library state and
repeatability. It fails on the old helper call; all 55 pointer facts pass with the
fix, including two fresh runs of the written path-matched test and verified cleanup.
Native scaffold/correspondence reports no missing, TODO or unchecked tests. The
public scaffold preview CLI crashes with a stack overflow; no CLI write was
attempted, and the underlying native scaffold audit passes with no additions needed.

Pointer is installed. Script/annex are still staged: full correspondence, native
test reconciliation, require/reload cases, original oneshot assertions, facade and
actual JS/Lua/Python process bootstraps remain open. Default-runtime emission here
is not an external Lua-process execution claim.

Recovery, exact replay and positive/negative evidence:
Foundation `lang_core_script_pipeline_stage.edn`.

## Script candidate correspondence complete

All 29 original functions/macros and both key lists now have indexed behavioral
facts in the candidate. Thirty-five facts pass. Native scaffold, incomplete and
unchecked audits return no findings. Fifteen deliberately wrong expectations for
the newly indexed contracts produce exactly fifteen failures, twenty passes and
no errors.

Additional coverage includes original ordered config keys, absent/empty/populated
module detection, require-entry reloading and primary selection, trailing option
maps, scratch runtime behavior, each tagged lifecycle entry point, and the existing
detailed native installation assertion. Installation now restores captured model
macro Vars rather than clearing their roots. Keyword-rest adaptation preserves
trailing-map overrides and is idempotent. Incidental Clojure reader locations are
removed from authored native test forms; semantic metadata and correspondence IDs
remain intact.

This closes candidate correspondence, not migration parity or installation. Full
original process-backed assertions, remaining native assertion dispositions,
real source reload/library hydration, facade cutover and actual emitted JS/Lua/
Python process bootstraps remain open. Existing installed script/annex files are
not replaced by this stage.

Source and test regeneration, retained installation before-image, validation and
negative controls: Foundation `lang_core_script_correspondence_stage.edn`.
