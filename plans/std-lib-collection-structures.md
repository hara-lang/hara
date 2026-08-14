# `std.lib.collection` structure roadmap

Status: design backlog

`std.lib.collection` owns persistent collection structures that are useful but
not foundational language literals. Its first public families are ordered maps
and sets, sorted maps and sets, queues, and string-keyed tries. Hash maps, hash
sets, lists, vectors, and their literal forms remain in `std.foundation`.

## Priority 1: finger tree

A persistent measured finger tree is the strongest next primitive. It can
support efficient deque operations, concatenation, splitting, and indexed or
measured sequence views without baking each view into an unrelated structure.

The first implementation should expose the tree as a distinct value family and
keep `queue` as its existing stable public type. A later queue/deque facade can
adopt the tree internally after Java and Rust agree on persistence, iteration,
equality, hashing, metadata, and HTA serialization.

Acceptance work:

- specify the measure protocol and identity value;
- define amortized operation guarantees for both runtimes;
- cover split, concat, left/right insertion, and left/right removal;
- define a portable HTA representation before connector transport is enabled.

## Priority 2: Bloom filter

A Bloom filter is the next useful specialized structure for large membership
checks where false positives are acceptable and false negatives are not. It is
not a set and must not implement exact set equality or lookup semantics.

Java and Rust must share deterministic hashing, bit ordering, capacity/error
parameter validation, merge compatibility rules, and HTA serialization. The
constructor should make the expected item count and false-positive target (or
the derived bit/hash counts) explicit.

## Later candidates

- a persistent priority queue or heap;
- a persistent bit set;
- a multimap facade over existing map/set families;
- a disjoint-set structure for graph algorithms;
- a deque API, preferably backed by the finger tree rather than a new primitive.

Each candidate belongs here only when it has meaningful cross-runtime value
semantics. Algorithm helpers over ordinary maps and vectors should remain
ordinary library functions rather than new runtime types.
