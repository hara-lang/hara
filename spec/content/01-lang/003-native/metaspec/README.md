# Hara native-boundary meta-specification

Status: **draft**

[`native-metaspec.edn`](native-metaspec.edn) defines the required shape of the
`std.native` and `std.native.*` specification.

A conforming native document declares every descriptor, method, runtime
profile, capability dependency, portable argument and result boundary, and
stable failure category. JVM, Rust, and Wasm differences must be explicit
profiles. Loading a namespace grants no authority.
