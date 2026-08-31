# Hara persistent data structures

Status: **draft**

The authoritative document is
[`data-structures-spec.edn`](data-structures-spec.edn). The
`hara/data-structures-metaspec` it declares conformance to is **TODO** — the
metaspec layer for this area has not been written yet; see the area
[README](../README.md).

This layer owns the observable behaviour of the built-in persistent data
structures: the structure catalog (vector, list, queue, deque, map, set,
ordered-map, ordered-set, sorted-map, priority-map, sorted-set, trie), op semantics,
iteration-order guarantees, transient (Mutable) semantics, and the
Java-parity hashing stack (RAPID default; SYSTEM, MURMUR3, SIP).

`deque` is a persistent count-measured finger tree with efficient operations
at both ends. `priority-map` is a persistent map ordered by ascending natural
priority; equal-priority keys retain stable insertion order.

Vector is one public `:std.native.Vector` category with two runtime backends.
Bracket literals and portable `BUILD_VECTOR` use compact tuples for counts 0
through 8 and persistent vectors from count 9. The compact backend is hidden;
`pair?` and `map-entry?` recognise only dedicated `:std.native.MapEntry`
values. Equality, hashing, iteration, lookup, destructuring, metadata, and
display remain vector semantics, while maps accept only MapEntry values through
`conj`.

It does not own reader syntax for collection literals
([01-lang/001-language](../../001-language)) or protocol dispatch
([01-lang/002-protocol](../../002-protocol)).

## Executable evidence

- [`conformance/hash-parity.edn`](conformance/hash-parity.edn) — 275 cases
  pinning raw hash functions, canonical number hashing, string-type
  (keyword/symbol) hashing, hash seeds, and collection hash composition.
- [`conformance/collections.edn`](conformance/collections.edn) — 54
  behavioural cases replaying op scripts against the Java reference
  structures and pinning count, iteration, lookups, rank probes, and hashes.

Both corpora are generated from the Java reference runtime and are
byte-for-byte deterministic across runs:

```shell
mvn -f java/pom.xml -Ptruffle package   # once, for java/target/classes
javac -cp java/target/classes -d target/hashdump specs/scripts/hashdump/HashDump.java
java -cp target/hashdump:java/target/classes HashDump \
  > specs/01-lang/020-data-structures/draft/conformance/hash-parity.edn
javac -cp java/target/classes -d target/hashdump specs/scripts/hashdump/CollectionDump.java
java -cp target/hashdump:java/target/classes CollectionDump \
  > specs/01-lang/020-data-structures/draft/conformance/collections.edn
```

Regeneration caveats:

- The generators (`HashDump.java`, `CollectionDump.java`) are tracked in
  [`specs/scripts/hashdump/`](../../../scripts/hashdump/); `target/hashdump/`
  is only the gitignored build output.
- `java/target/classes` must be fresh — re-run the `mvn package` step before
  regenerating if the Java reference sources have changed.

Rust consumers: `lang::hash::tests::java_parity_fixture`
(`rust/src/lang/hash/mod.rs`) and `lang::data::conformance_tests`
(`rust/src/lang/data/conformance_tests.rs`).

## Reference-runtime repairs

Writing the collections corpus surfaced two defects in the Java reference
`Queue`: `Base.iterator` ignored the popped-head offset (iteration, display,
and hashing read stale elements after `pop-first`), and `Base.nth` swapped
segment/index past the head. Both were repaired so that the corpus pins
coherent behaviour; `hara.lang.data.QueueTest` passes (5/5).

A third finding was standardised rather than repaired: Java's
`SortedMap.Node` does not override `hashCode`, so Java's own sorted-map
`hashCalc` is identity-based and non-deterministic across JVM processes.
The corpus pins the standardised unordered MapEntry composition
(identical to the CHAMP map formula), matching the Rust runtime; see
`:sorted-map-identity-hash` in the spec.
