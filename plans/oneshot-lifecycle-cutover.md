# Oneshot lifecycle cutover

`RuntimeOneshot` now implements identity `IComponent/start`, `stop`, and
`kill`. Foundation revision `fe54cc866473a888bfbd4ce2d3de6d8011f09f72`
provided this behavior through the Object fallback in `std.lib.component`.
Only this runtime type is extended; native component defaults are unchanged.

Source and path-matched regression test are written locally. The regression
checks receiver identity, retained fields, and repeated stop/kill. It fails
against the prior source and passes with the candidate. The complete written
source/test pair loads and passes 32 facts in a fresh native process
(`oneshot-lifecycle-written-job`, migration REPL 50370). Scoped diff check is
clean. Native correspondence reports no new, incomplete, or unchecked tests.
Installed scaffold preview and write both crash with stack overflow, so the
underlying native inventory API supplied correspondence evidence. Standalone
native `run` cannot discover project dependencies; fresh native-test loading
of the complete written pair provides the source execution evidence.

Foundation candidate records retain the original source, structural recipe,
indexed before-images, and negative test evidence:

- `lang_runtime_basic_oneshot_lifecycle_pending.edn`
- `lang_runtime_basic_oneshot_lifecycle_recipe.edn`

Those records describe candidate-time status; this note records installation.
Structural replay matches the installed source exactly and is idempotent.
`lang_runtime_basic_js_historical_pipeline_pass.edn` separately records the
passing historical JS pipeline with the equivalent appended lifecycle candidate
and remaining source/model overlays; it is not an installed-pipeline claim.

No commits or publication performed.
