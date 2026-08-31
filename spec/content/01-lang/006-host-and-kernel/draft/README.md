# Hara host and kernel boundary

Status: **draft**

The authoritative document is
[`host-kernel-spec.edn`](host-kernel-spec.edn). It conforms to the
[`Hara meta-specification meta-spec`](../../000-metaspec/draft/metaspec-metaspec.edn).

This layer owns the runtime authority contract between a Hara evaluator and
its embedding host: kernel and session identity (`[:kernel/id :session/id]`),
capability grant resolution (available = provider-installed, granted = policy
∩ available, deny-by-default), the provider dispatch pipeline
(`resolve-session → require-grant → validate → invoke-provider`), promise
settlement with `:host/*` ex-info codes, session transfer rejection of live
values, managed SandboxSpec validation, sandbox provider dispatch and
cancellation, private sandbox Session ownership, and the browser/native
embedding wire contracts.

The guest-visible surface is outside this layer. `std.native.Host`,
`std.native.Kernel`, and `std.native.Sandbox` descriptors, capability ids, and `:native/*` errors belong
to [003-native](../../003-native/draft/native-spec.edn); the
`std.foundation.host` / `std.foundation.kernel` facades belong to
[005-foundation-annex](../../005-foundation-annex/draft/foundation-annex.edn),
while `std.sandbox` is an explicitly required portable facade specified by
[004-foundation](../../004-foundation/draft/foundation-spec.edn).
This layer owns what those surfaces enforce — see the `:host-kernel/boundaries`
map in the spec for the full ownership table.

[`filesystem-provider-spec.edn`](filesystem-provider-spec.edn) defines the
trusted `IFilesystemFactory` registry, opened `IFilesystem` capability,
single-root Session attachment, asynchronous call context, redacted mount
descriptors, capabilities, pagination, revisions, conflict handling, provider
close, stable cross-provider failures, and conformance requirements for native,
memory, IndexedDB, SFTP, GitHub, and Google Drive mounts. The adjacent
[`conformance/filesystem-provider.edn`](conformance/filesystem-provider.edn)
is the normative provider corpus.

[`session-snapshot-spec.edn`](session-snapshot-spec.edn) defines HSS0 portable
startup images, incremental layers, sealed and overlay restore modes, secret
requirements, and the limits that keep live authority out of snapshots.

It promotes the unsorted
[`host-runtime.edn`](../../../00-unsorted/runtime/draft/host-runtime.edn)
document; once adopted, that file's normative entries live here and the
`HaraHostSessionConformanceTest` spec path should be repointed accordingly.
