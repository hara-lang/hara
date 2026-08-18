## Summary

Materialize the portable schema registry product code that PR #774 intended to publish.

PR #774 merged its branch-only candidate payload and validator, but not `core/lib/src/std/typed/registry.hal` or the registry-aware `std.typed.schema` changes. This repair applies that reviewed candidate to product paths, removes all staging machinery, and closes the Rust recursive-validation stack overflow by carrying registry and cycle state explicitly rather than through recursive dynamic bindings.

Validation covers complete pre-write candidate evaluation, fresh-process HAL probing, the dependency boundary audit, Java/Truffle parity, and Rust parity.

Tracks #667.
