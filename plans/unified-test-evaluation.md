# Shared unit and interactive evaluation

Status: implemented and locally verified; full color acceptance passes in all three modes.
No commit, release, or push was performed for this implementation.

## Ordered macro loading follow-up

Implementation validated in native unit, CLI cold/warm, and configured Emacs cold/warm modes.

- Required native source namespaces now compile and execute top-level forms in
  order, under a scoped evaluation namespace. Runtime-installed macros therefore
  receive their original unevaluated arguments, including XTalk `var` forms.
- Whole-namespace source caches are bypassed, not deleted. Explicit bytecode
  artifact loading is unchanged. An integration test seeds a legacy cache entry
  and proves two fresh runtimes use source while leaving the cache untouched.
- Native regressions cover metadata preservation, replacement of macro roots,
  exactly-once registration, forward calls, scoped nested requires, and stopping
  after a failed source form. All 60 native-lang tests pass with loopback access.
- JS/Lua/Python adapters now consume `install-type!` and tests inspect the
  context/resource registries and stored program options, not change reports.
- RuntimeBasic retains emission/module/library options, implements pointer
  invocation, and avoids decoding already-semantic return values twice.
- Lua basic evaluation restores the reference adapter's source-level body
  transform so scripts with dependencies return their final value.
- Fact metadata `:setup` and `:teardown` now become executable lifecycle hooks;
  setup precedes assertions and teardown also runs after setup failure.
- The shared executor scopes hooks and assertions to the owning fact namespace
  and restores the caller, including when invoked from an interpreter session.
- Recursive collection checkers preserve existing checker values, including
  `approx`; regression tests cover rejected values and idempotent wrapping.
- Interactive setup exposed a separate native printing crash: runtime state
  contains cycles. Value printing now detects active-path identities, preserving
  shared siblings and removing the guard on normal return and unwinding.
- Current focused evidence: 113 shared-runner checks, 16 collection-checker
  checks, 2 native cyclic-printing checks, 16 RuntimeBasic checks,
  8 JS adapter checks, 11 Lua adapter checks, 8 Python adapter checks, and 99
  Emacs tests. Negative controls fail for disabled setup, double decoding,
  incorrect resource coordinates, and incorrect Lua body output.

The completed color unit run passes all 12 facts. The recursive approximate
checker is covered by focused regressions that fail against the previous code.
The formerly empty hue fact now asserts seven boundary cases in each of JS,
Lua, and Python. Interactive CLI and Emacs both reached setup but crashed in
native result printing (confirmed by the broker stack-overflow crash report).
The rebuilt printer lets Emacs complete both passes: 19 forms, 12 facts, and
36 assertions per pass. The shared verdict is passed and the owned editor
server was cleaned up. CLI cold and warm also pass all 19 forms, 12 facts,
and 36 assertions per pass. Its final report has status passed, 38 records,
and no native-only skips. All identified owned acceptance processes exited.
The existing test expectations have not been weakened.

Performance is not yet optimized: actual C-e evaluation took about 828 seconds
cold and 609 seconds warm under concurrent validation load. The slowest form
took 128 seconds cold and 84 seconds warm, below the unchanged 180-second
deadline. The workspace host and installed project distribution host have the
same SHA-256: `7c42ce70b5eba53b7ada145daa1c98846edd782fb42bd57069c2ae7e5cb8b160`.

The hue negative control rejects `[999 120 240 300 60 180 42]` in all three
targets while the written expectation passes. The four newly indexed
RuntimeBasic port/lifecycle facts also reject incorrect expectations.
`code.manage` scaffold preview/write, incomplete, and unchecked report no
remaining findings for RuntimeBasic. Native reader-based function/reference
inventories report no missing function or macro references for the changed
compiler, executor, checker, RuntimeBasic, and Lua adapter source namespaces.

## Contract

- Native `test --project ROOT --file FILE` defaults to `--mode both`.
- `unit` runs the native suite through `code.test.execution/run-suite`.
- `interactive` uses native reader spans and exact source, once per form, in an
  owned cold RESP server followed by warm replay in that same session.
- Primitive-only Test/check/Test/register files are explicitly native-only.
  Mixed files retain primitive assertion evidence as well as fact evidence.
- Protocol 4 advertises EVAL-REPORT 1. EVAL with REPORT 1 sends TEST_REPORT,
  the existing RESULT/ERROR, and DONE. Display strings are not pass criteria.
- Source owns assertion execution, validation, counts, and cold/warm comparison.
  Empty facts fail; setup-only files cannot pass as tests. Partial reports survive
  later errors. Metadata retains its printed source separately from JSON evidence.
- Each selected interactive file gets a fresh process. Owned transports are
  closed on every exit. Read/parse failures and assertion failures do not prevent
  later selected files from running.
- The configured Etude C-e walkthrough uses the same native source plan and
  validator. It never discovers or attaches to the user's live editor server.
  Explicit namespace evaluation is sent once, not repeated for synchronization.

## Entry points

```text
make test-interactive FILE=test-lang/xt/lang/common_color_test.hal
hara-native test --project ROOT --mode both|unit|interactive --file FILE
hara-native test --project ROOT --form-timeout-ms 180000 --file FILE
M-x hara-test-interactive
make -C extensions/hara-emacs test-etude-walkthrough
```

The CLI writes `.hara/test/interactive.edn` after each completed response.
The Emacs walkthrough writes its configured JSON report after each response.
Both retain exact source, assertion evidence, and partial failure information.

## Validation observed

- Eight code.test implementation suites: 113 native checks passed.
- Foundation suite: 219 native checks passed.
- Color suite: 12 unit facts passed; CLI and configured C-e each pass cold and
  warm with 19 forms, 12 facts, and 36 assertions per pass. Both saved reports
  were read back and their final passed verdicts and counts verified.
- XTalk call suite: 14 unit facts passed; all 16 top-level forms passed cold and
  warm through both the CLI and actual configured Etude C-e.
- Emacs installation checks and 99 ERT tests passed.
- Native registry: nine tests passed; RESP: five tests passed; report broker:
  two tests passed; CLI argument parsing passed.
- A mixed-file failing Test/check is rejected at the same form by the CLI and
  actual C-e, even though EVAL successfully returns a displayed Result value.
- Negative controls demonstrated failure for duplicate namespace evaluation,
  missing primitive evidence, incorrect metadata tags, and nonlinear threading.
- The workspace distribution was rebuilt with `make hara-native-update`, and
  the Emacs package was installed locally. Version remains 0.1.33.

Interactive validation artifacts:

- `target/emacs-shared-types.json`: passing cold/warm real C-e suite.
- `target/emacs-shared-smoke.json`: intentional mixed-file failure, form 5.
- `target/emacs-color-shared-final.json`: historical color failure, form 2.
- `target/emacs-color-ordered-macros.json`: passing real C-e cold/warm color
  workflow, 38 exact evaluations and a passed shared assertion verdict.
- `.hara/test/interactive.edn`: passing CLI cold/warm color workflow, 38 exact
  evaluations, no native-only skips, and a passed shared assertion verdict.

## Defects uncovered and fixed

1. Namespace synchronization evaluated the selected namespace form twice.
2. Primitive checks in mixed files were absent from the fact-only journal.
3. cond->/cond->> duplicated the remaining expansion in both branches. Six
   clauses produced 32 copies of the final step. The shared helper now emits a
   linear single-evaluation expansion, with ordering and multiplicity tests.
4. Untyped grammar fallback inferred arguments unnecessarily before delegating.
5. Raw metadata sets could not cross the strict JSON boundary; printed metadata
   provenance now round-trips data metadata without serializing live callbacks.

## Earlier acceptance blockers (resolved by the follow-up)

The adapter migration initially failed at `runtime/install-type`; the source
now uses `install-type!` and inspects its actual registration contract.
The native compiler initially rejected XTalk `(var ...)` as native syntax
before the runtime-installed macro could quote it. Ordered source compilation
fixes that boundary without adding XTalk special cases to native `var`.

The empty hue fact, missing lifecycle hooks, namespace ownership, Lua return,
double decoding, nested approximate checker, and cyclic result-printing
failures were exposed by progressively running the actual color workflow.
The broader unit and editor evidence, not isolated regression counts, remains
the final acceptance criterion. Whole-source cache redesign is deferred.
