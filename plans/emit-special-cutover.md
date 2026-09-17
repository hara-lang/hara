# Original special emitter: installed local cutover

Authority: Foundation `fe54cc866473a888bfbd4ce2d3de6d8011f09f72`.
The generated source and path-matched tests are installed. Original model
installation and standalone fixture loading remain pending.

The emitter now follows the original Book lookup and preprocessing pipeline,
including callable module filters, merged Book entries, host evaluation and
Var dereference. The existing native `allowed-op?` helper is retained unchanged.
The original non-Var error code/message replaces the previous invented error.

Verification in fresh native processes:

- Complete candidate: 19 passing facts (8 historical, 11 retained native).
- Written emitter loaded from disk: 19 passing facts, no failures or errors.
- Pre-change emitter against the same suite: 15 passed, 1 failed, 3 errors.
- Adjacent written `lang.base.emit` suite: 21 passed, no failures or errors.
- Native scaffold preview/write-option and correspondence checks pass before
  and after installation: no missing, changed, incomplete or unchecked tests.
- Main migration catalog regenerates source and tests byte-for-byte.

All 13 historical assertions are preserved. The native script-wrapper fixture
was replaced by an isolated original Library; output, metadata and baseline
assertions remain. Libraries and their settings are reset during teardown.

These runs supply the staged original XTalk/Lua model definitions and explicitly
load `test/lang/base/emit_prep_lua_test.hal`. They do not overlay emitter, Book,
Library, Snapshot or impl source: those load from the installed tree. This is
not a standalone model-loading or whole-language-suite completion claim.

Reproduction and full native results are in Foundation
`resources/code/migrate/candidates/lang_base_emit_special_installed.edn`.
Displaced source/test before-images, hashes and guarded recovery instructions
are in `resources/code/migrate/recovery/lang_base_emit_special_installed.edn`.
The catalog owns both files. No commits or publication were performed.
