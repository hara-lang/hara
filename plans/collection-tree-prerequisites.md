# Collection tree prerequisites — 2026-09-15

## Tree-flatten installed

The pinned `tree-flatten` is appended to the existing collection implementation;
all prior source and tests are preserved byte-for-byte. Its std.block recipe,
authority blobs, structural before-images, exact replay, and checksums are in
Foundation's `collection_tree_flatten_installed.edn` candidate record.

Five focused contracts pass against the written source: the original historical
assertion, empty/scalar leaves, explicit separator/prefix, dynamic binding, and
idempotent canonicalization retaining the caller's original tree. All five
deliberately incorrect expectations failed without errors. Written source loads.

Native `keep` preserves map type, so the recipe explicitly traverses `(seq m)`.
Native map conjunction accepts individual entries, so recursive result maps are
merged explicitly. Neither adaptation changes the native collection APIs.

The complete collection suite cannot load: `IConj/conj map expects a two-element
entry`. The unchanged before-images reproduce the same error. This is a baseline
failure, not a passing suite; the five new focused checks pass independently.

The user approved retaining the cache. `re-create`, `re-create-reset`,
`tree-nestify`, and `tree-nestify:all` are now installed. The cache retains native
regex values; native regex reference identity is not exposed. Tests prove cache
hits do not replace the cache map, compilation failures preserve cache state,
and repeated resets return the empty baseline. Ten focused blocks pass, including
all four historical nestify assertions. Eight authored negative controls fail.
Bootstrap correspondence reports no missing or pending tests. The entire
collection file still has the independently reproduced pre-existing load error.

`std.lib.schema.impl` is installed: seven original facts plus seven contract facts
pass; all fourteen negative controls fail. Its original flat/tree/lu/vec record
shape, native display, reverse references and vector provenance are retained.
`std.lib.schema` is also installed for original PostgreSQL hydration: eight
original facts (including the saved sample fixture) plus eight contract facts
pass; all sixteen negative controls fail. Source inventory includes both files.
Source and test replay have been checked byte-for-byte against the written files.
Records: `collection_tree_nestify_installed.edn`,
`relational_schema_impl_installed.edn`, `relational_schema_public_installed.edn`.

The schema family now passes 80 tests: base 19, find 7, ref 24, impl 14, public 16.
These are schema tests, not proof of a working PostgreSQL client or live samples.
CLI scaffold preview/write reported no changes or findings. The namespace prefix
matches five schema namespaces; the corrected unchecked audit passes with count
five and zero findings/errors. The schema.impl recipe uses metadata-preserving
EDN printing so its open record survives serialization and exact replay.

## Application constructor and executable schema prerequisites

The original `postgres.application/app-create-raw` is installed through a
std.block-owned rewrite of the pinned Foundation constructor. It now constructs
the original schema, native pointers, an open Application record implementing
IDeps/IDisplay, and dependency-ordered lookup indices. Other application helpers
are retained for the next dependency-ordered stage; this is not a claim that the
entire application namespace is faithful yet.

The application suite passes 38 checks. Ten deliberately incorrect expectations
fail without errors. Coverage has no missing or pending public seams; existing
internal helper references are retained. The record's serialized adapter
regenerates the written source and tests byte-for-byte:
`postgres_application_raw_installed.edn`.

Constructor adaptations use native pointer/dependency APIs, `(seq tables)` for
mapcat, an explicit empty map for missing links, and `(iterate inc 0)` for lookup
indices. Native zero-argument range currently returns an empty sequence despite
its documentation. Pointer context is read through IPointer/ptr-context, not
ordinary field lookup; pointer comparison follows native structural equality.
The original app-get-entry string-key lookup is retained explicitly, including
its nil result against canonical keyword-keyed schema trees.

Direct project loading exposed a real Foundation defonce bug: Runtime/eval
detached the initializer from its caller namespace and lexical bindings. The
macro now emits the conditional def directly. The new regression fails against
the before-image on the captured local `offset`, then passes with the fix. The
written Foundation file loads, its 213 tests pass, and the written application
loads directly in a fresh project process. Record:
`foundation_defonce_caller_context_installed.edn`.

Cold interactive schema evaluation then exposed an empty-key arity mismatch in
expand-scopes: `(apply dissoc m [])` is invalid for the native Foundation Var.
The structural adapter now reduces dissoc over the keys, preserving the original
empty case. The existing custom-scope regression failed before this change;
all 19 schema.base facts now pass in both cold and warm interactive runs.
Scaffold preview/write reported zero changes and findings. Record:
`schema_scope_empty_keys_installed.edn`. Task-specific interactive evidence is
`.hara/test/postgres-prerequisite-interactive.edn` (the shared default report is
also used by concurrent work).

Follow-up: the user requested zero-key dissoc support. Foundation now accepts
`[value & keys]`, so schema.base has been restored to the original `apply dissoc`
call. The temporary reduction adapter is historical, not the current target.
See `foundation-dissoc-zero-keys.md` for the updated arity contract and evidence.

Focused unit coverage totals 365 checks: Foundation 213, application 38, schema
80, and existing table/hydration 34. This does not include live PostgreSQL or
claim that every other suite passes interactively. The installed Homebrew hara
launcher is stale (it rejects :config :role); validation used the current native
release binary. Bootstrap inventory also reports a pre-existing pending long?
test outside this change.

## PostgreSQL integration remaining

The last revalidation after installing schema.impl found client 32 passing / 8
errors, all at obsolete Library/Script calls. scratch-v0 cannot require its
unregistered sample source. `db.postgres.connection` still calls unavailable
`host/call`; no actual database connection is validated. The PostgreSQL root,
postgres-grammar and postgres-native still retain the rejected grammar-api path.

The original table-hydration authority requires std.lib.schema and the original
PostgreSQL application registry. Continue with that dependency before replacing
the root Book/grammar composition, then the original client and native transport,
then sample registration and live integration. Do not resurrect removed core
aliases merely to satisfy the replacement client tests. No existing application
database has been used or altered for these validations.

At the start of this phase, native collection lacked the original tree helpers.
Those helpers and the schema family are now installed, as recorded above.

The user approved retaining the original dynamically bindable separator.
std.foundation/*sep* is installed with default "/", public/schema metadata,
and a path-matched test for nested bindings and restoration after an exception.
The complete source candidate and written source were evaluated using native
`run`; the project test wrapper rejects the complete bootstrap's top-level
declare forms, so it was not used as the bootstrap source loader.

Bootstrap scaffolding inventoried the new seam. A wrong expected result failed,
the correct candidate passed, and the written Foundation suite passed all 212
tests. foundation_separator_installed.edn records exact append-only replay,
before/after checksums, and recovery instructions preserving previous edits.

Next: finish original application Book discovery and typed analysis, including
original duplicate-column dependency handling, then replace the custom table
hydration facade with the original hydration pipeline. Follow with the original
PostgreSQL model, native client transport, and disposable live sample integration.

Docker Desktop was started for the approved isolated PostgreSQL fixture.
Existing application containers restarted under their own policies and were
not modified. No migration test database or container has been provisioned.
The pinned Foundation database image is not present locally. Do not use the
existing application databases as a substitute for the approved disposable one.
