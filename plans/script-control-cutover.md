# Original script-control: installed local cutover

Authority: Foundation `fe54cc866473a888bfbd4ce2d3de6d8011f09f72`.
All six original functions and the watch-key vector are now generated into
`src/lang/core/script_control.hal`, with path-matched tests.

The implementation uses the installed core registry and native Space/context
APIs. It retains preparation, no-start scratch construction, stop/restart
arities and one-shot invocation. The reviewed one-shot adaptation uses nested
`finally` cleanup for newly owned contexts; existing contexts remain owned by
their caller.

Fresh native validation:

- Candidate and written source: 14 passed, zero failures/errors.
- Wrong historical one-shot output: 13 passed, 1 failed, zero errors.
- Scaffold preview/write-option and written correspondence: no new, changed,
  incomplete or unchecked obligations.
- Main catalog source/test regeneration is byte-exact; recovery read-back is
  exact.

The six historical facts account for 12 generated historical assertions.
Explicit adaptations accept native RuntimeDefault records in two map checks,
strengthen the stopped-runtime check to exact nil, and activate the original
one-shot example with unchanged `return 1` output. Eight additional facts cover
the watch vector, ownership, failures, configuration changes and lifecycle state.

Validation supplies staged original Pointer, runtime-proxy, RuntimeDefault,
script and model definitions. It uses the latest Result/function? Pointer
candidate. Registry, Book, Library, Snapshot, impl and script-control load from
the installed tree, without source overlays. This does not establish complete
runtime/proxy installation, standalone initialization or runtime.basic parity.

Machine-readable results and the exact replay prefix are in Foundation
`resources/code/migrate/candidates/lang_core_script_control_installed.edn`.
Authority units, structural before-images and guarded recovery instructions
are in `resources/code/migrate/recovery/lang_core_script_control_installed.edn`.
Both native paths were previously absent. No commits or publication were made.
