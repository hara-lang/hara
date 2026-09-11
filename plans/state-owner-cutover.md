# Original Snapshot/Library cutover

Installed original `lang.core.library-snapshot` and `lang.core.library`, their
matching tests, and the original Lua/Snapshot test fixtures. Exact before-images
and assertion dispositions are retained under Foundation `resources/code/migrate`.

Post-write checks: Snapshot 37 passes; Library 44 passes. They load the actual
source files and explicitly loaded on-disk fixtures, but still supply staged
original XTalk/Lua models. This is not a standalone full-project green result.
The native runner does not resolve required fixture namespaces from test roots.

The existing entry suite has 27 passes and one integration error: old script
code still calls `lang.core.library/library-raw-snapshot`. Move those callers to
the original impl/script ownership; do not restore the rejected Library wrappers.
The prior color compilation limit is not yet reached on this path.

Next: original impl/script/model cutover, test-fixture loading, and remaining
caller-owned native contracts. Eight generated-source whitespace findings remain.
The catalog marks source installation separately from pending integration.
