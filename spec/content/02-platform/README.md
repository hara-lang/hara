# Hara platform specifications

Platform specifications define host-facing contracts above the core HAL data and reader layer. They cover executable tools, runtime services, packaging, and other interfaces that remain portable across supported Hara runtimes.

## Specifications

- [`000001-cli`](000001-cli/) — deterministic command-line applications, routing, options, handlers, outcomes, and the public Hara CLI contract.
- [`000002-tap`](000002-tap/) — federated Git-authoritative tap trust, discovery, roles, and the shared tap protocol metaspec.
- [`000003-identity`](000003-identity/) — root policy, GitHub enrollment, publisher keys, namespace grants, delegation, and revocation.
- [`000004-artifact`](000004-artifact/) — immutable digest-addressed objects, manifests, provenance, and registry attestations.
- [`000005-harp`](000005-harp/) — deterministic safe package archives, generated `package.edn`, and read-only mounting.
- [`000006-package`](000006-package/) — the single `project.edn` authoring contract, lockfiles, resolution, releases, installation, and yanking.
- [`000007-extension`](000007-extension/) — `project.edn` extension declarations, providers, ABI, targets, capabilities, and remote WASM delivery.
- [`000008-asset`](000008-asset/) — versioned file, image, video, and 3D collections with deterministic derivatives.
- [`000009-publishing`](000009-publishing/) — GitHub identity, exact-project intake, isolated builds, protected finalization, and accepted Git records.
- [`000010-distribution`](000010-distribution/) — exact release resolution, immutable downloads, verification, caches, and offline operation.
- [`000011-mirroring`](000011-mirroring/) — exact registry and object replication, divergence rejection, failover, and repair.
- [`000050-transport-hta`](000050-transport-hta/) — the Hara transport ABI boundary used by stateful extension providers.
- [`000051-transport-resp`](000051-transport-resp/) — the RESP evaluation-broker transport.
- [`000060-substrate-base`](000060-substrate-base/) through [`000069-substrate-runtime-profiles`](000069-substrate-runtime-profiles/) — portable substrate contracts.
- [`000100-lsp-base`](000100-lsp-base/) — the portable Language Server Protocol profile, shared analysis facts, IDE capabilities, and safety rules.

Material not yet assigned a numbered platform specification remains under [`../00-unsorted/`](../00-unsorted/).
