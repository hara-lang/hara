# PostgreSQL schema prerequisite revalidation — 2026-09-15

## Update: keyword blocker resolved

The user approved restoration of Foundation multipart keyword behavior during
PostgreSQL integration planning. Native reader and keyword validation now
accept multipart names, and interning/equality retain explicit namespace/name
partitions rather than conflating identical printed strings. First-slash
parsing and constructor semantics were checked against the Foundation REPL.

The rebuilt release launcher ran the exact recorded source/test candidate:
**24 passed, zero failures, zero errors**. No schema assertions or source forms
were changed to obtain this result. Native regression validation passed all
60 tests across native-keyword and native-lang with loopback socket permission.

The schema-reference pair is now installed in `src/std/lib/schema/ref.hal` and
its path-matched test. Both written files match the validated candidate bytes.
The source inventory now includes schema base/find/ref. Scaffold preview and
write discover one owner with zero changed files and no missing tests; the
inventory reports no missing, incomplete or unchecked tests. All 13 authored
contracts fail under deliberate wrong expectations (11 historical facts still
pass), and the restored written suite passes all 24 facts. The three installed
schema suites pass 50 facts in fresh native processes.

`relational_schema_ref_installed.edn` retains checksums and executable replay
through both prior records, checked against the immutable Foundation source and
test blobs. Existing stage records are retained for recovery.

Next: catalog registration, collection tree operations and schema.impl,
then the original PostgreSQL hydration/application/grammar/compiler pipeline.
The earlier findings below describe the pre-fix baseline, not a current keyword
blocker. Real PostgreSQL provider/client/sample validation remains outstanding.

## Historical pre-fix evidence

The full language migration remains active. No PostgreSQL completion claim.

The complete source/test candidate from Foundation's
`resources/code/migrate/candidates/relational_schema_ref_contract_audit.edn`
was retrieved through the existing nREPL and evaluated unchanged in a fresh
native 0.1.33 process. Result: 21 passed, zero assertion failures, three errors.
All errors are `Base/keyword failed: Keyword name can only contain one slash.`
The retained failing IDs are keyword-round-trip, reverse-boundaries, and
root-and-missing. No assertions were dropped and the candidate was not installed.

## Stronger dependency evidence

The immutable authority at fe54cc866473a888bfbd4ce2d3de6d8011f09f72 contains:

- `std.lib.schema.impl/create-flat-schema` calls collection/tree-flatten.
- `simplify` and `schema-map` call collection/tree-nestify.
- The original collection/tree-flatten fact constructs a three-level map and
  expects keys rendered as `a/b/c`, `a/b/d`, `a/e/f`, and `a/e/g`.
- The original implementation constructs those keys with keyword, not strings.

Thus multi-slash keywords are required by an original historical dependency
test, not only by the three added schema-reference edge tests. The previous
literal search limited to schema/PostgreSQL files did not establish otherwise.
Neither tree-flatten nor tree-nestify is currently defined in native
std.lib.collection. Its existing unrelated functions must be preserved.

Native rejection owners are hara-native/core/rust/src/lang/data/keyword.rs
and core/rust/src/kernel/parser.rs. The parser regression suite explicitly
expects :a/b/c to be rejected in kernel/parser_tests.rs. Changing only the
schema adapter or the constructor would not reconcile reader behavior.

An asynchronous user decision was requested before expanding native keyword
semantics. Required validation for an approved change includes construction,
reading, namespace/name behavior, printing/reading round trips, and the retained
schema and historical collection tests. Do not substitute string keys, encode
slashes, or weaken assertions to bypass this decision.

Dependency order remains collection operations and schema base/find/ref,
then schema.impl and facade, then original PostgreSQL hydration/application,
root grammar, and compile-schema. Base/find are already installed; this note
does not certify the remaining owners. No Foundation or native source was
changed during this revalidation. The temporary stdin test symlink was removed.
