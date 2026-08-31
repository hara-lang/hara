# `std.typed` catalog contract

This directory publishes the immutable catalog-entry and dependency/component
contract implemented by `std.typed.catalog` in `hara-lang/hara`.

The Hara repository owns schema normalization, semantic hashing, exact
coordinates, dependency extraction, and strongly connected components. This
registry owns the normative publication document, pinned fixture bytes,
provenance, and structural admission checks over those Hara-derived values.

## Published artifacts

- `std-typed-catalog.edn` — normative draft contract.
- `conformance/catalog-v2.json` — pinned identified-schema catalog fixture.

The fixture includes exact dependencies, a self-recursive schema, and
dependency-first component evidence.

Run:

```sh
npm test
npm run catalog:check
npm run check
```

The JavaScript validator deliberately does **not** normalize HAL schema forms or
recompute semantic schema hashes. It verifies immutable envelope identity,
canonical ordering, exact dependency completeness, recursive components,
provenance and the registry document digest.

The fixture pins the exact Hara revision and source paths from which its hashes
and graph evidence were derived. A Hara integration gate must read these
checked-in bytes and prove HAL, Rust, Truffle, and HBC agreement before
`hara-lang/hara#903` closes.
