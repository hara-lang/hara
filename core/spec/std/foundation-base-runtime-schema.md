# Foundation Base Shims, Runtime Types, and Typed Schemas

## Status

This document defines the portable Foundation contract. Implementations may use
different host representations, but Java, Rust, interpreted source, and bytecode
must expose the same observable values and errors.

## Foundation and native ownership

`std.foundation` owns the portable root API. Operations implemented by the
runtime live on `std.native.Base`; Foundation exposes explicit forwarding
functions marked `^{:inline true}`. The metadata is a request, not a handwritten
rewrite rule: the compiler validates that the function body is a transparent
forwarder and derives its target from that body. Invalid inline declarations are
compile errors. Direct calls may lower like macros, while the Var remains an
ordinary first-class function for indirect calls.

Adding a portable operation to `Base` requires a corresponding Foundation shim.
`Arr` and `Obj` retain their mutable host-oriented APIs. Specialized persistent
collection constructors and predicates belong to `std.native.Algo`; public
`std.lib.collection` functions remain explicit wrappers such as
`(defn deque? [x] (Algo/deque? x))`.

The baseline bootstrap consists only of the Foundation namespaces required to
load and compile portable source. `std.lib.resp` and other library packages are
package-tier resources. `hara.compiler`, `hara.verify`, and
`hara.transpile.base.*` are not bootstrap dependencies.

## Base surface

Base includes constructors, primitive predicates, `satisfies?`, `type`, and
`instance?`. `tuple` accepts zero through eight values. `pair?` is true only for
the two-element tuple representation. `vec` and `set` convert an iterable into
their persistent collection types. `not-nil?` is the exact complement of
`nil?`. `reduce-in` remains a portable Foundation algorithm because its protocol
composition is not a primitive runtime operation.

## Runtime type values

`type` returns flat keywords:

- Native values use `:std.native.<Name>`, including `RegExp`, `Tuple`,
  `Promise`, `Coroutine`, `Namespace`, `NativeType`, `StructType`,
  `MutableType`, and `SchemaType`.
- A named struct or mutable instance uses `:<declaring-ns>.<Type>`, for example
  `:geometry.Point`.
- A native descriptor such as `Base` has type `:std.native.NativeType`.

`instance?` accepts a generated struct/mutable descriptor or a concrete native
descriptor that declares an `instance?` method. It rejects operational native
descriptors. Defining a struct or mutable does not generate
`<Type>/instance?`; generic `instance?` is the sole named-type predicate.

A loaded namespace and an installed alias may be referenced as values without
causing an implicit load. Resolution precedence is lexical binding, Var, loaded
namespace or alias, then unbound-symbol error.

## Schema values and Var contracts

`schema` compiles schema data into an immutable `SchemaType`. It accepts:

- raw shorthand data such as `[:map [:name :str]]` or `[:int]`;
- canonical longhand data such as `{:kind :map :children [...]}`;
- an existing `SchemaType` (idempotently);
- a Var whose contained value is raw schema data or a `SchemaType`.

`(schema #'description)`, `(schema description)`, and `(schema [:int])` are
structurally equal when `description` contains `[:int]`. Only the Var form has
that Var as its origin; origin is excluded from equality and hashing.
`(schema #'customer-name)` and `(schema customer-name)` are errors when the
value is not schema data. In particular, `schema` never reads a Var's `:schema`
metadata.

`schema-of` is the contract lookup operation. It accepts only a Var reference:
`(schema-of #'customer-name)` returns the compiled contract snapshot or `nil`.
Passing the function value is an error. Contracts belong to Vars and functions
do not inherit them.

Metadata may point at a schema-data Var:

```clojure
(def description [:int])
(defn ^{:schema #'description} customer-name [customer] (:name customer))
```

The compiler resolves and snapshots this contract when the definition is
compiled or reloaded. Later mutation of `description` does not silently change
the already compiled contract.

`Schema/kind`, `Schema/form`, `Schema/ast`, and `Schema/origin` inspect schema
values; `Schema/instance?` recognizes them. `(type (schema value))` is
`:std.native.SchemaType`. Printing is round-trippable as
`(schema <canonical-short-form>)`.

`Schema/origin` returns provenance, not another schema. Consequently
`(Schema/origin (schema #'customer-name))` is valid as an origin query even
though its result is not a `SchemaType`.

## Conformance requirements

Java and Rust must share tests for tuple arities, all flat type keywords,
native and named `instance?`, namespace values and precedence, inline shim
validation/lowering, schema short/long normalization, Var-origin equality,
schema errors, contract snapshotting, and interpreted/bytecode parity.
