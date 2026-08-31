# Hara protocol meta-specification

Status: **draft**

[`protocol-metaspec.edn`](protocol-metaspec.edn) defines the required shape of
the `std.protocol.*` specification.

A conforming protocol document declares the complete built-in protocol
catalog, canonical namespaces, method arities, inheritance, dispatch,
extension behavior, stable errors, and executable evidence. Protocol identity
and behavior must not depend on Java interfaces, Rust traits, or other
implementation objects.
