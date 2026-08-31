# HAL Artifact DSL Meta-Specification

Status: draft, version 0.1.0.

The artifact metaspec conforms to the self-describing normative
[`metaspec-metaspec.edn`](../../../01-lang/000-metaspec/draft/metaspec-metaspec.edn).
The normative artifact document is
[`artifact-metaspec.edn`](artifact-metaspec.edn). This Markdown file is an
informative guide.

An artifact specification is a sibling of the HAL language specification. It
does not change HAL syntax. It declares an artifact kind, its five specification
surfaces (`forms`, `entities`, `relations`, `codecs`, and `checkers`), semantic
laws, and data-driven conformance resources.

Every schema, requirement, law, relation, checker and conformance case has a
stable qualified identifier. Extensions may add keys, but extension keys must
be qualified. References between declarations are validated by identifier.

The common obligation statuses are `pass`, `fail`, `unknown`, `blocked`,
`waived`, and `not-applicable`. In particular, `unknown` is information rather
than success and must remain visible to downstream policy.

Artifact DSL implementations follow a data boundary:

```text
source → HAL forms with spans → surface model → canonical model
       → canonical EDN → obligations and findings
```

Canonical models contain only portable HAL/EDN values. Formatting, comments,
whitespace, runtime values and host handles are outside semantic equality.

## AI generation loop

An agent starts with a machine-readable generation request, emits a draft EDN
meta-spec, and runs the linter. Each finding carries a stable rule and
requirement ID, a data path, and a structured repair action. After lint reaches
closure, verification resolves all schema and declared cross-references. Only a
report with no failed, unknown, or blocked obligations is accepted as a
generated meta-spec.

The offline bootstrap workflow uses the portable HAL checker:

```clojure
(require [hara.metaspec.core :as metaspec])

(metaspec/conforms generated-metaspec
                  {:metaspec artifact-metaspec})
```

`conforms` checks document shape, qualified keys, stable IDs, uniqueness,
schema references, declared cross-references, and checker obligations. The
result is a structured report whose status is `:pass`, `:fail`, or `:blocked`.
Installed extension checkers are resolved only by exact package coordinate and
qualified HAL entrypoint.
