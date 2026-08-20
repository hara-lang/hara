# Native behavioral conformance

The authoritative behavioral corpus for the native boundary is:

`core/lib/test-fixtures/std/foundation/native_method_conformance.hal`

The live Java and Rust inventories define the surface. Tests derive type and method
counts from those exports and compare the resulting method keys with the corpus.
No total is copied into a test constant.

## Classification contract

Every live `Type/method` key appears exactly once and has one classification:

- `:portable` references a catalogued reason, includes an executable fixture and must normalize to the same result
  in every applicable runtime.
- `:capability-specific` names the required capability and references a reviewable reason explaining why the case
  cannot run without a provider.
- `:inventory-only` references a non-empty reason in the shared reason catalog. It is not a silent
  skip and should be promoted when a stable observation is available.

The Java Truffle tests, Rust evaluator tests, Rust bytecode agreement probe, and
Chromium/Wasm lane all consume this same file. A second runtime-local copy is
forbidden.

## Required coverage

The corpus and boundary report exercise canonical `Type/method` invocation,
representative values, arity and receiver failures, normalized error outcomes,
persistent conversion behavior, mutable bytes/array/object operations, iterator
cleanup, `has?`, and the distinction between native `function?` and HAL `fn?`.
The Rust and JVM guards also pin the `vec` and `set` identity fast paths.

Provider-backed methods remain explicit capability cases. Their dedicated suites
must test lifecycle and cleanup with the provider installed.

## Changing the native surface

When adding, removing, or renaming a native method:

1. Change both live runtime inventories and their implementation.
2. Add or update exactly one corpus entry.
3. Prefer a portable fixture. Otherwise name the capability or document the
   inventory-only reason.
4. Add normalized success and failure observations where applicable.
5. Run the Java, Rust evaluator, Rust bytecode, and browser/Wasm conformance lanes.
6. Update the external language contract only after the source-owned closure
   tests agree.

The closure tests deliberately mutate their parsed key set to prove that an
unclassified addition, removal, or rename fails.
