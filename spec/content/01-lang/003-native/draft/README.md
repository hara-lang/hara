# Hara native runtime boundary

Status: **draft**

The authoritative document is
[`native-spec.edn`](native-spec.edn). It conforms to the
[`Hara native-boundary meta-spec`](../metaspec/README.md).

This layer owns 33 guest-visible native descriptors and their canonical
`Type/method` Vars. The short form resolves through the descriptor registry to
the canonical `std.native.Type/method` identity. It defines capability checks
and runtime-profile parity without exposing general Java, Rust, JavaScript, or
host reflection.

`std.native.Sandbox` is capability-gated. It exposes only the provider-neutral
open, eval, value-call, cancel, status, and close lifecycle; provider
registration and backend handles remain trusted embedding concerns.

`std.native.File` remains the sole guest-visible mounted filesystem boundary.
Provider construction, capabilities, revisions, pagination, cancellation, and
close semantics are owned by the provider-neutral
[`filesystem-provider-spec.edn`](../../006-host-and-kernel/draft/filesystem-provider-spec.edn).
No provider-specific operation is added to `File/*`.

HAL wrappers are outside this layer. `String/trim` belongs here;
`std.foundation.string/trim` belongs to the Foundation annex.
