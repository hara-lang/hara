## Summary

Make one canonical schema AST shared by portable `std.typed`, Rust, and Java/Truffle.

- `Schema/ast` returns the same semantic map as `std.typed.schema/normalize`.
- Native `schema` accepts that canonical map and reconstructs an equal `SchemaType`.
- Retained `:children` longhand remains accepted as input but is immediately canonicalized.
- Canonical kinds distinguish `:union`, single-arity `:fn`, and multi-arity `:function`.
- Cross-runtime tests cover primitives, references, unions, nested collections, maps, variadic and multi-arity functions, enums, and extension schemas.

No retired `res-*` vocabulary is introduced; Result-related work continues to use `result`, `result?`, `result-status`, and native `Result/*`.

Tracks #667.
