# Bench completion regression — 2026-09-15

The original-source migration remains active and type-bench is still staged,
not installed. Authority: fe54cc866473a888bfbd4ce2d3de6d8011f09f72.

The existing shell-forwarding candidate passes all 23 facts on the rebuilt
native 0.1.33 runtime with Node available on PATH. The formerly failing
autonomous-completion probe also passes against that candidate: after waiting
on an unrelated timer, the active bench entry has disappeared without awaiting
its completion field. This supersedes the failing observation in
lang_runtime_basic_type_bench_completion_boundary.edn for the current runtime.

The probe is now retained as a referenced regression in Foundation's
resources/code/migrate/candidates/lang_runtime_basic_type_bench_autonomous_regression.edn.
The record chains to the shell-forwarding parent, preserves its source and all
23 existing tests, and provides a replay form that appends the new assertion.
Candidate: 24 passed. Deliberately wrong autonomous-completion expectation:
24 passed, one failed, zero errors. Saved-record readback and repeated replay
produce equal source/test outputs; the replayed program passes all 24 tests in
a fresh native process. Each new test uses an isolated active registry and
stops its bench in finally. No production bench file was written.

This does not establish general background scheduling. A separate fresh probe
observed Promise/run execute its callback before returning: state was
[[:ran] [:ran]], not the deferred-callback expectation [[] [:ran]]. Ryuk's
scheduling boundary remains open. Process-tree cleanup, inherited stdio,
diagnostic/exit reporting fidelity, prerequisite installation, type-basic
integration, and real JS/Lua/Python bootstrap pipelines also remain incomplete.
Killed processes still print native exit -1 in the candidate suite.

No native scheduling change, Docker operation, commit, or publication was
performed. The temporary stdin test symlink was removed after validation.
