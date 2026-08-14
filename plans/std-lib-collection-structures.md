# `std.lib.collection` structure roadmap

Status: active roadmap

`std.lib.collection` owns persistent collection structures that are useful but
not foundational language literals. Its first public families are ordered maps
and sets, sorted maps and sets, queues, persistent deques, stable priority maps,
and string-keyed tries. Hash maps, hash
sets, lists, vectors, and their literal forms remain in `std.foundation`.

## Delivered: finger-tree deque

A persistent count-measured finger tree now backs `deque` in Java and Rust. The
public API exposes deque semantics—indexed observation and persistent operations
at both ends—without exposing the internal tree as a separate value family.

`queue` remains its existing stable public type. Deques preserve identity in
HTA with extension tag 36 and use the flat `std.lib.collection/peek-*`,
`pop-*`, and `push-*` operations backed by the canonical native protocols.

Remaining optional finger-tree work:

- general measured values beyond element count;
- public split and concatenation operations if real consumers need them;
- benchmarks against the existing segmented queue.

## Delivered: priority map

`priority-map` is persistent in Java and Rust, orders entries by ascending
natural priority, and retains insertion order for ties. Re-associating an
unchanged priority preserves position; moving a key appends it to the target
priority bucket. HTA tag 37 preserves the distinct collection identity.

## Next candidate: Bloom filter

A Bloom filter is the next useful specialized structure for large membership
checks where false positives are acceptable and false negatives are not. It is
not a set and must not implement exact set equality or lookup semantics.

Java and Rust must share deterministic hashing, bit ordering, capacity/error
parameter validation, merge compatibility rules, and HTA serialization. The
constructor should make the expected item count and false-positive target (or
the derived bit/hash counts) explicit.

## Later candidates

- a persistent priority queue or heap only if priority-map does not cover the
  consumer's keyed scheduling needs;
- a persistent bit set;
- a multimap facade over existing map/set families;
- a disjoint-set structure for graph algorithms;

Each candidate belongs here only when it has meaningful cross-runtime value
semantics. Algorithm helpers over ordinary maps and vectors should remain
ordinary library functions rather than new runtime types.
