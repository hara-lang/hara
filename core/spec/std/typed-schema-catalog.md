# Portable schema catalog contract

Status: **implementation contract for #868**  
Depends on: `std.typed.schema` parity from #832 / PR #836

## Purpose

`std.typed.schema` remains the canonical structural schema algebra. A catalog adds
stable distributed identity, immutable version coordinates, deterministic
resolution, dependency indexing, and artifact linking around that algebra.

Anonymous schemas remain valid and do not receive implicit identities:

```clojure
[:map [:name :str]]
```

An identified schema is a catalog-entry envelope:

```clojure
{:schema/id :user/account
 :schema/version 3
 :schema/hash "sha256:..."
 :schema/form
 [:map
  [:id (var user/Id)]
  [:profile (var user/Profile)]]}
```

Identity never becomes a schema-node property. The normalized AST is still the
same map produced by `std.typed.schema/normalize`.

## Entry contract

A canonical entry contains:

```clojure
{:schema/entry-type :std.typed.catalog/entry
 :schema/id :user/account
 :schema/version 3
 :schema/hash "sha256:<64 lowercase hexadecimal digits>"
 :schema/name user/account
 :schema/form <portable source form>
 :schema/ast <canonical normalized AST>
 :schema/dependencies
 [{:schema/id :user/id
   :schema/version 1
   :schema/hash "sha256:..."}]}
```

Rules:

- `:schema/id` is a qualified keyword.
- `:schema/version` is a non-negative integer. Higher values are newer for
  latest-version selection; versions are otherwise immutable labels.
- `:schema/name` is the qualified symbol spelling of `:schema/id` and is used by
  the existing structural registry.
- `:schema/form` is portable schema data accepted by `std.typed.schema`.
- `:schema/ast` is the normalized structural form and is not a parallel AST.
- `:schema/dependencies` contains exact immutable coordinates, never mutable
  latest-version selectors.
- A supplied hash must equal the recomputed canonical hash.

Envelope fields not listed in the hash input may carry provenance, source paths,
publish timestamps, signatures, receipts, or build metadata without changing
schema identity. Schema-authored node and map-entry properties inside
`:schema/form` remain part of canonical schema content. This distinguishes
portable contract annotations from volatile publication metadata.

## Hash contract

The algorithm identifier is:

```clojure
:std.typed.catalog/sha256-v1
```

The logical hash input is:

```clojure
[:std.typed.catalog/sha256-v1
 :user/account
 3
 <canonical normalized AST>]
```

The result is rendered as:

```text
sha256:<64 lowercase hexadecimal digits>
```

### Canonical token stream

Values are encoded as length-delimited typed tokens. Lengths count UTF-8 bytes,
not host string units. Supported values are:

- nil, booleans, numbers, characters, strings;
- keywords and symbols using namespace/name spelling;
- bytes and regular-expression patterns;
- maps, sets, vectors, lists, and other sequential values.

Map key/value pair tokens and set member tokens are sorted by unsigned UTF-8 byte
order. Ordered collections retain order. Every nested token is length-delimited,
so concatenation is unambiguous.

The SHA-256 digest covers the UTF-8 bytes of this token stream. Identical inputs
therefore hash identically on HAL, Rust, and Truffle/JVM regardless of host map
or set iteration order.

The shared initial golden vector is:

```clojure
(catalog-content-hash :demo/value 1 :int)
```

```text
sha256:3fc60b1736332b9f2e9f9e0a7dee75cc19c6287cc4e066970ef97b23a75fd34a
```

Changing the identity, version, normalized structure, or schema-authored
properties changes the hash. Changing excluded envelope provenance does not.

## Catalog composition

A canonical catalog contains:

```clojure
{:catalog/type :std.typed.catalog/catalog
 :catalog/registry <std.typed.registry value>
 :catalog/entries {[:user/account 3] <entry>}
 :catalog/latest {:user/account 3}
 :catalog/parents [<catalog> ...]}
```

Catalog construction accepts an entry sequence or identity-keyed map and the
existing registry options:

```clojure
{:namespace user
 :aliases {shared common}
 :refers {Id user/id}
 :parents [base-catalog]}
```

Local exact coordinates take precedence over parents. A local entry may repeat a
parent coordinate only when its canonical hash is identical. Conflicting content
at the same identity/version coordinate is invalid.

Latest lookup chooses the numerically greatest visible version. Exact lookup by
identity/version, or by identity/version/hash coordinate, never falls back to a
different version or hash.

## Resolution and dependencies

Catalog construction projects the latest visible entry for every identity into
the existing `std.typed.registry`. Normalization and recursive resolution then
reuse `std.typed.schema`; no second traversal engine is introduced.

Named references are qualified through the catalog registry before dependency
indexing. Each direct edge is pinned to the exact visible coordinate selected at
catalog construction.

Dependency reports contain:

```clojure
{:schema/coordinate <exact coordinate>
 :dependencies/direct [<coordinate> ...]
 :dependencies/transitive [<coordinate> ...]
 :dependencies/recursive [<coordinate> ...]}
```

Ordering is deterministic first-discovery order over canonical schema traversal.
Duplicate coordinates are removed without reordering the first occurrence.

An alias-only cycle is invalid because it never reaches structural content. A
recursive structural schema is valid; back-edges are reported in
`:dependencies/recursive`.

## Public operations

The stable catalog surface is:

```clojure
(catalog/catalog entries options)
(catalog/lookup catalog identity)
(catalog/lookup catalog identity version)
(catalog/resolve catalog identity version)
(catalog/dependencies catalog identity version)
(catalog/verify catalog)
```

The `std.typed` facade publishes equivalent `catalog-*` operations. Existing
anonymous schema, registry, validation, inference, and explanation APIs remain
unchanged.

## HBC contract

Existing anonymous `schema_types`, function schemas, and inferred function
schemas remain byte-compatible.

An HBC program may additionally carry a catalog section containing embedded
canonical entries and their exact dependency coordinates. The section is
optional and append-only relative to the current payload:

- an artifact without the section decodes as an empty catalog;
- an artifact with the section embeds enough data to verify every entry hash;
- dependency links contain identity, version, and hash;
- execution never resolves an unpinned latest version;
- entries and dependency coordinates are emitted in canonical UTF-8 byte order;
- decode followed by encode reproduces identical bytes.

This supports both self-contained artifacts and exact links. Distribution tooling
may omit an already-available embedded dependency only when the artifact retains
its full exact coordinate and the admitting catalog verifies that coordinate.

## Verification

`catalog/verify` recomputes every visible entry hash and checks every dependency
coordinate against the visible catalog. The result is structured data:

```clojure
{:valid true
 :errors []}
```

Failures identify the owning coordinate and either a hash mismatch or unresolved
exact dependency. Package and specs-registry admission must reject catalogs whose
verification result is not valid.

## Layering

```text
std.type
  primitive runtime predicates and protocols

std.typed.schema
  anonymous structural schema AST and runtime validation

std.typed.registry
  structural name qualification and recursive lookup

std.typed.catalog
  identity, versions, hashes, exact dependency graph, artifact coordinates

std.typed.infer
  compiler-facing static inference
```

This tranche does not add schema evolution, migration, coercion, SQL, UI, or
code-generation targets.
