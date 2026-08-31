# Hara Foundation annex

<!-- generated from the canonical Hara API manifest; do not edit by hand -->

This annex separates the current portable Foundation API from automatic aliases, native static objects, and historical namespace migrations.

## Pinned source

- Repository: `https://github.com/hara-lang/hara`
- Ref: `df48836e9d9e10b34e4481b13194ec29f4068515`
- Commit: `df48836e9d9e10b34e4481b13194ec29f4068515`
- Manifest schema: `2`
- Surface digest: `sha256:54102b5ac431442726d2972f65c5e7645696322528ea90acf29495d37d26fe62`
- Migration digest: `sha256:6aed929f395bb42db7f11f2ce819c64c10880eb2fba178d3dd426bc14f1fe11f`

## Current Foundation surface

The root `std.foundation` namespace contains **338** public bindings. It is represented separately from the **5** child namespaces.

| Current child namespace | Alias | Public bindings | Profiles |
| --- | --- | ---: | --- |
| `std.foundation.bytes` | `bytes` | 8 | jvm, rust, wasm |
| `std.foundation.coroutine` | `co` | 7 | jvm, rust, wasm |
| `std.foundation.pretty` | `pretty` | 13 | jvm, rust, wasm |
| `std.foundation.promise` | `promise` | 11 | jvm, rust, wasm |
| `std.foundation.string` | `str` | 49 | jvm, rust, wasm |

Current children contain **88** public bindings in total.

## Native static objects

These runtime objects are recorded separately and do not contribute to the namespace count:

`Algo`, `Arr`, `Base`, `Bits`, `Bytes`, `Coroutine`, `Crypto`, `Document`, `Edn`, `Exception`, `File`, `Host`, `Iter`, `Json`, `Kernel`, `Maths`, `Num`, `OS`, `Obj`, `Package`, `Printer`, `Process`, `Promise`, `RegExp`, `Result`, `Runtime`, `Sandbox`, `Schema`, `Socket`, `Stream`, `String`, `Test`, `Work`.

## Historical migrations

| Former name | Status | Replacement or disposition |
| --- | --- | --- |
| `std.foundation.component` | `moved` | `std.lib.component` (namespace) |
| `std.foundation.crypto` | `moved` | `Crypto` (native-static-object) |
| `std.foundation.edn` | `moved` | `Edn` (native-static-object) |
| `std.foundation.file` | `moved` | `File` (native-static-object) |
| `std.foundation.host` | `moved` | `Host` (native-static-object) |
| `std.foundation.json` | `moved` | `Json` (native-static-object) |
| `std.foundation.kernel` | `moved` | `Kernel` (native-static-object) |
| `std.foundation.os` | `moved` | `OS` (native-static-object) |
| `std.foundation.pretty.engine` | `retired` | `std.foundation.pretty` (namespace) |
| `std.foundation.set` | `moved` | `std.foundation` (namespace-root) |
| `std.foundation.socket` | `moved` | `Socket` (native-static-object) |

Moved and retired names remain available as migration records, but they are not current `std.foundation.*` namespaces.

## Regeneration

Generate both the normative EDN and this README from the same pinned schema-v2 manifest:

```sh
python scripts/generate-foundation-annex.py \
  --input generated/foundation-api-manifest.json \
  --annex-output 01-lang/005-foundation-annex/draft/foundation-annex.edn \
  --readme-output 01-lang/005-foundation-annex/draft/README.md
```

CI must verify the pinned source commit and semantic digests before accepting regenerated output.
