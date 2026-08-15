# Current `std.foundation` architecture

This document describes the implementation boundary represented by Hara's registered standard-library inventory. It is intentionally narrower than historical Foundation plans and annex drafts.

## Loadable Foundation namespaces

The current public Foundation family is:

```text
std.foundation
std.foundation.bytes
std.foundation.coroutine
std.foundation.pretty
std.foundation.promise
std.foundation.string
```

`core/rust/standard-library.namespaces` is authoritative for loadable standard-library namespace membership. A source file, test fixture, automatic alias, or native object does not become a public namespace merely because it is present in the repository or visible to an evaluator.

The root `std.foundation` namespace owns the portable value layer: composition, collections, sequence operations, set algebra, metadata, references, macros, structural traversal, and the small language-level helpers automatically referred into ordinary namespaces.

The five child namespaces provide separately aliased portable/native-backed library surfaces:

| Namespace | Default alias | Role |
| --- | --- | --- |
| `std.foundation.string` | `str` | portable string facade |
| `std.foundation.bytes` | `bytes` | byte values and operations |
| `std.foundation.promise` | `promise` | promises and protocol facade |
| `std.foundation.coroutine` | `co` | coroutine facade |
| `std.foundation.pretty` | `pretty` | document and pretty rendering |

## Native static objects

Runtime facilities such as `Edn`, `Json`, `Crypto`, `File`, `Socket`, `Host`, `Kernel`, `OS`, and `Process` are built-in static objects backed by `std.native.*` runtime identities. They are available without requiring file-backed `std.native.*` namespaces.

For example:

```clojure
(Edn/read "{:a 1}")
(Json/write {"a" 1})
(Crypto/sha256 bytes)
(OS/platform)
```

The presence of aliases such as `Edn` or identities such as `std.native.Edn` does **not** imply that `std.foundation.edn` or another retired Foundation child is loadable.

## Higher-level ownership

Functionality above the native substrate belongs to focused portable libraries:

- filesystem composition and portable path/file workflows: `std.fs`;
- formatting, tables, reports, and terminal presentation: `std.format.*`;
- component lifecycle: `std.lib.component`;
- cryptographic algorithms above native primitives: `std.crypto.*`;
- collection helpers not retained in the root value layer: `std.lib.collection`.

Planned replacements must be recorded as planned rather than presented as implemented.

## Migration and generated API data

`core/spec/std/foundation-migrations.json` records former names, their status, replacement or disposition, rewrite guidance, and evidence.

`scripts/generate_foundation_api_manifest.py` combines:

1. the registered namespace inventory;
2. source-derived public binding data;
3. runtime alias/native-object configuration; and
4. the migration ledger.

The resulting schema-v2 manifest is the source consumed by the specification registry and generated documentation. Downstream repositories must not maintain independent handwritten Foundation inventories.

## Test placement

Ordinary tests under `core/lib/test/std/foundation/` correspond only to current Foundation child namespaces. Root Foundation behavior lives in `core/lib/test/std/foundation*_test.hal`. Native/static-object behavior is tested without requiring retired `std.foundation.*` children. Capability-provider and higher-level library behavior belongs under the provider or portable library that owns it.
