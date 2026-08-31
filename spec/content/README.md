# Hara specification corpus

This is the versioned payload of the `hara/foundation.specs` HARP. It is built
by `std.package.build/build-specs` with the Hara source release and is delivered
through its paired GitHub Packages `.specs` image.

The public catalogue is `registry-index.json`. Each catalogue record resolves
its source and documentation through an artifact-relative path, never through
the former Git specification registry. `spec-manifest.json` inventories the
payload. `provenance.edn` records the imported registry revision and curation
boundary.

The release contains the complete numbered language and platform corpus,
including conformance and golden fixtures. It also contains the canonical
unnumbered Hara specifications. Greenways contributions, historical archive
material, registry tooling, and duplicate preliminary package/language
documents deliberately remain outside this artifact.
