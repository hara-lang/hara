# Metaspec logic spike

Tracking: hara-lang/hara#199

This is a non-normative working note. The EDN documents under `specs/` remain
authoritative.

## Why this layer exists

The spike tests one concrete pipeline:

```text
HAL ^{:schema ...} metadata and metaspec declarations
  -> std.typed registry and normalized schema data
  -> lossless path graph and typed relation tuples
  -> imperative checks, Datalog derivation, or miniKanren search
```

The original EDN remains available at `:graph/document`; vector paths are node
IDs. The relational representation is therefore an index, not a replacement
serialization.

## Schema ownership

`std.typed.registry` and `std.typed.schema` are the executable schema
substrate. They own immutable named-schema registries, reference
qualification and resolution, primitive and composite validation, recursive
cycle handling, path-aware findings, and the extension dispatch ABI.

`tool.metaspec.schema` is an adapter over that substrate. It:

- converts keyword schema IDs into the symbol names used by
  `std.typed.registry`;
- compiles every root and named declaration into `std.typed` surface grammar;
- registers namespaced extensions only for metaspec document policy that is
  not part of the portable core grammar, including optional and closed map
  properties, typed sets, collection refinements, and repair-oriented
  diagnostics; and
- translates final typed findings into the stable metaspec finding and repair
  envelope.

Metaspec references must remain references in the compiled graph. They must
not be recursively expanded, replaced with `:any`, or resolved by a second
metaspec catalogue walker.

`tool.metaspec.lint` and `tool.metaspec.verify` continue to own document policy,
reference evidence, checker execution, and repair selection.

## Current comparison

| Concern | Imperative linter | Datalog | miniKanren |
| --- | --- | --- | --- |
| Deterministic lint findings | Best existing oracle | Natural finite derivation | Possible but verbose |
| Explanation/provenance | Explicitly constructed | Natural rule provenance | Requires goal instrumentation |
| Recursive closure | Manual traversal | Natural fixpoint | Natural fair search |
| Reverse lookup | Separate indexes | Query-variable projection | Native bidirectional use |
| Candidate generation | Explicit repair builders | Finite derived candidates | Natural when bounded |
| Negation | Direct predicates | Stratified finite negation | Requires constructive design or finite exclusion |
| Termination | Controlled loops | Finite facts and range-restricted rules | Caller must bound generative searches |

## Promotion boundary

- Keep `tool.metaspec.lint` and `tool.metaspec.verify` authoritative for
  document-level policy while routing value conformance through
  `std.typed.schema`.
- Keep metaspec-only semantics behind namespaced `std.typed` extensions rather
  than expanding the portable core grammar prematurely.
- Use the shared `std.typed.registry` for all keyword-addressed metaspec
  declarations and recursive references.
- Use Datalog for finite validation, dependency closure, missing evidence, and
  explainable derived obligations.
- Use miniKanren for interactive forward/reverse exploration and bounded repair
  generation.
- Do not add another normative specification unless it enables a new automated
  decision that the current metaspec cannot express.
