# Original core registry: installed local cutover

Authority: Foundation `fe54cc866473a888bfbd4ce2d3de6d8011f09f72`.
State: generated source and path-matched tests installed; pipeline parity is incomplete.

## Installed behavior

The original two atoms, 117 runtime coordinates, 34 Book descriptors and four
lookup functions replace the version-table, grammar-selection, Book-image and
static macro-index implementation. Runtime values are namespace symbols; Book
descriptors are unversioned maps. `registry-book` requires the namespace and
dereferences the registered Var. Missing coordinates and missing Vars return nil.
The original atom snapshot/reset operations provide state restoration; the
native tests restore their baseline in `finally`.

Source and test symbol routes come from the migration catalog before the
`tahto` to `lang` fallback. Only registered routes are remapped: JavaScript
currently points at `lang.model.v1.spec-js`; unregistered models still require
catalog registration and regeneration. No compatibility wrappers were added.

## Verification

- Candidate registry: 7 passed, zero failures/errors.
- Written registry: 7 passed, zero failures/errors; fresh native process loads
  the actual source through the test namespace, with no registry source overlay.
- Written impl regression: 42 passed; written lifecycle regression: 16 passed.
- Scaffold preview/write-option and incomplete/unchecked audit: 1 passed.
- Deliberately wrong JavaScript routing: 5 passed, 2 failed, zero errors.
- Written catalog regeneration: byte-exact source, tests and fixture.
- `git diff --check` is clean for the registry source/test pair.

Historical accounting: 14 assertions = 11 retained + 3 deferred. Three native
assertions supplement the retained assertions; seven facts execute in total.
The three original JS/Lua Redis/Lua Nginx model-loading assertions are deferred,
not replaced by or counted as the synthetic loader contract.

The fixture is installed at `test/migration/registry_value_fixture.hal` and
explicitly loaded for focused validation. Standalone test-fixture discovery is
still unresolved. Regression checks use staged original model construction and
explicit Lua fixtures; they do not prove a standalone project-suite pass.

## Downstream migration obligations

- `src/lang/core/runtime.hal`: lines 188, 205, 211, 214, 217 and 219 call retired
  runtime lookup/loading wrappers; line 287 uses retired Book version selection.
- `src/lang/core/script.hal`: lines 421, 429 and 434 use the retired static macro
  index. Original script/model installation must replace that path.
- `src/lang/seedgen/common_xtalk.hal`: lines 173 and 191 use the removed third
  `registry-book` argument.
- `src/lang/seedgen/common_util.hal`: lines 109 and 121 likewise use the removed
  third argument. These callers are catalogued, not silently modified here.

The focused passes above do not cover these incompatible consumers. Runtime
and script must follow their original owners; do not restore the rejected
version/grammar-api architecture to keep these old callers alive.

## Recovery and test dispositions

Full displaced source/tests and their individual test blocks are preserved in
Foundation `resources/code/migrate/recovery/lang_core_registry_native_cutover.edn`.
Version-selection, grammar-coordinate, image-cache and static macro-index tests
are retired with that architecture. State restoration and unversioned lookup
contracts are adapted to the original atoms/functions. Runtime loading and
actual model materialization remain pending integration, not equivalent coverage.

Machine-readable native results are in Foundation
`resources/code/migrate/candidates/lang_core_registry_native_cutover.edn`.
No commits or publication were performed.
