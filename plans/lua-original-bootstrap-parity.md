# Original Lua bootstrap parity — staged 2026-09-15

Authority: Foundation `fe54cc866473a888bfbd4ce2d3de6d8011f09f72`.
Replay record: Foundation
`resources/code/migrate/candidates/lang_runtime_basic_lua_bootstrap_parity_stage.edn`.

This candidate reuses the saved original XTalk `spec-base`/`common-lib` fixture
chain and the installed original Lua/XTalk models. It generates the return
helpers with `impl/emit-entry-deps`, and both input bodies and calls with
`impl/emit-as`. It invokes real Lua through the installed oneshot execution
helper. Six exact expected payloads cover number, nil, false, Unicode, array,
and object. No emitted helper implementation is handwritten.

The record read back equal and repeated replay produced identical candidate
text (42667 characters; the first transport adds a terminal newline). These
checks do not establish process parity. Native emission/execution was still
running when this staging note was written; no passing result is claimed.

Environment checks:

- `lua -v`: Lua 5.4.8.
- Lua's installed `cjson` encodes `{value=42}` as `{"value":42}`.
- Plain `luajit` cannot require `cjson`; `/Users/zcaudate/.luarocks` is absent.
- The immutable original process lists `:lua` as a supported program, but its
  oneshot/basic defaults are `:luajit`. A Lua 5.4 pass will not certify the
  default LuaJIT environment. No packages or lookup paths were changed.
- Foundation process-lua source and tests remain unchanged against the pin.

Installed Lua model SHA-256:
`c44248be6d62200670e1ad0b32d0572181bd10f3e5413d38eda62a0287c0ad9c`.

Remaining after a successful candidate: negative control, emitted-byte replay,
full process-lua source/test migration, installation of the original XTalk
fixtures, bench/container prerequisites, and real basic socket lifecycle.
This does not replace those obligations or the original LuaJIT default.

## Long-running native check

The first native run has no phase markers before the complete emission/execution
fact returns. Its source inventory is 428 forms: four namespaces, two script
declarations, 207 `defspec.xt`, 175 `defmacro.xt`, 39 `defn.xt`, and one fact.
At 11 minutes 12 seconds the same native process had accumulated 11 minutes
11.70 seconds CPU time and was using 100% of one core. It was not restarted.

One-second native stack samples show repeated `call_function`/`call_value` and
direct native context/closure frames; leaf samples also include formatting and
allocation. These observations do not isolate the Hara owner or prove a loop,
deadlock, successful emission, or assertion failure. If another full run is
needed, add diagnostic phase markers around fixture namespace loading and
dependency/body/call emission to distinguish their costs. Do not skip original
dependencies or alter the emitted implementation merely to shorten the check.

## Direct encoder isolation

While the full run remained live, a separate fresh native diagnostic required
the installed Lua model and directly emitted the original canonical
`return-encode` body using `impl/emit-as`. It did not register the 428-form
XTalk fixture. The transform produced a 1343-character printed form and the
emitter produced 1947 bytes. Real Lua with cjson returned the exact selected
payload `["data" 42]`; one fact passed with no errors. This returned within the
first follow-up poll rather than taking minutes.

That narrows the long-run investigation beyond this direct encoder path; it
does not distinguish fixture loading, registration, dependency collection, or
the other emitted helpers. It is not a substitute for `emit-entry-deps` parity.
The diagnostic's temporary stdin symlink was removed. The runner displayed the
other concurrent stdin-backed test path in its summary; the explicit phase
output and numeric assertion identify this as the direct diagnostic, not the
still-running full bootstrap check. No full-bootstrap pass is claimed.

## Dependency-collection boundary located

A phase-marked diagnostic preserved all 428 forms and reached the final fact
after loading the fixture, without the multi-minute stall. That diagnostic was
then deliberately stopped (exit 143) once it had answered the loading question;
it is not a failing assertion or a completed parity check. The original full
run was left untouched.

A second diagnostic decomposes the same original `emit-entry-deps` path into
`emit-options`, `collect-script-entries`, and per-entry `emit-entry` calls. Its
last returned markers are `:emit-options-start` and `:collect-start`; it has not
returned the entry IDs. Thus the observed long-running boundary is dependency
collection, before per-entry emission or Lua process execution. Fixture loading
is no longer the leading explanation. Inspect `Book/get-code-deps` through
`emit-template/cached-entry-deps` and per-language restaging before changing
the dependency traversal or host semantics.

## Entry restaging boundary located

The reversible wrapper diagnostic logs original function entry/exit and forwards
arguments unchanged. `return-eval` and `return-wrap` both complete preprocessing,
rewriting, XTalk scanning, and cached dependency lookup. `return-encode` completes
`to-staging`, then enters `rewrite-stage` without returning during observation.
This locates the long interval more narrowly than collection as a whole.

The installed `canonical-stage` and pinned original have the same control flow:
resolve the XTalk function definition, infer an environment when dot access is
present, then lower the form. No evidence yet distinguishes those substeps for
this entry. Do not bypass canonicalization or substitute predeclared dependencies
to make bootstrap tests pass. The wrapper functions are process-local, restore
their original values/metadata in `finally`, and are not production source edits.

## Full run returned; separate inference defect fixed

The original full run completed with one assertion failure and zero errors.
The fact took 2,134,652 ms (about 35.6 minutes). Number, nil, false, Unicode, and
object payloads matched. The array failed because its dynamically loaded body
accesses global `cjson`, but the diagnostic harness declared it local. The pinned
`process_lua.clj` explicitly uses `cjson = require("cjson")` in its original
oneshot wrapper and bootstrap. Restore that original global setup in the next
candidate; do not change the emitted return helpers or weaken the array assertion.

A bounded inference diagnostic also established that definition lookup returns
and the long interval is environment inference for `return-encode`; it reached
1002 inference calls before the deliberately injected diagnostic limit escaped.
That limit is not installed and its error is not a production failure result.

An independently reproduced fidelity bug was corrected: migration had rewritten
the literal `==` case arm in `infer-type` into a second `=` arm. Native inference
therefore returned `:xt/unknown` for `(== 1 2)` instead of the original boolean.
The corrected scoped structural rule protects this DSL symbol with an ordinary
symbol constructor and preserves the two original arm bodies independently.
It does not introduce a native equality API or bypass canonicalization.

The generated inference source and paired regression are installed. Candidate
and two written-file runs each passed four facts. The pre-change native probe
failed its equality assertion; the migration-rule negative control detects the
old rule. Language-catalog generation now reproduces both installed files
byte-for-byte, including the new test, with repeated generation equal.
Recovery records are `lang_typed_infer_equality_recovery.edn` and
`lang_typed_infer_equality_catalog_written.edn` in Foundation candidates.

The Clojure rule-test append landed, but its post-write lint gate reported 101
unresolved symbolic fact IDs throughout that existing file; full namespace reload
also exceeded the 10-second evaluation limit and was interrupted. Those gates
remain incomplete. Source-rule reload and the focused regression body were
verified separately before that full reload attempt. No full-suite green claim
is made. The equality fix has not yet been proven to resolve the long Lua run.

The original run is terminal. All remaining diagnostic processes were explicitly
stopped after answering their isolation questions, and their stdin symlinks were
removed. No test process remains live from these probes.

## Corrected global harness and focused rule verification

The successor candidate is
`lang_runtime_basic_lua_global_bootstrap_stage.edn`. It retains the former
test program, restores the exact original global cjson setup, keeps all six
assertions unchanged, and adds bootstrap/process phase markers. A new native
run against the installed equality correction has reached `:bootstrap-start`;
its result is still pending. This is a new corrected candidate after the prior
run's terminal assertion failure, not a restart of an unobserved process.

The written Clojure regression was re-read and executed through its registered
`code.test` Fact in its owning namespace. Positive, old-rule negative, and
restored results were exactly `[true false true]`. Initial direct invocation
from `user` returned false because this eval-based test requires its namespace
binding. `lang_typed_infer_equality_focused_fact.edn` records the corrected
post-write execution evidence. Broad lint and full namespace reload gates are
still not certified.

The follow-up case-key audit covers all four targets registered under
`:foundation/lang-typed-inference` (form, call, lower, infer). It found no other
case-key corruption: grouped keys in `access-kind` expand to individual keys;
the protected `==` branch is the intentional difference in `infer-type`.
`lang_typed_inference_case_key_audit.edn` retains original and generated dispatch
inventories and explicitly limits the claim to that target group.

## Equality inference correspondence revalidation — 2026-09-15

A fresh native process selected `lang.typed.xtalk-infer` through
`code.manage.cli/units-for` with the explicit absolute Hara project root.
The exact result `[unit-count incomplete-findings unchecked-findings]` was
`[1 [[]] [[]]]`; the audit assertion passed (one pass, zero failures/errors).
This closes the missing/TODO/unchecked inventory check for the installed
inference source/test pair, not the broader Clojure lint or pipeline gates.

The initial diagnostic omitted `code.test` and failed to resolve `fact`.
The next used relative project discovery and returned zero units; its assertion
correctly failed. Neither was treated as coverage. Explicit project discovery
resolved the actual manifest-listed source and passed the nonzero-unit check.
The temporary audit symlink was removed after the process exited. The separate
corrected Lua bootstrap session 62407 remains live with no new output at this
checkpoint; no restart or completion claim was made.

## Surrounding typed regression run — 2026-09-15

Release `hara-native test --project .` with explicit `--file` selections under
`test/lang/typed/` completed successfully: xtalk-analysis 13, xtalk-call 13,
xtalk-env 8, xtalk-form 31, xtalk-infer 4, xtalk-lower 18, xtalk-ops 19,
and xtalk-parse 41. Total: eight files, 147 passed, zero failed/errors/timeouts.
This checks the surrounding installed inference pipeline after the equality
correction; it does not certify the pending full Lua bootstrap or all typed
namespaces.

Read-only inspection of pinned `tahto/typed/xtalk_infer.clj` and
`tahto/typed/xtalk_lower.clj` confirms that the original infer-type also calls
lower-form and retries inference on changed forms, and original lower-dot
consults receiver inference. Those recursive relationships alone are not
evidence of a migration defect or authorization to bypass inference.

## Clojure reference inference measurement — 2026-09-15

The existing Foundation nREPL evaluated only the pinned
`lua-tf-x-return-encode` function in temporary namespace
`migration.lua-reference-probe`, with the original `std.lib.template` alias.
Input was `(x:return-encode out id key)`; the resulting body printed to 1356
characters. The loaded reference infer-type was called with namespace
`xt.lang.common-lib`, empty aliases/environment, and preserve-unknown true.
`with-redefs-fn` counted recursive calls and restored the original Var on exit;
the temporary namespace was removed in `finally`. No source files were written.

Result: 16,127.388709 ms, 80,027 wrapped recursive inference calls (excluding
the initial direct call), result keys `(:type :errors :env)`, one diagnostic
error. This is an isolated timing observation, not a successful semantic test:
it neither reproduces the full Book entry context nor inspects that diagnostic.
The substantial repeated inference also occurs on the Clojure reference side;
the earlier native 1000-call diagnostic threshold was not evidence of an
infinite loop. Full native Lua bootstrap remains pending. Do not compare these
timings as equivalent-work performance measurements or bypass inference based
on this probe.

## Saved Lua replay and reference diagnostic read-back — 2026-09-15

Read the corrected global candidate back from disk and invoked its saved replay
twice. Both results were byte-equal to each other and to the retained
`lua-global-program` supplied to native validation: 42,816 characters.
This verifies candidate reconstruction, not installation or process success.

Repeated the original encoder inference without the call-count wrapper:
15,734.32225 ms. The sole diagnostic is `:unsupported-binding-target` for
`(quote [r-ok r-err])`, the original pcall result binding. Result type is the
union of xt/unknown and xt/nil. The probe returns a result rather than throwing.
The native and original infer-environment implementations both consume the
returned environment without treating diagnostic entries as fatal. This
reference diagnostic therefore does not justify rewriting the binding or
weakening a migrated assertion. The full Book-context result remains unproven.

Fresh downstream tests also passed: emit-rewrite 9 and emit-template 18
(27 total, zero failures/errors). Native process inspection at this checkpoint
showed the remaining hara-native PID 20297 at 22:59 elapsed and 99.0% CPU;
session 62407 is still live. This is evidence of ongoing computation, not proof
of eventual completion or of where that time is spent.

## Corrected Lua bootstrap terminal result — 2026-09-15

Session 62407 has now terminated successfully, exit 0. It printed
`:bootstrap-done`, `:process-start`, the complete emitted return_encode /
return_wrap / return_eval bootstrap, and six emitted calls. The candidate's
one fact passed with no failures/errors/timeouts. Its six decoded payload
assertions remained unchanged: 42, nil, false, "hello λ", [1 2 3], and {"a" 1},
each with type "data". In particular the array case that failed under local
cjson now passes under the original global cjson setup.

Bootstrap was generated through emit-entry-deps and bodies/calls through
emit-as, then executed by actual Lua via oneshot/sh-exec. This proves the
corrected saved candidate on installed Lua 5.4.8 with cjson. It does not certify
the default LuaJIT setup, socket runtime, installed common-lib fixture cutover,
full process-lua migration, repeated end-to-end execution, or bootstrap byte
parity with Clojure. No inference bypass was introduced. The stdin symlink
was removed only after terminal success; no Lua validation session remains
live from this run.
