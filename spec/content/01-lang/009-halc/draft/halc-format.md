# HALC portable compiled-form format

HALC is the runtime-neutral compiled-form artifact for HAL. A `.halc` module carries canonical reader forms, portable metadata, source identity, and integrity information so a host can skip source decoding without committing to an execution engine.

> HALC is a portable compiled-form artifact. It is not Java AST data, Rust VM bytecode, analyzed compiler IR, or native code.

Java and Rust are peer implementations of this contract. Either runtime may encode a module and the other must decode and evaluate it with the same portable language behavior:

```text
source.hal -> module.halc -> HalcModule { forms, metadata, identity }
                                      |-> Java Truffle lowering
                                      `-> Rust evaluator or Rust VM compiler
```

Truffle nodes, Rust `Program` or `Instruction` values, evaluation journals, JIT traces, Trace IR, host handles, and caches must never be embedded in HALC.

## Version 1 envelope

All integers are unsigned and big-endian unless stated otherwise. Decoders reject trailing bytes after the declared payload.

| Field | Type | Version 1 value |
|-------|------|-----------------|
| magic | 4 bytes | `48 41 4c 43` (`HALC`) |
| format version | u16 | `1` |
| capability flags | u16 | defined below |
| payload length | u32 | number of following payload bytes |
| payload checksum | 32 bytes | SHA-256 of the complete payload |
| payload | bytes | canonical module payload |

Unknown versions and unknown capability bits are invalid. A decoder must not silently discard a requirement it does not understand.

### Capability flags

| Bit | Name | Meaning |
|-----|------|---------|
| 0 | executable foundation | module uses the closed foundation bootstrap subset |
| 1-15 | reserved | must be zero in version 1 |

Flags describe portable requirements of decoded forms. They do not select a host compiler, VM, AST representation, optimization tier, or JIT.

## Module payload and identity

A `string` is a u32 byte length followed by well-formed UTF-8 bytes. A `nullable-string` is a one-byte flag (`0` absent, `1` present) followed by a string when present. A `count` is a u32.

| Field | Type | Meaning |
|-------|------|---------|
| module identity | string | stable logical module or namespace identity |
| source identity | string | stable logical resource name, not a host filesystem path |
| source checksum | 32 bytes | SHA-256 of exact source bytes, or all zeroes when unavailable |
| form count | count | number of top-level forms |
| forms | values | canonical encoded forms |

Identities are data, not authority. A decoder must not read a path, fetch a URL, or grant a capability merely because an identity contains one.

## Canonical value encoding

Every value starts with a one-byte opcode:

| Opcode | Value | Payload |
|--------|-------|---------|
| 0 | nil | none |
| 1 | false | none |
| 2 | true | none |
| 3 | integer | signed 64-bit two's-complement bytes |
| 4 | float | 8-byte finite IEEE 754 binary64 bits |
| 5 | big integer | canonical decimal string |
| 6 | decimal | canonical decimal string |
| 7 | string | string |
| 8 | character | u32 Unicode scalar value |
| 9 | symbol | nullable namespace, name string, metadata |
| 10 | keyword | nullable namespace, name string, metadata |
| 11 | list | count, values, metadata |
| 12 | vector | count, values, metadata |
| 13 | map | count, key/value pairs, metadata |
| 14 | set | count, values, metadata |
| 15 | ordered map | count, key/value pairs in semantic order, metadata |
| 16 | ordered set | count, values in semantic order, metadata |
| 17 | regex | pattern string |

No opcode contains evaluation state or host identity. Unsupported opcodes are invalid.

Big integers use `0` or an optional leading `-` followed by digits with no redundant leading zeroes. Decimals use the portable HAL decimal spelling and round-trip without locale-dependent formatting. Floats preserve finite binary64 bits; encoders and decoders reject every NaN and positive or negative infinity bit pattern.

Characters must be Unicode scalar values; surrogate code points and values above `0x10ffff` are invalid.

## Metadata

Metadata is a one-byte flag (`0` absent, `1` present). When present, exactly one map value follows. Non-map metadata is invalid. Metadata recursively follows the same value rules and limits.

Metadata is portable descriptive data. Runtime caches, resolved Vars, Java classes, Rust types, AST nodes, bytecode offsets, journal state, and source objects are forbidden.

### Schema Var references

When a function's `:schema` metadata is a Var reference, a HALC encoder resolves it in the module's namespace environment before writing the artifact. Unqualified references such as `#'Customer` and current-namespace references such as `#'-/Customer` are encoded as the fully qualified symbol in `(var module.namespace/Customer)`. A namespace alias from the module's `ns :require` declarations is likewise replaced by its target namespace.

The referenced Var must exist in the compilation environment. At minimum, an encoder must reject a missing Var in the module being encoded; a compiler with loaded dependencies must also reject a missing external Var. Inline schema values remain ordinary metadata values and are not subject to Var lookup.

A local `def` reached through a function's named schema reference is a named schema definition. Var references nested anywhere in that schema value are resolved by the same rules, recursively. Local nested references must name an existing Var and are followed transitively; cycles, including a schema referring to itself, are valid and terminate by Var identity. Only definitions reachable from a function's `:schema` participate in this schema-graph pass, so an unrelated ordinary `def` containing a Var value is not reclassified as a schema.

This normalization changes form content only. It adds no opcode, capability bit, or host-specific resolved-Var object to the version 1 wire format.

After decoding, a compiler may materialize a semantic schema index from these canonical forms. The index maps fully qualified function Vars to their schema annotations and fully qualified named-schema Vars to the transitively reachable schema values. It is derived data rather than a second wire representation. A runtime that retains compiled modules must make the same index available to later lowering and optimization tiers instead of rediscovering schema references from source text.

A compiler may additionally derive conservative function types from the decoded executable forms. Declared schemas and inferred facts are separate: annotations remain user contracts, while inference records only what the compiler can prove and uses an explicit unknown type elsewhere. Java and Rust must derive equivalent facts for the shared initial inference domain (literals, schema-seeded parameters, lexical locals, `let`, `do`, `if` joins, collection literals, arithmetic, comparisons, and `count`). These facts are downstream compiler data and do not alter HALC version 1 bytes.

## Collection ordering

Plain map entries sort by unsigned lexicographic order of each canonical encoded key. Plain set elements sort by unsigned lexicographic order of each canonical encoded element. Duplicate canonical keys or elements are invalid.

Ordered maps and ordered sets retain semantic insertion order. Lists and vectors retain positional order.

Version 1 decoders accept noncanonical plain-map and plain-set order for legacy compatibility, but encoders always emit canonical order. Re-encoding a decoded module therefore produces canonical bytes.

## Regex restrictions

A regex stores only its portable pattern string. Host-specific flags, dialect objects, and compiled matcher state cannot be encoded. An encoder rejects a regex whose behavior cannot be represented by the portable pattern. A decoder may reject a pattern outside the HAL regex contract; it must not silently reinterpret it with host-specific behavior.

## Resource limits

Implementations may expose stricter configured limits, but never accept values beyond these version 1 maxima:

| Resource | Maximum |
|----------|---------|
| artifact bytes | 64 MiB |
| payload bytes | 64 MiB minus envelope |
| one UTF-8 string | 16 MiB |
| top-level forms | 1,000,000 |
| one collection count | 1,000,000 |
| nesting depth | 256 values |
| total decoded values | 4,000,000 |

Lengths and counts are checked before allocation. Arithmetic is checked for overflow. An oversized artifact is invalid even if the host could allocate it.

## Invalid-artifact behavior

Decoding is atomic and side-effect free. Before returning a `HalcModule`, a decoder validates magic, version, capability bits, envelope and payload lengths, payload SHA-256, UTF-8 and scalar validity, opcode payload shape, metadata shape, duplicate map keys and set elements, numeric canonicality, all limits, and complete input.

Truncation, checksum mismatch, unsupported versions or flags, malformed values, limit violations, and trailing bytes produce an artifact error. Failed decoding must not evaluate partial forms, define Vars, mutate namespaces, consult filesystem or network capabilities, or interpret artifact bytes as source.

Fallback from a missing or invalid optional `.halc` resource to separately packaged `.hal` source is loader policy, not decoder behavior. Strict loaders surface the artifact error.

## Cross-runtime compatibility

For every shared conformance case:

- Java-encoded HALC decodes in Rust;
- Rust-encoded HALC decodes in Java;
- both encoders produce identical canonical bytes;
- decoded forms and portable metadata are equivalent; and
- evaluating source and its HALC module has equivalent observable HAL behavior.

Java-specific lowering begins only after `HalcModule` decoding. Rust VM compilation begins only after `HalcModule` decoding. Neither downstream representation changes this wire contract.

## Legacy `HIR\0` policy

The former version 1 artifact uses magic `48 49 52 00` (`HIR\0`) with the same envelope and value layout. During the compatibility window:

- decoders may accept `HIR\0` version 1 as legacy input;
- decoded legacy input becomes an ordinary `HalcModule` plus a legacy-origin marker;
- encoders emit `HALC`, never `HIR\0`;
- `.hir` is an accepted legacy input extension only;
- `.halc` is the canonical output and packaging extension; and
- strict mode still validates checksum, flags, limits, and complete input.

Legacy decoding does not make HIR a second format contract. Removing the compatibility reader requires an explicit format-version policy change and migration notice.

## Conformance artifacts

The runtime-neutral corpus lives under `draft/conformance/`:

```text
values.edn
invalid-artifacts.edn
golden/complete.halc
golden/legacy-v1.hir
```

`values.edn` covers every scalar, namespaced symbols and keywords, metadata, every collection category, canonical ordering, nested forms, regex restrictions, and source/HALC evaluation parity. `invalid-artifacts.edn` covers checksum corruption, truncation, unsupported versions and flags, oversized fields, malformed UTF-8 and values, duplicate canonical entries, and trailing bytes. The golden files pin canonical HALC and legacy-reader compatibility across Java and Rust.
