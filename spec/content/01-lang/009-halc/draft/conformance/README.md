# HALC conformance corpus

This directory is the runtime-neutral acceptance surface for HALC. Java and Rust consume the same manifests and golden files; runtime-local copies are not authoritative.

`values.edn` defines portable form cases. `invalid-artifacts.edn` defines deterministic mutations and required rejection categories. Binary goldens are generated only by a successful cross-runtime interoperability run:

```text
golden/complete.halc   canonical HALC v1 emitted identically by Java and Rust
golden/legacy-v1.hir   frozen former HIR v1 accepted only by compatibility readers
```

Run `scripts/generate-halc-goldens`. The generator compiles `complete.hal` independently with Java and Rust, fails unless the bytes match, writes `complete.halc`, and derives the decode-only legacy fixture by changing only its magic. Runtime tests then decode both files directly from this directory.

Current SHA-256 values:

```text
4d1b0ba5671f11d7fc032883e73899866638ee429fa44f972b35470dbf706858  complete.halc
94e00b333f1c13191b54a51032a7364d7a37f9739e2b4f18b90c41e94ba86d8f  legacy-v1.hir
```

Goldens must not be hand-edited. A wire-format change requires a format-version decision, updated manifests, regeneration by both implementations, and review of the binary diff.

Invalid cases use mutation operations rather than large embedded byte arrays. Operations apply in order to the named base artifact. Unless a case explicitly recomputes the checksum, payload corruption retains the old checksum.
