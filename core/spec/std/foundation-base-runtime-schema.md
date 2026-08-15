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

Namespace inspection and dynamic evaluation live on `std.native.Env` and use
the same transparent Foundation-wrapper contract. Foundation exposes
`env-current`, `env-snapshot`, `env-vars`, `env-namespaces`, `env-namespace`,
`env-module`, `env-resolve`, `ns-alias-state`, `intern-var`, `eval-in-ns`, and
`eval`. `Env/eval` evaluates one form value in the current namespace;
`Env/eval-in` evaluates a collection of form values in an existing namespace.
Java and Rust must expose identical methods and evaluation behavior.

The same inline-forwarding rule applies throughout the embedded Foundation
family. Transparent shims over `Maths`, `Numbers`, `Bits`, `String`, `Bytes`,
`Promise`, `Coroutine`, and protocol methods carry `:inline true`, a public
docstring, and schema metadata. A wrapper that reorders arguments, supplies a
default, normalizes a result, performs capability policy, or composes more than
one call is not a transparent shim and must remain ordinary HAL.

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

## Pull streams

`Stream` is a native, asynchronous, unidirectional pull source. It implements
`IStream/next` and `IClose/close`; `(type stream)` is `:std.native.Stream`.
`Stream/next` returns a Promise which fulfills with one structured Hara value,
or `nil` at end-of-stream. Only one pull may be pending. Closing is idempotent
and a closed stream produces `nil`.

`std.lib.stream/generate` is the package-tier constructor. It owns a private
coroutine, supplies constructor arguments only on its first resume, exposes
yielded values one at a time, and discards the coroutine's final return value.
Because `nil` denotes EOF, yielding `nil` rejects the pull with
`stream/nil-item` and closes the stream. Generator errors reject the active
pull and close the stream. The namespace is deliberately absent from the
Foundation bootstrap bundle.

Foundation iterators are synchronous: `iter-next` either returns immediately
or the iterator is exhausted. `std.lib.stream/from-iterator` is the explicit
one-way bridge into Promise-based pulling. `unfold` accepts a direct or
promised step result of `[item next-state]`, with `nil` ending the stream.
`map`, `filter`, and `take` are lazy managed streams; `reduce` and `collect`
are Promise-returning terminals. Managed streams own and always close their
upstream source on EOF, error, early termination, or explicit close.

A stream is not duplex. Duplex transports compose a readable `IStream` with a
separate write operation; for example, a WebSocket exposes inbound messages as
a stream and outbound messages through `WebSocket/send`. Stream, coroutine,
and transport handles are worker-local and cannot cross session, HTA, snapshot,
or worker serialization boundaries.

Connected processes and sockets expose this composition directly.
`Process/duplex` receives stdout byte chunks and sends stdin byte chunks;
stderr remains independently observable through `Process/stderr-stream`.
`Socket/duplex` receives and sends byte chunks for one connected socket; a
listening socket is not a Duplex. Sends return Promises, receive sides preserve
the one-pending-pull Stream rule, and closing either Duplex is idempotent.

Duplex replaces transport-specific input/output plumbing, but not Relay.
Relay remains the portable layer for codecs and framing, serialized or
correlated exchanges, timeouts, pending-request dispatch, and unsolicited
events over a Duplex.

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

`SchemaType` implements `IDeref`. Dereferencing returns the normalized vector
shorthand, independent of the input spelling; for example, both `(schema :int)`
and `(schema [:int])` dereference to `[:int]`, while nested schemas dereference
recursively to forms such as `[:map [:name [:str]]]`.

`Schema/origin` returns provenance, not another schema. Consequently
`(Schema/origin (schema #'customer-name))` is valid as an origin query even
though its result is not a `SchemaType`.

## Conformance requirements

Java and Rust must share tests for tuple arities, all flat type keywords,
native and named `instance?`, namespace values and precedence, inline shim
validation/lowering, schema short/long normalization, Var-origin equality,
schema errors, contract snapshotting, and interpreted/bytecode parity.
