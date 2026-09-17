# Pointer main callback cutover

The local `lang.core.pointer/ptr-invoke` now dereferences native Var-valued
`:main :in` and `:main :out` callbacks before invoking them. Function-valued
callbacks and absent callbacks retain their previous behavior. This preserves
the original Foundation process configuration, including Var-backed JS
`default-oneshot-wrap`, rather than rewriting registrations to lose Vars.

Authority: Foundation revision `fe54cc866473a888bfbd4ce2d3de6d8011f09f72`,
`src/tahto/core/pointer.clj`, `ptr-invoke`.

The structural recipe, exact before-image, and regression evidence are in
Foundation `resources/code/migrate/candidates/lang_core_pointer_main_var_callbacks_pending.edn`.
That record describes candidate-time status; this note records the subsequent
local write. Replay from disk matches the written source byte-for-byte and a
second application is unchanged.

Pre-write evidence:

- Regression reproduces `value is not callable` before the change and passes
  afterward, exercising Var-backed input and output callbacks.
- Complete source/test pair: 56 facts pass, including cleanup verification.
- Native `code.manage.unit`: no new, incomplete, or unchecked obligations.
- Installed scaffold preview and `--write` both abort with stack overflow;
  the native inventory API is the recorded fallback.

Post-write status: source and path-matched test written; scoped diff check
passes. `pointer-main-written-job` loaded the written source and test in a
fresh native process: 56 facts pass, with cleanup verified. The standalone native `run` cannot locate
project source dependencies, and the installed project CLI rejects the existing
dependency namespace configuration `:role`; neither is a passing source check.

The historical JS test still fails with `value is not callable` even with this
fix, so this is a confirmed callback defect, not proof the entire JS invocation
is repaired. Temporary invocation tracing is running to locate the remaining
failure. The trace locates the error in `ptr-invoke-script`, before the changed
`ptr-invoke` boundary; investigation continues into emission callbacks.
No commits or publication performed.
