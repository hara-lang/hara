# Scaffold workflow audit — 2026-09-15

The full language migration remains active. This is a shared workflow
prerequisite, not completion of the port or the scaffold gate.

## Written selector fix

`code.manage.cli/units-for` discarded the parsed namespace selection and asked
`code.manage.context/units-for` to read every declared source/test pair before
the task filtered records. It now forwards the existing parsed selector to the
existing three-argument discovery API. No public API was added.

The path-matched test checks exact file selections for empty, single, missing,
and multiple namespace inputs. It fails on the old implementation (one failed,
zero errors) and passes on the complete native candidate. The full CLI suite
passes 33 tests in the candidate and twice after writing. Diff whitespace
validation passes. Candidate construction used native Hara string operations;
no new Clojure or other code-processing implementation was introduced.

Using current CLI source in a fresh native process, a scaffold request for the
real project's `code.manage.cli` now produces one record with exit 0 and no
plan errors. Its changed count is 1, not 0, for the correspondence issue below.

## Independent issues at the selector-fix checkpoint

- The installed `/opt/homebrew/bin/hara` resolves through Homebrew and reports
  `hara 0.1.0`. Its manage command rejects `--format`, although current source
  supports it. Its source runner rejects `:config :role`. Scaffold preview and
  write through that launcher still stack-overflow. It has not been upgraded,
  and no claim is made that the installed launcher gate passes.
- Current correspondence/scaffold analysis does not recognize the CLI test
  file's `Test/register` blocks. It reports 33 missing facts even though all
  33 corresponding tests execute. A native before/after probe reproduces the
  same 33 missing reports on both versions. Do not apply the proposed duplicate
  scaffolds; fix or explicitly migrate the existing test representation first.
- `lang.core.type-notify` is absent from `project.edn`'s explicit source-file
  list. A zero-record scaffold for it would not prove coverage. Reconcile the
  migration's manifest inventory without disturbing unrelated project edits.

No commits, publication, global launcher replacement, or duplicate scaffolds
were performed. The CLI source/test change has passing behavioral tests, but
its broader source-run and correspondence gates remain unresolved.

## Native registration correspondence fixed — 2026-09-15

`code.manage.unit.structure/fact-record` now recognizes literal `Test/register`
descriptors without changing the legacy fact-head registry. It preserves source
spans, refer/added metadata, and comment state, and reads desc/name titles.
An explicit non-nil test and an expected key establish a checked registration;
nil and false expected values count. Missing descriptors, tests, or expected
keys remain unchecked. This is an inventory check, not proof that an assertion
is semantically sufficient. Dynamic descriptors remain unchecked.

Two new behavioral tests fail against the original implementation (two errors
because no records are returned), pass against the complete candidate, and pass
after writing. The complete structure suite passes 53 tests in the candidate
and twice from disk. Fresh native downstream suites pass: scaffold 17,
test-doc 48, CLI 33, and unit 41 (192 total including structure).

A fresh native read-only audit of the actual structure and CLI source/test
pairs returns empty incomplete, unchecked, and missing-scaffold inventories
for both. This resolves the earlier 33 false missing reports without writing
duplicate tests. The audit calls scaffold/missing-definitions with its four
arguments; an initial one-argument diagnostic was invalid and was corrected.
Candidate source construction used native Hara; no migration authority or
Foundation source was modified. The changed pair passes git diff --check;
unrelated existing language-file whitespace findings remain untouched.

Installed launcher gates were retried: scaffold preview and --write both exit
134 with stack overflow, and source run exits 1 on unsupported :config :role.
These gates remain unresolved; passing current-native tests do not certify the
installed launcher. No global launcher replacement, commits, or publication.
The type-notify manifest inventory issue also remains open.

## Notification manifest discovery fixed — 2026-09-15

Added only src/lang/core/type_notify.hal to project.edn's explicit source-file
list, adjacent to the runtime entry. Existing native-version and removed-grammar
manifest edits remain intact. A native candidate check verifies equality of
all other project options and exact preservation of the previous file list
after excluding the one new entry.

The exact notification-selection probe fails against the prior manifest:
[0 nil nil :missing :missing]. After writing it returns one source unit,
src/lang/core/type_notify.hal, its paired test/lang/core/type_notify_test.hal,
and empty incomplete/unchecked findings. The notification suite passes all
44 tests in a fresh native process, including local TCP lifecycle tests.

Current native CLI source now produces a real scaffold plan for that namespace:
exit 0, summary count 1, changed 0, errors 0, and byte-equal before/after test
source. An initial probe incorrectly expected a :total summary field; the
verified probe uses the actual :count field and compares before/after. No
duplicate test write was needed. The installed Homebrew launcher issue is
unchanged and is not certified by this result. The changed manifest passes
git diff --check. No commits or publication.
