# HAL core language specification

Status: **draft**
Target version: **1.0.0**

The authoritative document is
[`hal-langspec.edn`](hal-langspec.edn). This README is its human-readable
companion and must not introduce requirements absent from the EDN document.
The document conforms to the
[`HAL language metaspec`](../metaspec/README.md).

## Scope

This specification owns the host-neutral HAL core language: its shared data
profile plus evaluation, functions, bindings, definitions, errors, iteration,
namespaces, and modules. The data layer includes:

- nil and booleans;
- strings and characters;
- symbols and keywords;
- signed 64-bit integers and binary64 floating-point numbers;
- immutable lists, vectors, maps, and sets;
- bracket literals as logical vectors independent of compact or persistent
  runtime representation;
- structural equality, hashing, and metadata;
- UTF-8 reading and canonical readable representations.

The core contract also defines portable evaluation and the `(ns ...)` and
`(ns+ ...)` forms. `ns` selects a named definition scope; `ns+` reconfigures
the current scope without accepting a name. Namespace `:config` supports
`:blank`, `:override`, `:only`, `:rename`, `:set-global-alias`, `:set-global`,
and `:role`. `:rename` controls Foundation library aliases with `:exclude` and
`:alias`; `:set-global-alias` contributes a namespace alias globally;
`:set-global` contributes exact qualified Vars as terminal-name imports,
including compact protocol Vars such as `IColl/start-string`. `:intrinsics`
remains reserved for native/runtime intrinsic symbols and is not a namespace
alias operation.

Protocol, Native, and Foundation semantics remain normatively owned by their
numbered specifications. Their executable inventories and parity corpora are
co-located here so every runtime consumes one language-conformance tree.
Host capabilities and runtime implementation remain outside this document.

## Executable evidence

The central [`conformance/`](conformance/) directory contains the core and
reader corpora, module cases, the Protocol/Native/Foundation inventories, and
the JVM Truffle, Rust evaluator, Rust bytecode, and WebAssembly parity inputs.
The primary core contract is [`conformance/core.edn`](conformance/core.edn).

The previous broad platform-language document remains a migration source under
[`../../../00-unsorted/platform-language/`](../../../00-unsorted/platform-language/);
its executable corpora have moved here.
