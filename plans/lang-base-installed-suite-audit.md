# Installed lang.base suite audit — 2026-09-15

Full migration remains incomplete. This audit includes legacy native additions
as well as migrated original owners; passing files do not prove source parity.

## Execution

Selected all 37 files returned by `rg --files test/lang/base | sort`, passing
each as an explicit `--file` to
`../hara-native/core/rust/target/release/hara-native test --project .`.
The runner completed with exit 1: **934 passed, 5 failed, 5 errors**, no
timeouts/skips/cancellations. No files or expectations were changed for this run.

## Red results and next disposition

| File | Result | Observed issue |
| --- | --- | --- |
| compiler_test | 1 error | Legacy test resolves removed spec-xtalk/+selection+ and constructs catalog grammar. Do not restore the rejected alias. |
| emit_compat_test | 2 errors | Missing emit/emit-infix and obsolete arities; requires retained native-test reconciliation against original owners. |
| emit_data_test | 40 pass, 2 fail | Exact map output expects a,b but receives b,a; both original and retained native facts affected. |
| emit_orchestration_test | 5 pass, 1 fail | Symbolic Book/grammar resolution succeeds; default namespace differs from the fixture's expectation of lang.base.emit. |
| emit_prep_js_test | 1 error | Fixture-only file contains no runnable tests. |
| emit_prep_lua_test | 1 error | Fixture-only file contains no runnable tests. |
| emit_special_test | 18 pass, 1 fail | Module-table output expects sub_fn,add_fn but receives add_fn,sub_fn. |
| registry_test | 8 pass, 1 fail | Legacy catalog finalization expects lang/type marker; original Grammar contains highlight/emit/banned fields instead. |

Fresh repeat of only emit-data and emit-special reproduced exactly the same
three failing facts: 58 passed, 3 failed, zero errors. These are not explained
by state contamination from preceding suite files. The pinned original
emit-coll sorts only the special :# and :.. positions, preserving equal-ranked
map iteration order. Native source has the same sort. Blanket lexical sorting
would be a semantic change and would not preserve both recorded expectations.

Pinned prep-options uses env/ns-sym as its default namespace; the native owner
uses ns-current. The orchestration assertion needs an explicit behavioral
comparison before changing either implementation or expectation.

## Passing preprocess evidence

All six preprocess owners passed: assign 24, base 14, input 10, resolve 15,
staging 8, value 20 (91 total). All four Book suites passed: entry 14, meta 13,
module 22, Book 66 (115 total). These counts certify execution of the installed
tests, not completion of historical inventory, regeneration, or full pipelines.

The legacy grammar-api suite also passed its five checks. That is evidence
that the unwanted architecture remains present, not evidence that its removal
is complete. No compatibility API was reintroduced to make this audit green.

The separate corrected Lua bootstrap session 62407 was repeatedly confirmed
live during this audit and was neither stopped nor restarted.

## Caller-namespace test correction

The orchestration failure is resolved without changing emitter source.
Foundation prep-options under explicit user and tahto.base.emit namespace
bindings returned those respective symbols. A fresh native probe returned its
caller namespace migration.emit-namespace-probe, equal to ns-current. Thus the
old native expectation of a fixed lang.base.emit namespace contradicted the
original contract.

Changed only that preserved native fact's title, expected namespace, and
refer/id metadata. The old incorrect expectation failed in the full run
(five pass, one fail); the complete candidate passed all six facts. Written
tests passed twice in fresh processes; the original path-matched emit suite
also passed 21 tests. No historical Foundation assertion or implementation was
changed. The temporary diagnostic symlink was removed after terminal exit.

Foundation candidate record
`lang_base_emit_orchestration_caller_namespace.edn` retains complete before and
after test images and the replay adaptation. Disk read-back and two replays
were byte-equal to the installed test. Changed-file whitespace checks passed.
The other failures above remain unresolved; the 37-file suite has not been
rerun since this correction, so no revised aggregate count is claimed.

## Retained emitter compatibility assertions reconciled

The two emit-compat facts now call the original emit-common/emit-infix owner
and the four-argument emit/emit API. Every expected output string is unchanged.
Both facts now carry refer/id metadata. The std.block-backed migration engine
parsed and rewrote forms structurally, retaining unchanged source forms.
No source implementation or compatibility alias was added.

Complete candidate: two passed. A deliberately incorrect infix expectation
produced one passed, one failed, zero errors. Written tests passed twice;
the focused run with emit-common passed 138 tests (136 common, two retained
compatibility facts). Saved before/after images and structural replay live in
Foundation's `lang_base_emit_compat_original_api.edn`. Disk replay, repeated
generation, and applying the adaptation again all equal the written bytes.
The temporary candidate symlink was removed after terminal execution.

## Map-order cause and ordered-fixture candidate

Fresh native probe confirmed that {:a 1 :b 2} iterates b,a and a constant-key
sort preserves that order. The same Clojure literal is PersistentArrayMap,
iterates a,b, and its constant-key sort also preserves order. Native Map is a
CHAMP implementation with hash-placement iteration. The mismatch is not an
unstable sort.

Existing Algo/ordered-map preserves a,b, remains map?, and equals the ordinary
map by contents. Both historical emit-data map/vector output expectations pass
with that constructor. A std.block-backed structural candidate replaces only
literal map inputs at emit-data/data/emit-data call sites; its three affected
inputs retain their original entry sequence. All expected strings and other
tests are retained. Complete candidate passed 42 tests, zero failures/errors.

The candidate and full before-image are saved in Foundation as
lang_base_emit_data_ordered_fixture_stage.edn. Its structural adaptation is
idempotent. It is NOT installed: catalog-rule/native-addition integration,
negative control, and written-file validation remain. Do not claim the two
installed emit-data failures resolved yet. No Map or emitter semantics changed.
The independent emit-special ordering failure uses entry metadata sorting and
must be diagnosed separately, not fixed by applying this fixture adaptation.

## Ordered-fixture rule integration checkpoint

Foundation rewrite-emit-data-test-form now structurally converts literal map
arguments at emit-data/data/emit-data call sites to Algo/ordered-map, preserving
the Clojure reader's entry sequence. The guarded source write succeeded.
Explicit require :reload and post-write checks passed: exact replacement,
idempotence, quoted-code preservation, and failure of the old rule to produce
the expected ordered constructor.

The corresponding existing fact extension was evaluated successfully in the
REPL, including an old-rule negative control, but its file edit was rejected
by the Clojure pre-write lint gate: 101 existing unresolved symbolic fact IDs
and two unused-require warnings. No gate bypass was attempted. Approval was
requested to repair the fact-ID lint handling without changing the test IDs.
Permanent regression, catalog native-addition integration, generated-file
installation, and written native validation are still pending.

An earlier combined evaluation client stalled before sending any definitions
to the REPL. The healthy JVM answered separate health/checkpoint probes.
Only verified client PID 35366 was terminated (session 6730, exit 143); the
server and unrelated processes were untouched. Smaller evaluations succeeded.

## Ordered emit-data fixtures installed through the catalog

Updated only the retained native test-source field of the emit-data catalog
target, then generated the complete test from the pinned original source and
the written structural rule. Emitter source regenerated byte-exactly without
any source change. The generated candidate passed 42 tests. Reversing all three
ordered-map fixtures produced 40 passed and two failed facts, zero errors.

The generated native test is now installed and passed all 42 tests twice in
fresh processes. Reloaded disk catalog regeneration matches both installed
source and test byte-for-byte; a second generation produces identical tests.
An initial verification command had an out-of-scope local variable and was
discarded; the corrected check returned [true true true] for source, test,
and repeated-generation equality.

Recovery record lang_base_emit_data_ordered_fixture_written.edn retains old
and new catalog lines and migration-rule forms, with the previous candidate
record retaining the complete native test before-image. The temporary native
candidate symlink was removed after terminal execution. The permanent Clojure
regression extension is still blocked by the fact-ID lint gate; this native
success does not certify that gate or the entire migration.

## Module-table ordering diagnosis and candidate

Instrumented the actual emission call inside the existing fixture lifecycle.
Both entries have priority 5, time nil, and line nil. Native code-map traversal
and the stable metadata sort produce add-fn,sub-fn; the original small-map
insertion order is sub-fn,add-fn. The separate priority/time ordering regression
still produces first,late,last. No missing timestamp or line was invented.

A fixture-only candidate initializes that newly installed module's code map
with existing Algo/ordered-map before the two original entry additions, using
existing library get-module/add-module! operations. It retains existing entries
when converting, all metadata, all output assertions, and existing cleanup.
The complete candidate passed 19 tests (baseline 18 passed, one failed).
Its structural replay is idempotent and full before/after images are saved in
lang_base_emit_special_ordered_fixture_stage.edn. Rule integration, negative
control, installation, written validation, and catalog regeneration remain.

The first diagnostic looked outside the fact lifecycle and saw an empty
library; that was not used as ordering evidence. A later candidate client
stalled before reaching the JVM; verified PID 42000 alone was stopped, exit
143. Smaller evaluations succeeded without restarting the REPL. The temporary
native probe symlink was removed after terminal candidate success.

## Module-order fixture installed through the existing rule

Integrated the ordered code-map initialization into the existing
rewrite-emit-special-tests-form structural composition. Its candidate is
idempotent on the pinned original fixture; the old rule lacks that step.
Guarded source write and explicit namespace reload succeeded. Pinned catalog
generation has no diagnostics and reproduces unchanged emitter source exactly.

The complete generated test passed 19 facts. Deliberately replacing the ordered
map with an ordinary map reproduces the module output failure: 18 passed,
one failed, zero errors. Installed the generated test; all 19 facts passed
twice in fresh processes. Disk catalog regeneration equals both installed
files byte-for-byte and repeated generation is identical.

Recovery record lang_base_emit_special_ordered_fixture_written.edn retains
rule before/after forms and references the full native before-image in the
staged record. No emitter source, entry metadata, expected output, Book default,
or catalog target was changed. Permanent Clojure regression-file work remains
behind the unresolved fact-ID lint gate. The temporary candidate symlink was
removed after terminal validation; no validation process remains live here.

## Executable fixture contracts installed

The native runner selects every .hal file under configured test paths, including
both emit-prep fixture files. Added three authored Test/check cases to each
existing fixture namespace: unchanged empty Book/module state, exact fragment
expansion, and assembled Book fragment/code contents with identity form and
dependencies. Lua additionally checks subtraction's standalone form. No original
fixture definition, namespace, or path was changed, and no empty test was added.

Both complete candidates passed three tests each. Deliberately incorrect
fragment-expansion expectations failed one case in each file (two passed,
one failed, zero errors). Installed through native additions on the existing
catalog targets. Written files passed twice, three tests each. A downstream
emit-special run passed 22 checks: its 19 existing facts plus the three imported
Lua fixture cases. Do not misreport those imported cases as new emitter facts.

Disk catalog regeneration equals each installed file byte-for-byte and repeated
generation is identical. The JS file retained an extra trailing blank line;
catalog output now preserves it. Two attempted whitespace-only native edits
were rejected for patch-context mismatch and made no change. Recovery record
lang_base_prep_fixture_contracts_written.edn retains original fixture bytes and
before/after targets. The temporary candidate symlink was removed after terminal
execution. Legacy compiler/registry failures and the Clojure lint gate remain;
the full installed suite has not been rerun since these corrections.
