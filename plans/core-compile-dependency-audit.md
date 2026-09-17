# Faithful core compile dependency audit

Authority: Foundation `fe54cc866473a888bfbd4ce2d3de6d8011f09f72`,
`src/tahto/core/compile.clj` and its matching test. Audit date: 2026-09-13.
The initial audit made no production changes; subsequent verified dependency
installations are recorded below.

## Revalidation — 2026-09-14

The installed artifact resolver now matches the complete source and test outputs
in `lang_core_compile_resolve_pending.edn` byte-for-byte. Fresh artifact tests
report 49 passing and the same one old-compiler `INormalizer` error described
below. The pending candidate filename must not be interpreted as installed
status for other functions.

The existing prelude owns `deps-ordered` (`src/std/foundation.hal`); schema
compilation does not require adding `std.lib.deps`. Fresh
`test/std/foundation/deps_test.hal` passes all six facts, covering ordering,
transitive dependencies, and cycles. Book integration still needs its own
schema compilation check.

The staged compiler from `lang_core_compile_specialization_dispatch_pending.edn`
was evaluated in a fresh native process using its recorded `:harness/stage`
output. Its five `:contracts/stage` facts were explicitly executed with
`Test/run-fact`, not merely loaded; their status vector was exactly
`[:passed :passed :passed :passed :passed]`. They cover installation before
dispatch, default-library selection, installation failure and restoration,
batch precedence/order, and stopping a batch after failure. This validates the
staged candidate only, not the currently installed legacy compiler.

For reproducing dynamic candidate checks, create the compiler/test namespaces
before `eval-in-ns`, parse source strings with `read-forms-spanned` and select
`:form`, and execute the registered facts explicitly. `read-forms` takes a file
path, and merely registering facts under another namespace is not sufficient
for the outer native test-file runner to execute them. Migration record file
bytes are decoded after awaiting `File/read` before string equality checks.

The complete source candidate is now retained in
`lang_core_compile_complete_source_candidate.edn`. It includes all 16 original
functions and six compiler-type registrations, preserving pinned source order
and section comments through `code.migrate.lang/rewrite-owned-source` and
`std.block` structural replacement. The saved adapter regenerates the complete
source byte-exactly; a second structural pass is identical.

The schema adaptation uses existing prelude `deps-ordered`, native `str/join`,
and the established dynamic-require form. A hand-authored Book-boundary test
checks dependency-before-dependent order, exact headers/footers per module,
output path, unchanged emission options, and restoration of intercepted Vars
and their metadata. Reversing the candidate's dependency order causes this
assertion to fail. The restored candidate passes. This does not replace the
original PostgreSQL scratch-v1 integration assertion, which remains pending.

Fresh native evaluation of the saved complete candidate passes two outer
checks: the schema contract is explicitly executed and passes, and all six
compiler registrations resolve to their exact original owning Vars with the
prior registry table restored. Dynamic evaluation wraps the original `declare`
in `eval` for the harness only; this adaptation is not in the production source
candidate. A registration-specific negative probe is still due before install.

Further complete-candidate validation executes 15 historical/helper and
specialization facts successfully: artifact producers/descriptors/sidecars (3),
directory links/conflicts (2), specialization descriptor (3), original dispatch
and batch facts (2), and dispatch/order/failure/restoration contracts (5).
The exact fragment paths and keys are recorded in the complete source candidate.
These facts do not cover all original compiler integrations.

The emission harness now uses installed XTalk/JS/Lua models, not the old model
copies embedded in `lang_core_compile_native_checkers_pending.edn`. Only its
saved XTalk fixtures are selected by namespace. Batch `eval-in-ns` of those
fixtures failed with `unbound symbol: var`; evaluating their top-level forms
one at a time successfully loaded all four: spec-base, common-lib, common-math,
and common-data. This is a harness evaluation finding, not a reason to change
the compiler or weaken an emission assertion. Fixtures use a cloned library,
which is reset in `finally`. The original graph assertions must still run
against that complete setup before claiming emitted-module parity.

The first sequential graph run was deliberately terminated (exit 143), not
classified as an assertion failure or timeout. PID 41238 was confirmed live
at 100% CPU; a one-second macOS sample showed heavy formatting/allocation. The
harness printed entire failed fact results, potentially including large Books.
Its replacement prints only fixture/fact names, status, and elapsed time.
Do not infer a production compiler hang from this diagnostic run. The local
sample is `/tmp/hara-native_2026-09-14_182139_cfun.sample.txt`; inspect only
bounded sections, not the full call graph.

The subsequent bounded harness exposed a separate mistake: code.test's
`*eval-mode*` defaults true, so registering facts via dynamic evaluation also
executed them. That diagnostic was terminated (exit 143). The corrected harness
binds `code.test.context/*eval-mode*` false during registration and explicitly
runs facts once in their native registration order. It has confirmed the
original preparation, module.root, and module.graph facts pass with the installed
models. The graph fact completed in 127295 ms, with the outer assertion checking
exactly three passing statuses; native session 89852 terminated with exit zero.
The exact corrected harness and terminal observation are saved in
`lang_core_compile_installed_models_emission_validation.edn`. The graph's high
runtime merits investigation, but is not a pending result. No production
compiler/source-test pair was replaced by these harness experiments.

## Installed TypeScript dependency and historical integrations

The installed TypeScript sidecar still called the nonexistent
`lang.typed.xtalk-parse/analyze-module`, whereas the pinned original uses
`analyze-namespace`. This caused both directory-selected and single-sidecar
integration facts to error. `src/lang/model/v1/spec_js/ts.hal` now uses the
original analyzer and `emit-namespace-declarations` definition; the unused
catalog facet and renamed `emit-module-declarations` wrapper were removed.
Previously staged collection adaptations are installed too, with a new nested
type-reference regression that fails against the pre-change implementation.

The newly written path-matched TypeScript test file passes 30 facts in a fresh
native process: 24 retained historical facts, two explicit analyzer-boundary
contracts, three constant contracts, and one compound-type regression. Native
unit inventory has no missing, unchecked, or new scaffold obligations. Scaffold
CLI preview/write still abort with exit 134; do not report that CLI gate green.
Both written files regenerate byte-exactly from the saved structural adapters
and hand-authored additions. Final-newline canonicalization is recorded.

Recovery, full before-images, replay, and the three still-deferred original
typed-fixture facts are in `lang_model_js_ts_original_owner_installed.edn`.
The clean native inventory is not full historical TypeScript parity: the
missing original typed fixture and its three original assertions remain due.

After this installation, all four original integration facts pass against the
complete compiler candidate and installed models: directory-selected (122005
ms), JavaScript single-module sidecars (4622 ms), Lua single-module (2798 ms),
and directory discovery (3149 ms). The last run exited zero, with fact
registration separated from execution and cloned library/type-registry state.
`lang_core_compile_four_integrations_verified.edn` retains the exact harness.

Together with the earlier helper/specialization and graph checks, this verifies
16 of the pinned compiler's 17 historical facts after the fresh `compile-script`
check recorded below. The PostgreSQL schema integration remains. The complete compiler source itself is
still staged; installation also requires replacing its old diagnostic-map
consumers without preserving the rejected catalog compiler.

The original `compile-script` fact now passes against the complete candidate
and installed XTalk/JS/Lua models (1845 ms, native session 73918, exit zero).
Its pinned output assertion is unchanged; fixtures use a cloned library and
isolated typed registry, restored on exit. The previous process handle 97473
was unavailable, so no result was inferred from it. The fresh result and exact
reproduction program are in
`lang_core_compile_script_installed_models_verified.edn`.

## PostgreSQL original-owner cutover — 2026-09-14

The original PostgreSQL root is now structurally staged in
`lang_model_postgres_original_root_pending.edn`, with the complete pinned
source/tests and native before-images retained. It restores the original
features/template/grammar/Book/install composition, not a catalog-compatible
substitute. The root is not installed.

Its first fresh load failed because `postgres-form-defn` was a facade over
`postgres-native`, which imports the rejected catalog registry and fails on
`IGrammarCatalog`. The function emitter now directly owns its original source
in `src/lang/model/v1/spec_postgres/postgres_form_defn.hal`. The approved
structural adapters retain original function order, existing native
attribute-map precedence, and the native `:postgres` language tag extension.
`postgres-native` and its other helper definitions were not changed.

The paired written test file passes nine behavioral contracts in two fresh
processes (55089 and 52965). A wrong Python tag makes the language-table and
Python-emission contracts fail (40818). A separate written-source, byte-match,
and native unit-inventory check passes three facts (15711), with no incomplete,
unchecked, or new scaffold obligations. The CLI scaffold preview/write still
abort with exit 134. Source and tests regenerate byte-exactly from the saved
`std.block` adapters; recovery and evidence are in
`lang_model_postgres_form_defn_original_owner.edn` and its `_installed` record.
The original complete SQL assertions are retained but still pending; these
nine contracts are not a claim of historical SQL integration parity.

With the original function owner, the root candidate advances to `+features+`
and fails resolving the `form-deftype/pg-deftype-format` Var hook. The existing
table facade exports a formatter, but its own focused test also fails:
three facts pass and formatting errors with unsupported `INth/nth` (82083).
The native hydration owner also lacks the original
`pg-deftype-hydrate-hook`. Restore the original table-definition and hydration
owners in dependency order, then validate the staged vector and lifecycle
owners together with the root before retrying the compiler's schema fact.

### Original table-definition owner installed

`postgres-form-deftype` now owns the full original implementation, replacing
its prior publication-only facade. Existing native helper files remain intact,
and the previously exported native hydration entrypoint is retained and tested.
The pinned source/test inputs, both native before-images, structural adapters,
and exact replay function are saved in
`lang_model_postgres_form_deftype_original_owner.edn`.

All 24 historical facts execute: 19 top-level facts plus five previously
uncounted nested facts. Two assertions inside `with-redefs` are now collected
as an exact vector assertion, not evaluated without checking. Native rebinding
restores both Var value and metadata. The original expected results remain;
two malformed partition fixture shapes were corrected after the isolated
pinned Clojure function confirmed both original inputs throw. The original
inputs remain as two additional rejection tests.

Six additional contracts cover reference names, the retained native hydration
result, those invalid inputs, exact formatter scopes/metadata, and failure-path
Var cleanup. Candidate and written pairs pass all 30 tests; a negative candidate
fails all 30 with zero errors. Fresh written-source/byte-match/unit-inventory
validation passes three facts with no missing, unchecked, or scaffold-new
obligations. Scaffold CLI preview/write still abort with exit 134. The saved
adapters regenerate both written files byte-for-byte, and scoped diff checking
passes. Terminal evidence and SHA-256 values are in the `_installed.edn` record.

The direct original-root probe now gets past the table formatter and stops at
the absent `postgres-form-deftype-hydrate/pg-deftype-hydrate-hook` (82309).
A dynamic root probe without preloaded schema dependencies produced an
`expand-scopes` error; this is a harness-load distinction, not a schema test
regression. The schema base's 19 tests and direct table loading pass.

Next dependency order: complete the relational schema constructor's `ref` and
`impl` dependencies, then the original hydration namespace and its application
registry hook. The existing registry owner is `postgres.application` under
`src-lang`, not a new `lang.runtime.postgres.base.application` wrapper. Audit
its original Book integration before connecting the hook: it still contains
`script/+modules+` lookups. Do not substitute its `schema-from-tables` map for
the original `schema/schema` result, whose `:vec` is consumed by hydration.

The project nREPL was genuinely live but unhealthy (even arithmetic failed with
compiler stack overflow). It was restarted with `clj-repl-ctl`, and migration
evaluation/replay succeeded in the replacement server on port 53838. No
Foundation `src/tahto` or `test/tahto` files were written.

Before installation, assemble the complete historical test pair, and preserve
and replace consumers in `lang.core`, artifact
module dispatch, and PostgreSQL client compilation; they still expect the
rejected `compile-module` diagnostic-map API.

## Existing owner, not a new compiler utility namespace

The original `std.make.compile` output and registration helpers already have
an owner in `work.flow.make.artifact`: `compile-fullbody`, `compile-out-path`,
`compile-write`, `compile-result-seq`, `compile-write-artifacts`,
`compile-summarise`, `compile-resolve`, `types-add`, `*compile-filter*`, and
`*mock-compile*`. `work.flow.make.compile` instead constructs Work plans and
is not the correct alias substitution for the original language compiler.
Existence is not full historical parity: current artifact tests exercise the
mock write path; real write/delete receipts and all original branches still
need explicit comparison.

## Dependency order within the original namespace

1. Artifact producer resolution and recursive descriptor normalization;
   then `compile-module-artifacts`. Preserve Var, symbol, and function producer
   cases and the historical ordered primary-plus-sidecar assertion.
2. `compile-module-single`, using `impl/emit-options`, lifecycle module setup,
   and the existing artifact owner. Preserve emitted full-body transforms.
   `compile-script` separately uses pointer metadata and entry resolution.
3. `compile-module-create-links` and specialization conflict detection; then
   `compile-module-directory-selected`, using snapshot/book/dependency imports,
   links, filters, single-module compilation, and artifact summarization.
4. Directory discovery/loading and `compile-module-prep`; then root and graph
   wrappers. The original root and graph both dispatch with `:graph`.
   Native source discovery and dynamic require need explicit adaptations;
   do not copy a `.clj` scan into native production behavior silently.
5. The already staged specialization descriptor, then specialization dispatch
   and batch specialization. Install the specialized module before dispatch;
   retain exact descriptor precedence and the original option removals.
6. Schema compilation and compiler-type registration, with registry baseline
   restoration in tests. Only then validate facade publication and consumers.

Do not replace the existing native graph compiler wholesale without recovery
and comparison evidence. Do not substitute placeholders for unavailable
original dependencies or weaken the pinned output assertions.

## Fresh installed-source checks

Commands run from `technology/hara` using
`../hara-native/core/rust/target/release/hara-native`:

- `test --project . --file test/lang/core/compile_links_test.hal`:
  12 passed, zero failures/errors.
- `test --project . --file test/work/flow/make/artifact_test.hal`:
  42 passed, zero assertion failures, one error. The case
  `refuses to write an artifact when module compilation has errors` loads
  `lang.core.compile`, whose `lang.base.compiler` dependency fails with
  `unbound symbol: lang.protocol.compiler/INormalizer` at line 39.

This is an observed current-worktree failure before any compiler edits in
this slice, not evidence of a regression introduced by the staged descriptor.
The suite does not establish that the original compile pipeline is installed.
The next implementation should target the original functions above while
keeping the incompatible replacement path and its tests visible in evidence.

## Lua directory emission finding

The historical directory-selected fact reaches Lua module export emission and
fails with `:key :table`, `:symbol nil`, and the intact form `(tab ...)`.
A fresh isolated probe reproduces the distinction: `(cons 'tab [[:a 1]])`
destructures and emits correctly, but `(cons 'tab (seq [[:a 1]]))` retains
`first = tab` while vector destructuring binds its head to nil.

Native `lua-module-export` introduced that extra `seq`; the pinned Foundation
function conses directly onto `module/module-entries`. Removing the added
conversion restores successful emission of a populated export. The correction
and its path-matched regression are now installed. The original native suite
passed 37 facts; the candidate and freshly written source/test pair pass 38.
The new regression fails against the pre-change source and checks destructuring
and exact emitted Lua as well as the form's operator. Native `code.manage.unit`
reports no missing, TODO, or unchecked tests. Both scaffold CLI invocations
still crash with stack overflow. Standalone `run` lacks the generated project
catalog; fresh project-aware source-plus-test validation passes.

Reproduction, original native source, structural adapter, candidate, negative
probe, and directory test program are retained in Foundation's
`resources/code/migrate/candidates/lang_model_lua_export_original_collection_pending.edn`.
The record was read back exactly; rewriting the candidate again is idempotent.
`lang_model_lua_export_original_collection_installed.edn` records the full
validation and reconstructs both written files byte-exactly.
The populated export was also generated twice identically and executed through
`tool.sh/run-record` on local Lua and LuaJIT: both returned `7\n`, exit zero,
and empty stderr. `lang_model_lua_export_runtime_verified.edn` retains the
one-fact/two-interpreter probe. This does not establish runtime.basic bootstrap
parity.
The standalone dependency-selection contract for
`compile-module-prep` passes one fact; this is not whole-compiler parity.

The full historical directory fact initially emitted successfully but failed its
nested positive-integer matcher. Foundation `contains-map` coerces values through
`common/->checker`; native `contains` requires an explicit `satisfies` wrapper.
The initial predicate translation also used nonexistent native `int?`. Native
`long?` preserves the original fixed-precision `pos-int?` boundary. Two fresh
facts verify positive counts and rejection of zero, negative, missing,
fractional, and arbitrary-precision values. The unchanged historical assertions
now pass with this adaptation: one directory-selected fact with three
assertions, followed by all three prep/root/graph facts.
`lang_core_compile_directory_graph_verified.edn` records the results and links
to the byte-exact test/program replay. These are staged compiler tests, not an
installed production compiler.

The insertion originates in `code.migrate.lang/rewrite-lua-model-stage-form`.
A corrected rule has been tested in a disposable Clojure namespace, including
negative, exact-form, direct-cons preservation, and idempotence checks. It is
recorded in `lang_model_lua_cons_rule_pending.edn`, not installed in `.clj`:
managed gate REPL 58892 still throws `StackOverflowError` compiling `(+ 1 2)`.
No restart or gate bypass was performed. Regeneration through that old rule
alone can still reintroduce the defect; use the recorded replay correction
until the guarded rule change is installed.

## Directory discovery stage

The original discovery function is staged with a native `.hal` predicate and
path extraction from `std.fs/select` entry maps. Namespace reads use
`code.project/read-forms` and `namespace-from-forms`, joined in input order
through `Promise/run`, `Promise/all`, and `Result/synchronize`. Native require
uses an evaluated vector libspec. The original diagnostic and namespace flow
is retained without introducing a filesystem helper API.

The selection/order/options contract and unreadable/undeclared namespace
diagnostics both pass and both fail against an empty implementation. The
original discovery-to-Lua-compilation fact also passes.

Native `binding` cannot replace ordinary function Vars. The test-only
`with-redefs` adaptation snapshots values and metadata, publishes temporary
roots with `intern`, and restores them in `finally`. A separate regression
proves nested restoration and exception cleanup, with a failing deliberate
expectation. Function-root comparisons use native `=`, whose Function branch
uses pointer identity; `Base/identical?` accepts only map/record objects.

Source, original test, adapters, focused/negative checks, historical program
replay, and recovery evidence are in
`lang_core_compile_directory_discovery_pending.edn`. A transport-truncated
embedded program field was replaced by an exact replay recipe. No production
compiler source or test was overwritten.

## Next compile-script prerequisites

- Foundation `std.make.compile/compile-resolve` dereferences Vars and pointers
  before resolving symbols or invoking functions. The current artifact owner
  lacks those two branches and differs on unresolved symbols. Existing native
  tests cover literals and unresolved symbols, not the original pointer path.
- The pinned `compile-script` fact asserts an exact duplicated JavaScript-style
  `gcd` body despite supplying `:lang :lua`. Preserve it and inspect the
  pointer-derived language metadata; do not substitute a string predicate.
- The current common-math fixture rewrites `gcd` into an imperative body. The
  pinned authority is `src-lang/xt/lang/common_math.clj`, including its
  `defspec.xt` registrations. Faithful math/typed prerequisites are required
  before claiming that exact script assertion passes.

## Compile-resolve prerequisite stage (2026-09-14)

The pinned helper is now structurally adapted, preserving its original
docstring, metadata, branch order, and recursive control flow. Native mappings
use `Base/type` for Vars, `pointer?`, `function?`, and `deref`. The original
unresolved-symbol failure is restored explicitly; the conflicting native
return-symbol assertion is retained in the recovery before-image.

Seven focused facts pass: literal callable collections, recursive Var/symbol/
function chains, unresolved symbols, producer failures, actual native Pointer
dereference, pointer-producer failure with runtime root/metadata restoration,
and preservation of other dereferenceable values. The old implementation has
three passing facts, three assertion failures, and one pointer-invocation error.
The pinned historical helper fact contains no assertions; these checks are
hand-written semantic coverage, not claimed historical coverage.

The existing full artifact suite has the same 42 passes and one INormalizer
loading error before and after. The staged path-matched test pair adds the
seven checks and reports 49 passes, zero assertion failures, and that same
existing error. Production source and tests remain unchanged in this slice.

`lang_core_compile_resolve_pending.edn` in Foundation's migration candidates
retains both current-file before-images, pinned forms, adapters, complete
candidate pair, focused negatives, and full-suite evidence. Read-back equality
and byte-exact source/test replay are verified. The scaffold CLI preview and
write both still abort with stack overflow; the native inventory API runs but
does not recognize legacy Test/check correspondence. Neither full inventory
completion nor production compiler completion is claimed.

## Compile-script and original math stage (2026-09-14)

The pinned compile-script fact now passes without changing its expected path
or duplicated JavaScript gcd body. The math source is taken directly from the
pinned blob: only the namespace owner is adapted to `lang.core.script` and
`lang.typed/defspec.xt`, with the native round override. All 29 original spec
declarations, macro arities, expressions, docstrings, and metadata are retained.
No additional typed implementation change was needed for these registrations.

Two old-port differences are demonstrated, not merely inferred: the current
imperative gcd body fails the historical exact-output assertion; replacing
defspec declarations with schema metadata leaves all 29 spec types absent in
the typed registry. Both corresponding checks pass with the pinned source.

A final isolated run passes three facts: the historical compiler assertion,
byte-identical repeated compilation followed by actual Node execution returning
`[6,10]`, and exact numeric function signatures for all 29 declarations. Library
snapshot, settings, and type-registry restoration is verified by the explicit
`compile-script-baseline-restored` marker. An initial harness placed fact:global
after the facts and did not execute teardown; the final harness configures it
before fact registration. The initial green check is not cleanup evidence.

Foundation candidate `lang_core_compile_script_math_pending.edn` preserves the
pinned source and 29 historical math facts (87 JS/Lua/Python assertions), current
native math source/test before-images, compiler/test adapters, negative checks,
Node execution, and the final combined run. The historical math runtime suite
has not been run. Source, test, and complete program replay are byte-exact;
math regeneration is idempotent. Production math/compiler files remain unchanged.

The next compiler slice is specialization dispatch and batch specialization,
followed by schema fixtures and type registration. Full math runtime tests and
the existing native compiler recovery/cutover remain required before completion.

## Specialization dispatch and native IFn (2026-09-14)

Original specialization dispatch and batch source are staged with all previously
verified selected/directory/root/graph dependencies. The first historical test
failed because Rust could not invoke the selected Var; batch compilation passed.
The user explicitly corrected the boundary: native Vars and pointers must be
IFn values. The temporary explicit-deref adapter was rejected, not installed.

Rust now resolves callable Vars at invocation time, follows dynamic bindings,
preserves direct-native and interpreter-fiber dispatch, and rejects cycles using
Var identity. Pointer IFn dispatch already existed. The native language suite
passes 44 tests; the release CLI was rebuilt. JVM invokeCallable already handles
Vars and pointers, so no JVM implementation was changed. See hara-native's
`docs/callable-vars.md`, including the separately observed direct-native nested
yield failure that also occurs without a Var.

The rebuilt runtime passes both historical specialization facts with the
original Var call intact. Five additional facts cover all dispatch branches
(including unknown-kind graph fallback), installation-before-compilation,
exact retained/removed options, default library selection, installation failure
with function-root/metadata restoration, both batch arities, descriptor option
precedence, ordered processing, and stopping after a batch failure.

`lang_core_compile_specialization_dispatch_pending.edn` preserves source and
test inputs, structural adapters, the rejected workaround as historical evidence,
pre-fix failure, and post-runtime-fix results. Read-back and source/harness/test/
contract replay are byte-exact. Production compiler files are not yet replaced.
Real specialized-module emission, schema fixtures, compiler registrations, and
the full native compiler recovery/cutover remain outstanding.

## Specialization integration investigation (2026-09-14)

A real, self-contained JavaScript module now passes specialization through the
original compiler dispatch: the clone is installed with its own self-link, the
source module remains unchanged, and the emitted double-value/calculate functions
execute in Node with exact output `[12,20]`. Only artifact writing is mocked;
module installation, emission, and target execution are real. This is one passing
integration fact, not a production compiler cutover.

Two earlier probes remain recorded as failures. Directly specializing the XTalk
math module as a JavaScript source fails because the source is absent from the
raw JavaScript book. A JavaScript module requiring that inherited XTalk module
fails clone creation with `Missing namespaces`. The pinned library-snapshot
implementation also passes a raw book to module-specialize, whose module-create
checks require availability. This source trace is not yet an executed Foundation
parity check; no merged-book workaround has been introduced.

`lang_core_compile_specialization_integration_pending.edn` records both failures,
the passing self-contained probe, and a follow-up probe for repeated output and
idempotent baseline restoration. That first follow-up reached successful Node
execution but errored because the test used an unbound native `assert`; a fifth
program uses explicit failure checks instead. Read-back and all five complete
native program replays are byte-exact. The corrected follow-up result is pending;
failure sensitivity and the inherited-dependency classification also remain open.

Follow-up: the corrected test passes with byte-identical repeated compilation,
source preservation, successful Node execution, and explicit library/settings
baseline checks after two resets. A deliberately wrong expected output `[12,21]`
fails as an assertion (not an error), with cleanup still verified. Restoring the
correct `[12,20]` expectation passes again in a fresh native process. The record
now contains all six byte-exact replay programs and the final passing run.

Pinned Foundation install-module-specialized, module-specialize, and module-create
functions were evaluated from the immutable source records against local snapshot
data. They reproduce `Missing namespaces` for the inherited XTalk math dependency.
The scoped function replacements restore their original roots and metadata. Thus
this raw-book dependency limitation is preserved Foundation behavior, not a reason
to introduce a merged-book adapter. The original broader migration still requires
schema fixtures, compiler registration, and production source/test cutover.

## PostgreSQL model dependency stage (2026-09-14)

The current PostgreSQL model still uses grammar-api/registry selection instead of
the pinned direct grammar construction and script/install. It cannot establish
faithful schema-test parity. The pinned model inventory contains 14 namespaces,
151 top-level facts, and 185 top-level assertions. Model-internal dependency layers
are: common-tracker/entity-util/meta/tf; common; form-defconst/form-defpartition/
form-defrole/form-deftype-hydrate/form-let/form-vec; form-defn/form-deftype; root.
External dependencies, including the application owner, must be checked before
advancing through these layers; the layers alone are not a complete port queue.

The original tf source is staged first. Current native source passes five of its
eight historical facts, but has an omitted-tag bounds error and wrong nil/keyword
tag strings. The faithful candidate passes all eight facts plus an input-order
contract. The original two-entry Clojure array-map fixture is represented by an
explicit native Algo/ordered-map; its literal expected output is unchanged. A
reverse-order contract proves the candidate does not sort object keys and fails
against the current implementation. Native mapcat receives seq of map entries;
otherwise native source-sensitive map reconstruction rejects scalar output.

Candidate and negative evidence, original source/tests, native before-images,
structural adapters, dependency layers, and byte-exact source/test/program replay
are saved in lang_model_postgres_tf_original_pending.edn. The current source also
owns extra pg-json-value/free-data/free-vec helpers, so their preservation remains
part of cutover. Scaffold preview and --write both exit 134 with stack overflow.
No PostgreSQL production source/test file has been replaced in this stage.

### Transform helper installed

The postgres-tf source/test pair is now installed. The eight original functions
retain their historical behavior; the three extra native helpers are retained
byte-for-byte and have additional complete-result facts. All existing tests are
retained, with explicit adaptations for the original unanchored prefix match
(`user-id` becomes `er_id`) and ordered map input. Set sequence destructuring
requires materialization as a vector; a probe disproved the initial nil-empty?
diagnosis, and the rejected termination rewrite was not installed.

The complete candidate and written focused suite both pass 24 checks. Four
deliberately wrong helper/dynamic-binding expectations fail as assertions, then
the restored tests pass. The written source loads independently in a fresh native
process. Native unit/scaffold, missing, incomplete, and unchecked report no
outstanding definitions for the pair, both before and after writing; this is the
fallback for the installed manage CLI's documented stack overflow.

lang_model_postgres_tf_installed.edn links original/native before-images and
records structural replay, retained helpers, explicit test adaptations, negative
checks, and written validation. Read-back, source/test regeneration, and a second
regeneration match the installed bytes exactly. No Clojure language source or
test files were modified, and no commits were created. Remaining PostgreSQL
model layers and external application dependencies still precede the historical
schema fixture and compiler cutover.

### Metadata dependency staged

Original metadata has 14 historical facts. Twelve are independent; get-schema-seed
and classify-ptr require the installed PostgreSQL script/builtin fixture. Those
two facts remain saved unchanged and pending, not replaced by mocks. After native
satisfies adaptation for predicate checkers, current source passes seven of the
independent facts and fails five: enum field names, extension identifiers, flat
policy metadata for existence/drop, and the schema-existence return shape.

The staged original source passes all twelve, plus four exact contracts covering
extension seed filtering, schema-seed selection, module setup/teardown ordering,
module existence output, and policy schema override. Namespace-qualified map
destructuring is explicit, and keep receives map entries as a sequence. Original
syntax quotes are read in their owning Foundation namespace before symbol-owner
translation. No alternate grammar-api model or normalized pointer-entry adapter
is introduced.

lang_model_postgres_meta_original_pending.edn preserves source/test authorities,
native before-images, adapters, pending facts, and passing results. Source, test,
and complete program replay are byte-exact. Production metadata remains unchanged;
pointer contracts and preservation of its extra helpers/+meta+ surface remain
required before cutover. Tracker's historical tests also depend on the scratch
schema; entity-util has runtime/default-application dependencies, so model-only
topological layers must not be mistaken for a fixture-complete ordering.

### Metadata pointer lifecycle verified

Real native pointers with their own Library now exercise the staged original
classify/setup/teardown path across six entry variants. No native entry getter
is mocked. Checks preserve flat static fields, public-schema fallback, nil versus
true existing state, original enum-op distinctions, policy schema override, and
setup from form rather than form-input. Fixture snapshot/settings cleanup is
checked, including repeated reset.

Existence dispatch covers table, enum, index, function, policy, and unknown types.
The unknown branch exposed a native case-without-default difference: it returns
nil where Foundation throws. A scoped structural adapter adds the original
failure explicitly. Foundation's IllegalArgumentException message was reproduced
as `No matching clause: :unknown`; it was not inferred from a passing native test.

Combined validation passes 18 facts. Without the explicit failure adaptation,
the same checks pass 17 and fail one; restoring it passes all 18. The two original
script/builtin fixture facts remain pending. The pointer record
lang_model_postgres_meta_pointer_pending.edn preserves the probes, source adapter,
negative/positive results, and byte-exact source/test/program replay. Production
metadata remains unchanged while its extra native helper surface and original
fixture dependencies are reconciled.

### Metadata full-pair preservation stage

The full candidate now retains all twelve extra native helper function bodies and
the existing +meta+ definition byte-for-byte. Native Test/check blocks are migrated
to indexed facts with their expressions preserved before explicit original-shape
adaptations. Real-pointer fixtures replace portable-entry stand-ins for classify
and drop; flat policy fields replace nested replacement metadata. Original enum,
index, extension, and schema-existence expectations are explicit adaptations with
before-images, not broad predicates substituted for exact output.

Current source/current tests: 28 pass, one preexisting error at the removed
lang.core.script/compile-form call. Full candidate/adapted combined tests: 46 pass,
zero failures, that same one error. Current source against the same combined tests:
27 pass, 18 failures, two errors. The original pipeline assertion remains active;
it has not been commented out or skipped. It requires the real original PostgreSQL
model/emission path, so production metadata remains unchanged.

Native scaffold/missing/incomplete/unchecked inventory for the full candidate has
no missing or unchecked definitions. This does not override the failing pipeline
test or the two pending historical script/builtin fixture facts. The full-pair
record lang_model_postgres_meta_full_pair_pending.edn links authority/recovery
records, helper preservation, fixture/expectation adapters, and verification.
Read-back, full source/test replay, and a second regeneration are byte-exact.

### PostgreSQL tracker original candidate

The complete tracker unit candidate passes 14 facts, preserving the two existing
native helpers independently of the original insert/modify functions. The latter
now extract configured source fields (`id`, `time`), not output field names.
The original Tracker is represented by an open native defstruct with IDisplay.
Required tracking, ignored operations, and disabled-operation precedence are
checked against the original behavior. Current source against the combined tests
passes seven, fails five, and errors twice (missing Tracker constructor).

An initial candidate passed 13 and failed one: native structs do not satisfy
IMapType, unlike Clojure records. The hand-authored native record test now checks
instance? Tracker and exact extraction output. This does not replace or count as
the three historical scratch/Task integration facts, which remain pending with
their original source preserved. The independent historical display fact passes.

Installed scaffold preview and --write both abort with stack overflow (exit 134).
Fresh native code.manage.unit scaffold/missing/incomplete/unchecked checks pass
with no missing or placeholder obligations. The pending record
lang_model_postgres_tracker_original_pending.edn preserves immutable authority,
native before-images, structural adapters, helper source, exact candidate bytes,
verification, and replay.

The tracker source/test pair is now installed. Fresh release-native validation
passes all 14 checks both from the written path-matched test and from full written
source/test evaluation. Read-back is byte-exact, correspondence remains clean,
and scoped git diff --check passes. The installed record
lang_model_postgres_tracker_installed.edn links the recovery/replay record and
post-write results. This is unit cutover, not complete historical integration.

Launcher parity remains unresolved: bare hara-native run intentionally omits
libraries and reports unbound defstruct; installed project-aware hara run instead
reports `defstruct name must be an unqualified symbol`. The current release-native
project test runner accepts the same source and exercises the actual Tracker.
Neither launcher failure is represented as a successful standalone source run.

### Next dependency: PostgreSQL entity-util

Read the complete immutable entity-util source/test pair and current native pair.
The authority has 28 facts; the current native test file has only one Test/check.
The next stage must preserve that native test while recording its dependency on
the old script/register-library-module! setup rather than silently dropping it.

Original default-application uses the runtime selected for the requested namespace,
then reads its module metadata with errors suppressed. The current native helper
instead reads the shared Library overlay. The actual existing owners for the
original calls are lang.base.util/lang-rt (Foundation tahto.core/rt publication)
and lang.core.workspace/rt:module, not replacement core facade aliases. Inspect
and test these owners before staging the application-registry functions.

Other fidelity gaps: normalise-ref currently treats arbitrary maps as pointers
and omits Vars; native pointer? and var-sym already provide the required boundaries.
Unknown tracking and access cases silently fall back in the replacement whereas
the original no-default cases throw. Original app/addon registry tests use
with-redefs and mutate two atoms without restoring them; preserve their assertions
while adding snapshot/finally restoration and checking cleanup on failure.

### Entity-util original candidate: 31 passing facts

Staged all 28 original facts with their assertions preserved; the complete
candidate now passes these and three added contracts. Adaptations use the owning
lang-rt/rt:module functions, native pointer? and Var type/var-sym, atom/swap! for
the local counter, deterministic anonymous-function rewriting, native deref,
and expanded grouped-case clauses with explicit original unknown-value failures.
The historical with-redefs fixture uses scoped Runtime/intern root replacement
with finally restoration, not binding on a non-dynamic Var. Its two registry atoms
are also restored on success and exception; an exact cleanup contract passes.

Current native source against the same 31 checks passes 29 and fails two: real
pointer/Var normalization versus map lookalikes, and unknown preset/access errors.
The candidate passes all 31. The historical default-application fact mocks the
function itself, so it does not establish real runtime-module lookup fidelity.
That integration and preservation of the existing native overlay test remain
required before cutover.

Installed scaffold preview/--write both abort with stack overflow. The direct
native unit inventory reports missing *ns-str* coverage and six historical facts
whose => assertions are nested within let as unchecked. Do not discard those
assertions to satisfy the inventory. Resolve the diagnostics and add the dynamic
Var contract before writing the source/test pair.

lang_model_postgres_entity_util_original_pending.edn preserves the immutable
authority, native before-images, structural rule forms and evidence, complete
candidate source/tests, positive and negative results, and replay. No production
entity-util source/test changes have been made.

### Entity-util full-pair integration

The full candidate passes 35 checks. A real RuntimeDefault in an isolated Space
selects an actual.module BookModule from a local Library when the requested caller
namespace is caller.module. The two modules deliberately carry different
applications; explicit and current-namespace lookup both select :actual, while an
absent runtime module returns nil. Library/settings and Space state are reset on
every exit. Current native lookup errors on removed library-book-modules.

All 28 historical facts remain. Six pure constructor facts lift their nested let
checks to top-level => clauses without changing inputs or expectations; the
constructors may be reevaluated per check. The added *ns-str* fact checks default,
temporary binding, exception propagation, and restoration. Full native
scaffold/missing/incomplete/unchecked inventory now has no outstanding obligations.

The existing native test fails before migration on missing script/script-snapshot.
Its isolated helper is adapted to a real local Library/Space fixture, and its
original :fixture-app expectation is preserved. No obsolete script API is added.
An additional helper check verifies exception cleanup and a subsequent successful
isolated invocation. Full-pair source/test replay and second generation are exact
in lang_model_postgres_entity_util_full_pair_pending.edn, which links the complete
authority/recovery record.

The source/test pair is now installed. The written path-matched native test and
fresh full written-source/test evaluation both pass all 35 checks; post-write
correspondence is clean and read-back equals the generated bytes. Three deliberately
incorrect expectations (field priority, runtime-selected application, dynamic
binding) produce exactly three assertion failures and no errors; restoration
passes all 35. lang_model_postgres_entity_util_installed.edn records these results.

The installed launcher remains incompatible with this dependency path, rejecting
the existing :config :role in std.lib.context.space. The release-native project
runner is the verified execution target, not a claimed successful installed
standalone run. Eight whitespace-only source lines from immutable authority remain
and are reported by git diff --check. Overall model/scratch integration is still
pending; this cutover does not establish the full migration goal.

### PostgreSQL common original candidate

Read the full immutable source/test pair and current native pair. Both sources
define 38 functions. The historical test file has 38 facts, five empty. The staged
candidate preserves all 33 asserted historical facts and supplies five behavioral
facts for those empty obligations, plus a string-escaping probe evaluated against
the pinned Foundation pg-string definition. All 39 checks pass. Current native
source against the same checks passes 19, fails eight, and errors twelve.

Adapters preserve original emit-common delegation, metadata, entry-resolution
precedence, token shape, ignored first loop argument, and uppercase CASE tokens.
Native boundaries require explicit empty separator for one-argument string join,
regexp/RegExp.replace with Java replacement escaping translated, native form?
for the enum !:eval branch, and seq materialization when mapping enum values.
The normalized declaration returned by grammar-spec/format-defn is materialized
as a vector before nested destructuring; the native Seq path otherwise loses
symbols/table bindings. Syntax-quote expansion is read in its owning namespace.

Real Book/BookModule/Snapshot checks verify book-view versus live-module fallback,
snapshot entries, native emit-hook delegation, and missing-link failures. The
original get-code-entry-view throws before a fragment-only fallback can run;
the candidate preserves that behavior, not a silently repaired resolver. String
probes include apostrophes, quotes, backslashes, and newlines.

Installed scaffold preview/--write both abort with stack overflow. Native unit
inventory reports only missing +pg-query-alias+ and +pg-type-alias+ facts; no
unchecked functions remain in the staged set. Before cutover, add those tests,
reconcile/preserve the current native facts and extra one-argument
pg-deftype-ref-name overload, then run full-pair negative/post-write validation.
No production common source/tests have been changed.

lang_model_postgres_common_original_pending.edn preserves authority, native
before-images, five original empty facts, adapters, structural before/after
evidence, candidate source/tests, results, and replay. Its large record contains
the full recovery evidence rather than only the passing test summary.

### PostgreSQL common full pair installed

The full source/test pair is now installed and passes 80 checks. All 38 existing
native tests are retained as facts, with explicit fixture/expectation adaptations
where they encoded the replacement architecture. Original emitter delegation uses
emit-common; linked tests use real Book/BookModule/Snapshot fixtures instead of
no-snapshot fallback. Policy coordinates, missing static keys, assertion delegation,
and quoted constant ids follow the original behavior. Nil symbol metadata is
coerced to an empty map before assoc to preserve Clojure's nil-map behavior.

The extra native one-argument pg-deftype-ref-name overload remains, alongside the
original two-argument snake-case rule. Exact checks distinguish them. Both alias
maps now have full-value tests. Native scaffold/missing/incomplete/unchecked
inventory is clean for the full pair.

Ten deliberately incorrect expectations produce exactly ten failures, no errors;
restoring expectations passes all 80. Written path-matched tests and a fresh
written-source/test evaluation both pass 80; post-write inventory is clean.
Scoped git diff --check passes. Full replay is recorded in
lang_model_postgres_common_full_pair_pending.edn, with installed/negative/post-write
evidence in lang_model_postgres_common_installed.edn and full recovery in the
original pending record.

The installed launcher still rejects :config :role in existing book-module;
release-native project tests are the verified target. This is not a successful
standalone installed-launcher run or completion of model/scratch integration.
Before selecting the next model layer, account for external dependencies:
form-defconst requires postgres.core.impl-base, impl-insert, impl-main, and
application helpers as well as the model-local common/tracker prerequisites.

### Original partition implementation installed behind existing facade

Ported immutable form-defpartition into the existing native postgres-partition
implementation owner. The postgres-form-defpartition facade remains unchanged.
Four existing helper bodies are retained byte-for-byte, with their native tests.
The three original function bodies now preserve recursive schema/name handling
and use snapshot entry schemas. Native adaptations normalize the declaration
sequence before destructuring and use regexp/str-tag equivalents.

The historical tests contain three top-level facts and five nested facts. All
eight now run independently with unchanged assertions; the nested facts receive
explicit ids and refer metadata. Current implementation against those tests passes
two and errors six; the original candidate passes eight. Added contracts cover
keyword name normalization, three-level ancestor order and inherited schemas,
default partition non-recursion/next-key handling, and real Snapshot/Book schema
lookup with macro-options restoration. The complete owner pair passes 19 checks.

Nine deliberately wrong expectations produce nine failures and no errors, then
restoring them passes all 19. Fresh written source/test evaluation passes 19 and
post-write native correspondence is clean. Written owner and unchanged facade tests
together pass 22 checks. Scoped git diff --check passes. Recovery/replay lives in
lang_model_postgres_partition_original_pending.edn; installed and negative evidence
lives in lang_model_postgres_partition_installed.edn.

Installed scaffold and standalone launcher limitations persist; the release-native
project runner is the verified target. This validates normalized partition DDL
forms, not the still-pending full original PostgreSQL grammar/scratch pipeline.

### Original form-vec staged; root grammar parity still required

Revalidated the partition installed record against disk and regenerated its
source, tests, and negative variant exactly twice. The installed source/test
bytes match the replay; no partition edits were necessary.

Read immutable form-vec source/test in full and staged its six original functions
through code.migrate.lang/rewrite-owned-source. Existing engine rules normalize
single arities and reader anonymous functions. Explicit adapters route string,
prose, and emitter owners; use regexp; preserve error message/data; replace the
local Counter with an atom starting at the original -1; and map over seq queries
instead of reconstructing a native map. No production sorting was added.

Nine hand-written unit facts cover case-preserving operator normalization,
scalar wrapping, odd query groups, explicit AND, multiline OR, traversal order,
nested-vector indentation restoration, keyword rendering, punctuation, and empty
sections. They pass in a fresh native process. Nine deliberately wrong final
expectations fail independently, with no errors, and restoring them passes nine.
Native code.manage.unit reports no missing/incomplete/unchecked obligations for
the six-function candidate. CLI scaffold preview and write both still abort with
a stack overflow (exit 134). The verifier-only declare wrapper is not part of the
proposed production source.

All six historical facts/seven assertions remain unchanged apart from namespace
and with:emit owner routing. Existing native source against them passes one and
errors five because its helper calls the removed lang.base.emit/*emit-fn*.
The candidate passes one, fails three, and errors two through the current root
grammar: percent-wrapped scalars and newline sentinels emit literally, and set/JS
emission encounters str/join arity/type failures. These are not accepted output
adaptations. The current root still assembles grammar-api/registry definitions,
not the original features/template/grammar/book/script pipeline.

Durable source/test before-images, all adapters, indexed replacement evidence,
historical failures, focused positive/negative checks, and byte-exact replay are
in Foundation resources/code/migrate/candidates/
lang_model_postgres_form_vec_original_pending.edn. Readback equals the staged
record and reproduces all five generated outputs exactly twice. The native
fresh-process run reconstructed from the written EDN also passes all nine
focused facts (exit 0); the recorded native source/test before-images still
match disk byte-for-byte. The native
source/test pair is intentionally unchanged: its four additional helper bodies
and ten native test blocks still require full-pair reconciliation, followed by
original grammar integration and written-file validation. This is a staged
candidate, not an installed namespace or a historical parity claim.

Next source dependency: form-let (common/tf, preprocess-base/emit-preprocess,
library-snapshot, collection walk helpers), then form-defn. Keep remaining DDL
and hydration owners ahead of the root grammar/book/script installation; avoid
substituting test-only operators for the original grammar feature topology.

### Original form-let installed with historical scope and staging behavior

Ported all nine immutable form-let functions and the dynamic input-symbol Var
into the existing postgres-form-let owner. Retained the two native-only helpers,
symbol-set and pg-let-declaration, byte-for-byte. The original named block shape,
typed/set/vector assignments, declaration deduplication and sorting, nested
scope/forward-reference checks, assignment callbacks, and block scope bindings
now replace the prior custom implementations.

Copied engine rules normalize wrapped arities, reader anonymous functions and
bounds-safe nested sequential parameters. Explicit adaptations route the native
owners, convert volatile cells to atoms, named recursive functions to letfn,
and eager run! calls to doseq. Historical with-redefs of the synchronous dynamic
emitter becomes binding. Original error messages/data are preserved. A proposed
vector conversion in the declaration loop was rejected: diagnostic output proved
declarations were present there and lost at pg-tf-let-block's nested parameter.
The final source retains the original loop and uses the existing parameter rule.

All nine historical facts/15 assertions are retained. Eleven existing native
tests are now corresponding facts; they bind the actual emit-common owner and
the named-declaration fixture supplies the original typed declaration shape.
Seven new facts cover typed/destructured assignments, destination/callback
controls, a real Snapshot/Book-to-staging callback, scope boundaries, emitter
failure restoration, duplicate declarations, and the dynamic Var baseline.
Local mutable fixtures reset in finally; dynamic bindings restore on every exit.

The pre-change implementation passed one historical fact, failed two and errored
six. The full candidate passes 27 facts/33 assertions. Eight wrong expectations
produce eight failures and no errors, then restoring them passes 27. Native
code.manage.unit reports no missing/incomplete/unchecked obligations, including
the dynamic Var. CLI scaffold preview and write still stack-overflow (exit 134).

Written-file native testing passes 27; a separate fresh process evaluating the
written source/test pair also passes 27. Post-write correspondence is clean and
scoped git diff --check passes. Source/test regeneration is byte-exact twice and
matches disk. Recovery/adapters/evidence are in Foundation migration resources
lang_model_postgres_form_let_original_pending.edn; final installation evidence is
lang_model_postgres_form_let_installed.edn.

During validation, an external runtime update temporarily left the project pin
at 0.1.29 while the release binary was 0.1.30. The 0.1.29 debug binary established
the baseline; the project pin subsequently advanced externally to 0.1.30. Final
checks use matching release-native 0.1.30. No project pin was changed by this port.
The installed standalone hara launcher still rejects :config :role in a dependency.
Full original PostgreSQL grammar/book/script and generated SQL remain downstream.

Next is form-defn: its existing facade publishes from postgres-native, whose
custom emitter currently rejects Python despite the historical test requiring
plpython3u output. Preserve native-only surfaces while restoring the original
emit-fn/common and cross-language emission path; do not weaken that assertion.

### PostgreSQL symbol-metadata defaults restored

The original form-defn work exposed a prerequisite defect in installed
postgres-common/pg-sym-meta: native explicit-key destructuring lost absent-key
defaults. The existing code.migrate structural map-destructuring rule now lowers
this function to presence-sensitive lookups, with absent metadata normalized to
an empty map. Defaults remain [:jsonb], :default and []; explicitly present nil
values remain nil, and unrelated metadata survives. The pinned original function
was evaluated in Clojure to establish all three exact results.

All previous 80 facts remain unchanged. Three new semantic facts make 83 passing
facts in candidate, regenerated, written-pair and direct path-matched fresh native
runs. The pre-change implementation fails two new assertions; three deliberately
wrong expectations fail independently, then restoring them passes 83. Native
code.manage.unit correspondence has no missing, incomplete or unchecked tests.
CLI scaffold still aborts with stack overflow (134), and the installed standalone
launcher still rejects dependency :config :role; those paths are not claimed green.

Regeneration chains through lang_model_postgres_common_full_pair_pending.edn,
matches twice byte-for-byte and matches the written source/test pair. New recovery
and verification records are lang_model_postgres_common_defaults_pending.edn and
lang_model_postgres_common_defaults_installed.edn in Foundation migration resources.
Scoped git diff --check passes. No immutable tahto source or tests were changed.
Original form-defn SQL parity and the downstream PostgreSQL grammar remain pending.

### Original form-defn staged; exact SQL parity still red

The four original form-defn definitions now have a reproducible std.block stage
targeting their existing postgres-native owner, not a new implementation namespace.
The stage restores emit-fn type/preamble handling and emit-common's composed body,
including actual !:lang delegation for Python, SQL scalar wrapping, PLpgSQL blocks,
schema/property forms and input-symbol scope. All unrelated native owner forms and
the facade remain untouched. The :postgres language-table extension is retained.

The existing native formatter accepts defn attribute maps and gives them precedence
over symbol metadata. A scoped adaptation retains that behavior only when an
attribute map is present; the symbol-only path follows Foundation, including entry
versus symbol documentation. The original typed-argument helper confirms :assign
true for := defaults. New tests assert exact metadata for both paths, complete
normalized emitter forms, language tags, alias identity, SQL rejection, and scope
restoration after an emitter error. These normalized-form tests are not SQL parity.

Serial fresh-process validation: nine unit facts pass; nine deliberately wrong
expectations produce nine failures and no errors. Combining the two historical facts
with the nine unit facts gives ten passes and one error fact: the original Python
SQL assertion raises `str/join expects a string`, and that same fact's PLpgSQL
assertion emits incorrect keyword/character text under the current root grammar.
Neither historical SQL expectation was changed. The original formatter fact passes.
Full-owner correspondence is red because postgres-native lacks path-matched tests
for its many retained native definitions; existing root tests still need reconciliation.

Foundation resource lang_model_postgres_form_defn_original_pending.edn contains
the pinned source/tests, structural rules, whole-owner before-image, unchanged
native root/facade test recovery, nine handwritten contracts, negative controls,
full-pair and inventory failures, and the replay function. All six replay outputs
match twice byte-for-byte after EDN read-back; regenerated unit tests pass nine in
a fresh process. A truncated transport capture was detected by equality checks and
replaced with a verified complete record. No form-defn HAL source/test was installed.

Continue with actual PostgreSQL grammar/book dependencies and full native-owner
test reconciliation. Root spec-postgres still imports the rejected grammar-api and
registry architecture. Do not fake a grammar or weaken SQL assertions to install
this stage; preserve the original dependency order and cross-language parent Books.

### Relational schema base installed; native multimethod boundary repaired

Original table hydration requires std.lib.schema/schema and its application hook
requires the original relational schema shape. Native std.typed.schema is a distinct
typed validator, not a replacement. The current postgres.application schema builder
is simplified and still needs faithful reconciliation. Dependency-first work now
starts with the original std.lib.schema.base source/test pair at the pinned revision.

Installed src/std/lib/schema/base.hal and its path-matched test. The scope catalogs,
scope checker, attribute/default helpers, and real type-checks multimethod retain
their original meanings. Existing copied rules lower anonymous functions, filter
values, and nested sequential parameters. Native declarations retain multimethod
doc/added metadata on the owning name; defmethod's single arity is flattened.
The existing fn? predicate preserves IFn acceptance, including Vars, pointers and
maps. Clojure float? accepts Float and Double; native double? covers the native
floating representation while rejecting integers.

The original base-meta hash-map iteration order is captured from the pinned reader
and explicitly represented by Algo/ordered-map. This preserves both historical
default-order assertions without sorting production results. Scope expansion's loss
of intermediate edges is documented; source graph provenance remains available and
canonicalization idempotence is tested.

All eight historical facts remain, with reader-only expansion of a namespaced map.
Eleven handwritten facts add exact scope/default/error/metadata contracts, native
callable predicates, argument-tail preservation, caller-order preservation, and
primitive dispatch branches. Nineteen facts pass in candidate and fresh written
pair/direct-path tests. Eleven deliberately wrong expectations fail independently,
then restoration passes nineteen. Native code.manage.unit reports no missing,
incomplete or unchecked obligations. CLI scaffold preview/write return no records
before a new source exists, then both stack-overflow after installation; the native
candidate inventory is the meaningful correspondence evidence. Standalone hara run
still reports unbound defonce, so that launcher is not claimed green.

The first real multimethod exposed a runtime bug also reproduced by the unchanged
Foundation test file and minimal release/debug probes. Nested direct-native callback
contexts shared a registry but swapped/restored stale snapshots, losing registrations.
The native runtime now keeps the active map when nested contexts share that registry.
The new regression covers both backends, nested and later callbacks, exact dispatch,
and failed-declaration rollback; fresh threads release the thread-local baseline on
every exit. It fails before the implementation change and passes afterward. The full
native-lang suite passes 45 with existing loopback fixtures, and the release runtime
was rebuilt. The Foundation test file advances past registration but then reports
unbound def; that later failure remains unresolved, not a passing broad-suite claim.

Recovery/rules/evidence: relational_schema_base_pending.edn and
relational_schema_base_installed.edn in Foundation migration resources. Source, test
and negative-test regeneration match twice byte-for-byte after read-back; written
files match the record. Native runtime details are in hara-native's
docs/multimethod-declarations.md. No tahto files, commits or publication were changed.
Next: remaining relational schema find/ref/impl dependencies and facade, then the
original application/hydration hook and PostgreSQL grammar. Form-defn remains staged.

### Relational schema lookup installed; reference rules staged

Installed std.lib.schema.find and its path-matched tests. The three historical
facts and four additional attribute-preservation, empty-input, identifier-selection,
and reverse-reference boundary facts pass in fresh candidate and written-file tests.
Native empty keys return [] whereas the original all-idents returns nil; the scoped
not-empty adaptation preserves that original result. Four deliberately wrong new
expectations fail independently. Native unit correspondence is clean and the normal
launcher loads the written file. Post-install CLI scaffold still stack-overflows;
that tooling check is not claimed green. Source/test/negative regeneration and disk
bytes match after record read-back. Recovery: relational_schema_find_pending.edn and
relational_schema_find_installed.edn in Foundation migration resources.

The next original dependency, std.lib.schema.ref, is staged but not installed.
Copied structural rules lower arities and sequential parameters; local string-path
operations use existing native primitives without porting std.lib.invoke. The macro
retains its owning dynamic Var and returns native List code instead of a Seq. Native
function-level recur needs an explicit loop, preserving the original grouping body.
Per the user's correction, sequence/list/nil prepending uses cons explicitly; vector
conj remains append. This preserves the original mixed-group ordering rather than
changing the historical expectation.

All eleven historical facts plus a new four-branch cons/append contract pass (12
facts). Its deliberately incorrect ordering expectation fails. Additional callback
nesting/failure-restoration, forward/reverse metadata, naming, selection and grouping
contracts exposed and verified the macro fix. The larger 22-fact candidate has 19
passes and three errors: native keyword parsing and construction both reject deeper
paths such as :a/b/c. These are added fidelity probes, not removed historical tests.
No keyword contract change has been made. The compound-name expectation was checked
against the pinned original determine-rval definition; it is :work-email-big-accounts.

relational_schema_ref_pending.edn retains the immutable source/tests, quoted structural
factories and adapters, candidates, new contracts, explicit cons negative control,
test results, absent target before-images, and replay function. Source, historical
test, complete staged test and negative test reproduce twice byte-for-byte. The
reference pair still needs a decision on unsupported deeper keyword representations,
full new-contract negative controls, correspondence and written-file validation.
Do not claim it installed or bypass the original relational schema dependency order.

Reference contract audit follow-up: the native inventory identified the dynamic
*ref-fn* Var as an additional test obligation. Its direct invocation under binding
and restoration to nil now has an exact assertion. Ten runnable added contracts
pass, ten deliberately wrong first expectations fail without errors, and restoration
passes ten again. Expected-throw assertions remain intact in these negative controls.
Native unit scaffold/missing/incomplete/unchecked correspondence now passes. The three
deep-keyword probes remain in the complete staged tests and are explicitly excluded
only from this independently named runnable-contract validation, not silently dropped.

relational_schema_ref_contract_audit.edn preserves the parent before-images, added
dynamic-Var contract, negative-control adapter, inventory input/results and repeatable
replay. The next owner std.lib.schema.impl has seven historical facts and requires
the original Schema fields flat/tree/lu/vec plus tree-flatten and tree-nestify, neither
of which was found in native source. No deeper keyword literals were found in the
pinned relational schema source/tests or inspected PostgreSQL dependency paths;
runtime-generated deeper identifiers are still possible. Native keyword semantics
have been left unchanged pending user direction. No reference HAL pair was installed.

### Original Python basic-client exchange diagnostic running

The keyword decision does not block independent process validation. A fresh native
diagnostic now composes the existing replayed RuntimeBasic/server/bench prefix with
the saved Python spec-require stage. The original default-basic-client generates its
bootstrap through emit-entry-deps and emit-as; the installed hand-written source-python
transport is not used as evidence for this requirement. The new test uses the original
RuntimeBasic lifecycle and checks six exact JSON payloads (including false, nil and
Unicode/newline) plus empty server/bench registries after stop, with finally cleanup.

This is explicitly dependency-isolated: that saved stage omits xt.lang.common-promise.
The full-required stage remains red at the stale lang.core runtime/bootstrap export.
It is not a substitute for that dependency, original script/annex fixtures, or a full
Python namespace cutover. No production source/test was written. The diagnostic input
reproduces twice byte-for-byte. Its live handle, limitations, source before-image and
test are saved in lang_runtime_basic_python_exchange_diagnostic.edn. At checkpoint,
nREPL 50370 future python-exchange-job and native PID 64956 were live and CPU-active;
poll that same future and record its terminal result before claiming any exchange.

Python diagnostic terminal follow-up: python-exchange-job completed successfully.
Its one fact exercised six exact payloads through the original emit-generated CPython
client and original RuntimeBasic server/bench lifecycle, then verified both registries
were empty. The expected bench shutdown printed {:out "" :err "" :exit -1}; the native
test process itself exited zero. lang_runtime_basic_python_exchange_observed.edn saves
the result and replays exactly after read-back. Full common-promise/core-facade and
original script/annex integration remain pending; no namespace cutover is claimed.

While that same process ran, a one-second native stack sample showed VM/protocol lookup
and allocation work. Native find_protocol now compares borrowed name slices instead
of allocating qualified/runtime names for each registry candidate. Exact-name and
malformed-name tests pass before and after. Fixed-work debug timing for 7,600 lookups
was 50.097209 ms before and 6.954459 ms after; this is not an end-to-end speed claim.
The complete native-lang suite passes 45, release rebuild passes, and that release
passes schema-base 19 and schema-find 7. Existing runtime edits were preserved.

The new nREPL 50370 future python-exchange-repeat-job runs the deliberately wrong
payload expectation and then the restored original diagnostic serially on the rebuilt
release. Poll this new handle, not the now-terminal python-exchange-job. Results and
elapsed times remain pending until it completes. Native optimization details are in
hara-native/docs/protocol-lookup-cost.md. No tahto files, commits or publication changed.

### Runtime return-transform native IFn reconciliation queued

The installed return-transform still explicitly dereferences wrapper and formatter
Vars. The pinned original instead calls (wrap-fn (format-fn forms)) directly. Besides
being unnecessary after native IFn support, early dereference freezes the wrapper
before the formatter evaluates. A formatter that changes the wrapper Var should be
followed by invocation of its current value, as the native invocation contract requires.

The structural candidate removes only those two dereference wrappers. Existing source,
historical tests and recovery comments remain intact. Added tests exercise live wrapper
replacement with finally restoration, nil/false through map IFn callbacks, and exception
propagation. The prechange/candidate/full-pair checks are queued in nREPL 50370 future
runtime-ifn-check-job, which waits for python-exchange-repeat-job before invoking any
native verifier. No HAL edit has been written and no regression result is claimed yet.

lang_core_runtime_return_transform_ifn_pending.edn saves exact source/test before-images,
the pinned owning form, adapter, structural evidence, full candidate, new contracts,
guarded replay and live handle. Both replay passes and EDN read-back match exactly.
Continue by reading that job's result, then scaffold and perform written-file validation
only if the candidate and full pair pass. Do not restart either existing job merely
because observation times out.

Return-transform follow-up: the early-dereference regression is proven (old code
returns [:old [1 false]], original-style direct invocation returns [:new [1 false]]).
Installed the structural source change and two path-matched facts. All 62 facts pass
in candidate, direct written-file and fresh written-pair runs with runtime cleanup
verified. The second new contract's deliberately incorrect nil/false expectation fails
without suppressing its expected-exception assertion; restoration passes. Native unit
scaffold/missing/incomplete/unchecked correspondence is clean. Normal launcher failure
(:config :role) and CLI scaffold stack overflow remain recorded tooling limitations.

Recovery/replay are in lang_core_runtime_return_transform_ifn_pending.edn,
lang_core_runtime_return_transform_ifn_written.edn and the final
lang_core_runtime_return_transform_ifn_installed.edn. Read-back, source/test/negative
regeneration, and actual written bytes agree. No other runtime definitions were changed
in this slice; other remaining explicit Var wrappers are not implicitly claimed fixed.

Python repeat follow-up: the positive six-payload exchange passed on the rebuilt
release in 287059 ms. The initial negative control was invalid because reprinting @
introduced clojure.core/deref. The corrected adapter uses native deref; it produced
the intended one failure (actual 42 versus deliberately expected 43), zero errors,
and empty server/bench registries. That result and replay are saved in
lang_runtime_basic_python_exchange_negative_verified.edn. The old repeat/recheck and
runtime-ifn jobs are terminal. Only nREPL 50370 future python-exchange-final-job remains
live: it is repeating the restored positive after the valid negative proof. Poll that
handle, retain the dependency-isolation limitation, and record the final result.

Python restored exchange is now terminal and passed (one fact, six exact payloads,
zero remaining server/bench entries, 287625 ms). The read-back-verified result is in
lang_runtime_basic_python_exchange_restored.edn. This remains dependency-isolated
validation of the original emitted client, not production Python namespace cutover.

The original Python default-body-wrap assignment/declaration branches now have a
structural staged correction: native form? replaces Clojure seq? for code forms.
The pre-change branch test fails with missing return forms; the corrected test
passes. Both historical wrapper facts pass separately. Actual emit-as output was
inspected and executed with CPython: declarations, subsequent assignment, false and
nil return exactly 7, 8, false and null. Before the correction that execution raises
Exception: None; after restoration it passes. No hand-written bootstrap substitutes
for the original emitted wrapper. The full staged process retains its required
xt.lang.common-promise import and therefore its unresolved core-facade dependencies.

lang_runtime_basic_process_python_body_branches.edn retains source before-images,
the owner-scoped adapter, full and focused outputs, unchanged historical tests, new
branch/execution contracts and actual results. Replay is byte-exact twice. No
production source/test files were installed. CLI scaffold preview and write both
exit 134 (stack overflow); initial relative-project attempts exited 1 and were
retried with the absolute root. All Python exchange/body verifier jobs are terminal.

### Management dependency and shared volatile-update rule

Revalidated the existing lang_core_manage_helpers_stage.edn rather than replacing it
with a parallel helper implementation: all twenty staged tests still pass. Four
additional module-selection, used/ignored-link, missing-line and alias checks pass;
all four deliberately incorrect expectations fail, and restoration passes. The
new alias expectation was corrected against the pinned Clojure helper: un-used is
not a suffix of dep.unused. Assertion-only mapv vec normalizes native MapEntry values
without altering the original helper's sequence return. Evidence and replay are in
lang_core_manage_helpers_revalidated.edn.

The written native manage baseline has four errors on removed library-create. An
isolated four-fact historical subset is not full-suite parity: its predicate
expectations need explicit native satisfies matchers, and its missing-line assertion
runs after task-driven purging in the original full suite. Do not weaken that nil
expectation or silently invent fixture line numbers. The earlier task-package stage
still records the unresolved original Task versus native Work contract. Seven task
callables, two task defaults and complete historical setup remain pending; production
manage was not changed.

A concrete shared rule gap was fixed in resources/code/migrate/rules/source.edn:
volatile! and vreset! were lowered but vswap! remained unbound despite zero planning
diagnostics. The new :clojure/vswap-swap-native dependency rule maps the update to
the existing swap! prelude. Before-rule native checks produce one error and one
failure; candidate and written-catalog checks both pass two facts, covering extra
update arguments, false results, and unchanged state after an update throws. The
pinned manage source now records this applied rule and contains no remaining vswap!
symbol. Regeneration is idempotent. lang_core_manage_vswap_rule_installed.edn retains
complete rule-document before/after images, pinned source and plans, a minimal
replay catalog, test bodies, and actual verification output. EDN read-back, replay
and written rule-document bytes agree. All jobs launched in this slice are terminal.

### Native Work management integration (user decision resolved)

The user confirmed that task machinery already belongs to work.flow.task. The
earlier Task-versus-Work decision is resolved: reuse native task compilation,
invocation, selectors, result grouping, aggregation and packaging. Do not port
std.lib.invoke/std.lib.future or introduce another Task type.

src/lang/core/manage_task.hal and its path-matched test are now written. This is
an internal management-configuration adapter, not a new task engine. All six
behavioral facts pass against the written file in a fresh native project process.
Five deliberately wrong expectations fail; the sixth ordering regression fails
against the pre-change adapter, then passes after restoring original order-by
semantics for results, aggregation and finalisation without reordering execution.
Native code.manage.unit reports no missing, TODO, unchecked or new scaffold
obligations on the actual written source/test pair. CLI scaffold preview/write
returned zero records before the new namespace existed; those exits are not
counted as correspondence evidence.

Seven original management task definitions now generate compact native Work
bundles. Their candidate passes all 17 historical facts with the written adapter,
including verified library instance/settings cleanup. Two real-library selection
facts and three bulk/package/summary/error facts also passed the compact candidate
before the ordering addition. Historical literal assertions remain unchanged;
predicate matchers and Work invocation/error transport are explicit adaptations.
The complete facade import is not exercised: the fixture uses actual library
owners because full core facade migration is still unfinished.

The installed hara run launcher still rejects :config :role. Its failed standalone
run is recorded separately from successful fresh native project tests, which load
the written source. The initial disk inventory probe incorrectly treated File/read
as text; decoding its dereferenced bytes fixes that probe, and the written-file
inventory passes. No runtime changes were made for either harness issue.

lang_core_manage_native_work.edn retains source/test before-images, immutable
authority linkage, a parent-record SHA-256, original task forms, structural
factories/adapters, exact outputs, regression tests and terminal verification
results. EDN read-back replay reproduces the helper and compact management source;
a second management generation is byte-identical. Production manage.hal and its
existing tests remain unchanged pending presentation/print, parallel/random and
result-wrapper contract audits, plus the final test merge. The broader migration
goal remains incomplete; this record supersedes the earlier awaiting-decision
status rather than claiming a completed management port.

### Management parallel dispatch audit

Pinned process/bulk/result/manage source confirms that none of the seven original
management main functions returns Foundation's Result wrapper. That generic wrapper
is not a missing dependency for these exports and must not be ported by default.

The original :parallel branch does require overlapping item execution through pmap.
A bounded regression has item one await a gate released by item two, while item
three fails; it also asserts input/result order and bulk error isolation. The
installed serial adapter fails on item one's timeout, as expected. Promise/run and
Promise/delay 0 candidates fail identically. Native LocalPromiseProvider::run calls
task() immediately, and Promise::schedule also executes zero delay immediately.

Positive delay exposes a separate suspended-value boundary: the migration call
chain attempts collection lookup on a Promise. Minimal delayed-all tests reproduce
pending Promise observations without the migration adapter, through both deref and
IDeref/deref. Bounded-wait probes obtain native Array values (distinct from Vector),
but bounded waits throughout the actual adapter still fail the overlap regression.
Using native Test/check instead of fact does not fix the call-chain error. These
probes distinguish assertion shape, suspension handling and real overlap; none is
evidence of working parallel dispatch. Do not install an arbitrary timeout or
silently call serial/cooperative execution equivalent to Foundation pmap.

lang_core_manage_parallel_audit.edn preserves complete candidates, bounded and
native-check probes, assertions, native source locations and terminal outputs.
No failing parallel candidate was written to Hara. The installed adapter remains
at its passing six-fact version; production manage remains unchanged. Next work is
native dispatch/progress and suspended-call tracing, plus presentation/random
contract audits. This is a dependency finding, not completion or a new Task-type
decision.

### Host synchronous callback boundary fixed

The suspended-value finding is now fixed in hara-native's
core/rust/src/vm/machine/coroutine_runtime.rs. The ordinary compiled-function native
callback previously turned a suspended frame into a Promise return, allowing
callers to consume the wrong value and bypass delayed exception/cleanup behavior.
It now resumes suspended frames at the non-Wasm synchronous boundary. Explicit
Promise returns, async functions, and the separate fiber callback retain their
existing behavior. This does not change Promise/run scheduling or claim pmap parity.

Before the fix, two of three new native-lang regressions fail: numeric consumption
of a Promise and an uncaught delayed error with cleanup still at zero. After the
fix, native-lang 48, native-command 2 and native-test-registry 6 all pass. The first
sandboxed broad attempt has four loopback socket permission failures; the permitted
rerun passes all 56. Release hara-native was rebuilt successfully before Hara checks.

The two minimal delayed-all/Work checks now pass with vec normalisation of native
Array results. The written management adapter still passes six facts, and all 17
historical management facts pass against it with verified fixture cleanup. The
parallel candidate now reaches the expected result comparison instead of failing
on Promise collection lookup, but still fails its overlap assertion. Do not install
that candidate or claim parallel completion. lang_core_manage_suspension_fixed.edn
records the clean native base revision, exact patch, source after-image, regression
and build results, post-build Hara evidence and remaining work. Native runtime
documentation is in docs/synchronous-callback-suspension.md. No commits/publication.

### Cooperative management dispatch written

Two scheduler defects are now fixed. A global progress guard suppressed every peer
during nested waits; it now guards individual Promise identities through weak
references and an unwind-safe stack. Promise/all also polled ready callbacks before
registering all peers; it now registers the batch before polling. The independent
native regressions fail before their respective changes and pass afterward, along
with guard restoration after callback panic. Native-lang 51, native-command 2 and
native-test-registry 6 pass. Release hara-native was rebuilt successfully.

The internal manage-task source/test pair now includes deferred bulk dispatch via
Promise/delay 1 and Promise/all. It preserves input/result order and error isolation;
single invocation does not consume the parallel option. All seven written adapter
facts pass, including the previously failing overlap assertion. All 17 historical
management facts pass with fixture cleanup. Two real-library facts pass with six
parallel invocations, and all four calls in the three bulk-contract facts pass with
parallel enabled. This proves cooperative overlap, not CPU-thread pmap throughput.
Promise/run and zero-delay behavior were not changed, and no Task engine was added.

Scaffold CLI preview/write both exit 134 with a stack overflow on the existing
namespace. Native code.manage.unit inventory of the actual written pair reports no
missing, TODO, unchecked or new scaffold obligations. Candidate, written-file and
post-build checks are preserved in lang_core_manage_parallel_written.edn, together
with the parent record/checksum for before-images, structural replay adapters,
native base revision/patch and documentation. EDN read-back regeneration matches
the written source exactly. Production manage source/test remain unchanged pending
presentation/random audits and the complete test merge; broader migration is not
complete.

### Management random ordering written

The internal management adapter now implements the original :random bulk-input
option. Its Fisher-Yates permutation uses existing Crypto/random-bytes, rejects
biased four-byte draws, preserves values and the caller's collection, and
materialises single-pass inputs once. It does not add a public Foundation shuffle
API. Single invocation and :random false retain original behavior.

The routing regression fails against the prior adapter because no shuffle occurs;
it passes after the change. A separate regression found and fixed double iterator
consumption. Three deliberately wrong shuffle expectations fail, then restoration
passes. The written source/test pair passes 11 facts; native code.manage.unit finds
no missing/TODO/unchecked/scaffold obligations. All 17 historical management facts
still pass with fixture cleanup. The deterministic routing test restores its
temporary owning-Var replacement in finally and checks the metadata boundary.

CLI scaffold preview/write still fail with stack overflow (134). Exact native
inventory and written-file tests are retained in lang_core_manage_random_written.edn,
alongside the parent checksum/before-image references, structural adapter, authored
tests, failed regressions and exact replay. Source regeneration matches the written
file. Presentation/default print behavior and production manage replacement remain
unfinished; this is not a completed management or language migration.

### Management presentation candidate: historical tests green, format-string gap open

The pinned std.print.format.report source/tests now have a staged structural
adaptation to an unpublished lang.core.manage-display boundary, using existing
native ANSI/string operations rather than replacing the native report-document
renderer or creating another task engine. All eight historical facts pass,
including exact colored row and title strings. A deliberately wrong title gives
seven passing facts and one failure; restoration returns eight passing facts.
The copied test rule emits top-level Test/run, rejected by the current runner;
an explicit adapter preserves assertions as registered native facts.

Additional assertions verify function formatters, empty strings and CRLF/trailing
newline behavior. The string-format branch remains genuinely broken: "%02d"
resolves the absent std.foundation/format instead of producing "03". Historical
tests do not exercise that branch. Do not claim complete formatter parity or
install the candidate until this and stronger report:column/Result boundary tests
are resolved. Native Result display status is staged in its display context,
separate from transport success/error; no std.lib.result type is introduced.

lang_core_manage_display_stage.edn retains immutable source/test authority,
copied-rule plans, structural adapters, before-failure/negative/restored outputs,
the extra failing assertion and byte-exact replay. EDN read-back reproduces both
candidate source and tests and proves the source adaptation idempotent. No
presentation source/test files were installed, no scaffold/write validation is
claimed, and production manage remains unchanged. Wiring original print options
into the Work adapter and the full migration objective remain unfinished.

### Management cumulative item time written

The Work-backed management adapter now restores bulk-summary's cumulative timing
field after finalisation, summing every item record's time. Ignored and errored
items contribute, and empty input returns zero. The regression fails against the
prior adapter because a finaliser's -999 timing value survives; it passes after
the original timing override is restored. The assertion compares the returned
total with the exact recorded durations of successful, ignored and failed items.

The complete candidate and written pair each pass 12 facts. The historical
management fixture passes all 17 facts and reports cleanup verified. Written
code.manage.unit inventory reports no incomplete/unchecked/new obligations;
scaffold preview/write still crash with stack overflow (134). The durable record
lang_core_manage_cumulative_written.edn preserves the parent checksum and
before-images, source/test outputs, regression results and structural replay.
Replay matches written source/tests exactly and is idempotent (an initial
duplicate-field rewrite was guarded before saving the migration rule).

Presentation and production manage cutover remain pending. The original elapsed
measurement ends after item processing, before presentation and finalisation;
the adapter's elapsed timing boundary still needs correction and validation.
No full management or language migration completion is claimed.

### Management elapsed end boundary written

Elapsed time is now captured immediately after item processing, before result
aggregation, projection and summary finalisation. The returned timing overrides
any finaliser-provided elapsed field. The regression runs serial and parallel
batches with a 100ms finaliser and requires elapsed to fit within the real interval
before finalisation begins. Both modes fail against the prior boundary; all 13
candidate and written adapter facts pass after the change. All 17 historical
management facts still pass with verified fixture cleanup.

The native clock is intrinsic: replacing its Var did not alter calls in an
isolated, restored probe, so no clock-mocking claim is made. Native written-file
inventory reports no incomplete, unchecked or new scaffold obligations. Scaffold
preview/write still abort with stack overflow (134).

lang_core_manage_elapsed_written.edn retains the parent checksum/before-images,
source/test outputs, failed and passing checks and byte-exact idempotent replay.
The elapsed start remains after shuffle, unlike original bulk-items; that start
boundary still needs correction. Presentation and production management cutover
also remain pending. The full approved migration remains incomplete.

### Management shuffle timing written

Selection remains outside the timed interval, while shuffle now occurs after the
start timestamp and before item processing. Empty batches skip shuffle, as the
original bulk-items does. The delayed-shuffle regression fails against the old
boundary and passes with the new one: elapsed includes shuffle, cumulative time
does not, and returned item ordering is unchanged. A separate empty/listing
regression proves shuffle is not invoked there. Tests restore the temporary
shuffle Var replacement, metadata and local mutable state in finally.

The candidate and written source/test pair pass 15 facts; all 17 historical
management facts pass with cleanup verified. Written code.manage.unit inventory
has no incomplete, unchecked or new scaffold obligations. Scaffold preview/write
still abort with stack overflow (134). The record
lang_core_manage_shuffle_timing_written.edn preserves exact parent before-images
and checksum, failing/passing checks and byte-exact idempotent replay. Management
presentation and production cutover remain unfinished, as does the full migration.

### Management duration formatting written

The original format-ms dependency is now in the internal management adapter.
Five literal format expressions are structurally specialized to native padding
and string construction; existing std.lib.time and the public formatting surface
are unchanged. Positive-divisor remainder normalization preserves the original
negative-input behavior. Sixteen boundary inputs match direct evaluation of the
pinned Foundation function, including exact padding and minute/day precision loss.
Raw millisecond timing fields remain available; labels are presentation only.

The candidate and written pair pass 16 facts. An incorrect duration expectation
produces 15 passes and one failure, then restoration returns 16 passes. All 17
historical management facts still pass with cleanup verified. Native written-file
inventory has no incomplete, unchecked or new obligations. Scaffold preview/write
still abort with stack overflow (134). lang_core_manage_duration_written.edn retains
authority, parent before-images/checksum, adapters and byte-exact idempotent replay.

A source audit confirms the seven original management task definitions use
function-based result-data formatters; neither their columns nor inherited template
columns use printf format strings. The generic report candidate's printf branch
remains incomplete, but is not needed by those fixed definitions. General std.print
parity is not claimed. Connecting the complete original print flow and production
management cutover remain outstanding.

### Original management title printing connected

The Work-backed adapter now prints the original ANSI-bordered title before timing
bulk processing when any recognized print flag is enabled. Function titles receive
merged parameters and the constructed environment before item execution. Single,
listing and silent requests do not print a title, and domain results are preserved.
Literal-format specialization also preserves the original nil-to-"null" conversion;
the first concatenation candidate dropped nil and was corrected after a failing
regression.

The complete written pair passes 19 facts. The old adapter fails the title callback
and captured-output assertion. Removing ANSI styling produces 16 passes and three
failures; restoration returns 19 passes. All 17 historical management facts pass
with fixture cleanup, now visibly printing the seven original task titles. Written
native inventory reports no incomplete, unchecked or new scaffold obligations;
scaffold preview/write still abort with stack overflow (134).

lang_core_manage_title_written.edn retains authority forms, the parent checksum and
before-images, structural source/test adapters and all failure/recovery evidence.
Source and tests regenerate byte-exactly and their adaptations are idempotent.
Original item rows, result tables, warnings/errors and summaries remain unfinished;
production management cutover and the full migration are not complete.

### Management projection ordering staged; native laziness decision pending

Three regressions expose additional differences in the written adapter: unrequested
projections run, projections precede finalisation, and deferred summary notices are
missing. The staged adapter removes native cumulative timing from the pre-aggregate
base, finalises before packaging, projects only requested fields and prints original
:std.task/after-summary notices before projection even with summary printing off.

The candidate passes 21 facts with one strict laziness failure. Native Seq freezes
and memoizes subsequent reads, but Iter/seq calls peek_first while constructing the
sequence, so the first projection runs before the caller consumes a vector-package
result. The original defers every projection. operation.rs::iterator_seq and the
native Iter declaration confirm this boundary; no deferred constructor was found.

The candidate and failures are retained in lang_core_manage_projection_stage.edn
with parent checksum/before-images and exact idempotent replay. Nothing from this
candidate was installed, no assertion was weakened and native semantics were not
changed. User decisions were requested for native deferred-sequence support and
printed-summary map layout. Existing title-enabled source/tests remain the written
state; the full migration remains incomplete.

### Pinned lazy-package oracle verified

Direct evaluation of the immutable bulk-package function confirms that no output
projection runs when its vector-package result is returned, consumption executes
the projections, and repeated reads do not execute them again. A deliberately
throwing projection is raised during consumption, not invocation. The original
empty result remains truthy and consumes to an empty vector.

The native candidate passes the empty-result check but fails the exception-boundary
check: first-element lookahead raises the projection error during invocation.
Current iterator_seq still calls peek_first. This is a confirmed semantic gap,
not merely a stricter test assumption. lang_core_manage_lazy_oracle.edn preserves
the pinned function, replayable oracle, native probes and results. Native sequence
support still awaits the user's decision; neither API nor written management
source/tests were changed by this investigation.

### Approved deferred sequence support installed

The user approved the native extension. Iter/seq-deferred now constructs the
existing memoizing Seq without first-element lookahead; Iter/seq is unchanged.
The management structural adapter wraps only its item/result output projections
with Iter/seq-deferred and Iter/iter-map. It retains work.flow.task and the
original finalisation, requested-field and deferred-notice ordering.

Written manage_task source/tests pass 24 facts in a fresh native process.
Native inventory reports no incomplete, unchecked or new scaffold obligations.
Both scaffold CLI commands still abort with the existing stack overflow (134).
Two newly moved probes initially failed because their compile calls were not
qualified in the written test namespace; qualification fixed that test-harness
error and all 24 written facts then passed. No assertions were weakened.

The native language/command/test-registry suites pass 53 + 2 + 6 tests. Four
socket tests initially failed under sandbox restrictions and pass with local
loopback access permitted. Both new deferred-sequence regressions failed before
the constructor was implemented and passed afterward.

lang_core_manage_deferred_written.edn preserves the parent checksum, structural
adapters, boundary assertions and deterministic replay with written-file hashes.
The earlier pending-decision entries above are historical. Summary map rendering
remains undecided; item/result/warning/error/summary presentation, production
management cutover and the full migration remain incomplete. JVM/browser parity
for the new constructor has not been verified.

Final validation also passed all 35 native CLI binary tests and cargo check
--offline --bin hara-native. The staged original management fixture reran
against the installed adapter: 17 facts passed, all seven task titles were
printed and :manage-work-cleanup-verified confirmed restoration of fixture
state. The saved deferred migration record was read back and its parent hash,
source/test replay and written hashes verified.

### Original bulk error diagnostics restored

The pinned bulk-process-item prints `>> <message>` whenever item execution
throws, independently of all print flags. Direct evaluation of that immutable
function produced exactly `>> broken item\n`; the previous native adapter
returned the error record silently. An owner-scoped, idempotent structural
rewrite now restores the diagnostic inside the bulk catch branch. Single-item
failures still propagate without a bulk diagnostic.

The pre-change suite failed the new exact-output regression (25 passed, 1
failed). The candidate and fresh written suite pass all 26 facts, including
sequential/parallel execution with presentation disabled and unchanged domain
results/error records. A deliberately incorrect single-error expectation also
failed, then the correct written suite passed again. Native inventory reports
no missing, unchecked or new scaffold obligations; both CLI scaffold attempts
still exit 134 with the known stack overflow.

lang_core_manage_error_output_written.edn retains the pinned function, parent
checksum, structural adapter, semantic tests, negative evidence and replay.
Read-back replay reproduces both written files exactly. Original parallel
per-item output buffering/input-order flushing remains to be implemented;
direct callback printing is not yet equivalent to bulk-items-parallel. Item
rows, result/error/warning tables, summary presentation and production cutover
remain unfinished. No native runtime changes were made in this slice.

### Original parallel output grouping and order restored

The previous adapter emitted `one-start\ntwo\none-end\n` when the first
item suspended and a peer printed. Direct evaluation of pinned
bulk-items-parallel confirms `one-start\none-end\ntwo\n`: it captures each
item's output and flushes blocks in input order. The new regression failed
against the written adapter (26 passed, 1 failed).

The structural adapter now captures each run-item with with-out-string and a
local result atom reset in finally. Promise/all subscribes all peers, preserving
the established overlap contract; individual pending items are then awaited
and their captured text flushed in input order with native p. It does not wait
for the aggregate result before beginning ordered flushing. No new task engine
or runtime primitive was added. This retains cooperative Promise concurrency,
not JVM pmap's CPU-thread throughput.

The complete candidate and fresh written source/test pair pass 27 facts;
overlap, error isolation, elapsed boundaries and lazy projections remain green.
Native inventory reports no missing/unchecked/new scaffold obligations. Both
CLI scaffold commands still abort with the known stack overflow (134).
lang_core_manage_parallel_capture_written.edn retains the pinned function,
oracle output, before failure, structural adapter and parent checksum. Its
read-back replay reproduces the written source and test byte-for-byte.

Item rows, result/warning/error tables, summary presentation and production
management cutover are still pending. The full migration is not complete.

The historical management fixture also passes all 17 facts against the written
adapter, prints all seven original task titles and confirms fixture restoration
with :manage-work-cleanup-verified.

### Original presentation column preparation installed

prepare-columns is selected structurally from the copied-rule plan of the
pinned std.task.bulk source and appended to lang.core.manage-task without
changing its body. The historical literal width assertion is preserved with
only its test owner changed. Additional tests cover explicit zero/fixed widths,
inferred widths, missing fields, empty columns and preservation of column
options. A deliberately incorrect padding constant makes both tests fail
(27 passed, 2 failed); the correct candidate and fresh written files pass all
29 facts. Native inventory is clean; scaffold CLI preview/write still abort
with the known stack overflow (134).

lang_core_manage_columns_written.edn records the planned form, original test,
parent checksum, added semantic tests and byte-exact source/test replay. This
is a required renderer dependency, not connected result-table output yet.
The seven original task configurations use function-based data formatting and
no printf column formats. Permission was requested to specialize their internal
renderer to that contract; no unsupported generic formatting branch has been
installed. Summary map layout and the broader presentation/cutover remain open.

### Item-duration formatting audit: native rounding is not the original

The original bulk-process-item expression `(format "%.2fs" (/ time 1000.0))`
does not match native str/to-fixed for ordinary durations: 15ms is `0.02s`
in the original versus `0.01s` natively; 995ms is `1.00s` versus `0.99s`.
An owner-scoped literal adaptation using integer millisecond rounding passes
all 19 ordinary oracle cases, including negative zero and second carries.

The full signed-integer boundary audit prevents a broader parity claim:
integer-only formatting retains precision that the original floating-point
conversion discards. A candidate native fixed-point fallback still differs
from the original at 10 of 466 tested boundary inputs. The mismatches and
executable probes are retained in lang_core_manage_item_duration_audit.edn;
read-back verifies both oracle vectors and the parent checksum.

No duration candidate or incomplete renderer was installed. Written source and
tests are verified unchanged from the 29-fact column-preparation slice. This
is evidence for the pending item-row adaptation, not completed presentation.
