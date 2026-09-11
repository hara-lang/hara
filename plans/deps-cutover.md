# Dependency imports and collector: installed native cutover

Authority: Foundation `fe54cc866473a888bfbd4ce2d3de6d8011f09f72`.
Delivery state: source/test pairs installed locally; model and caller integration remains incomplete.

## Installed files

- `src/lang/core/impl_deps_imports.hal` and its path-matched test.
- `src/lang/core/impl_deps.hal` and its path-matched test.
- Original `test/lang/base/emit_prep_js_test.hal` data fixture.

The original dependency/import algorithms replace the previous implementations.
Reviewed native test additions remain in the generated tests. Displaced files
are recoverable from Foundation's
`resources/code/migrate/recovery/lang_core_deps_native_cutover.edn`.

## Validation

- Fresh native candidate runs: imports 19 passed; collector 53 passed.
- Fresh runs after writing: imports 19 passed; collector 53 passed.
- Post-write runs loaded installed imports, collector, Book, emit, entry,
  Snapshot and Library sources, without overlays for those implementations.
- Original XTalk/Lua/JS model construction candidates and explicit fixture
  loading remain necessary. This is not a standalone project-wide pass.
- Scaffold preview/apply, incomplete and unchecked audits: two passing checks.
- Wrong-expectation controls: 19 imports failures and 53 collector failures,
  both with zero runtime errors.
- Foundation assertion-accounting validation after writing: 26 helper checks,
  five pinned-pair checks and one workflow check passed.
- Changed source/test pairs pass `git diff --check`.

## Historical assertion accounting

| Pair | Historical total | Retained | Deferred | Native additions |
| --- | ---: | ---: | ---: | ---: |
| imports | 18 | 9 | 9 | 12 |
| collector | 23 | 22 | 1 | 39 |

Assertions are distinct from the native runner's fact counts above.
The migration now exposes separate accounting and retained deferral evidence.
Historical totals were not reduced; native additions cannot compensate for
unrecorded historical loss. Both inventories explicitly report incomplete.

## Remaining work

- Restore the deferred JS module and Python polyfill integration assertions.
- Restore the collector's historical XTalk and native script-setup integration.
- Cut over old callers of the former three-arity module-import interface.
- Install the original impl/script/model pipeline; do not restore grammar-api
  or obsolete Library adapters.
- Resolve native test-fixture discovery so standalone project tests can load
  test namespaces through their declared test roots.
