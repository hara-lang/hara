# HAL language metaspec

Status: **draft**
Version: **alpha**

The authoritative document is
[`language-metaspec.edn`](language-metaspec.edn). It defines the shape of HAL
language specifications.

This README is an informative companion and must not introduce requirements
that are absent from those EDN documents.

## Purpose

The language metaspec standardises:

- document identity, version, and lifecycle status;
- ordered normative sections and stable requirement identifiers;
- special-form, macro, function, and reader-form declarations;
- conformance, parity, implementation, and historical references;
- validation, rendering, and promotion requirements.

## Authority model

The language-spec EDN document is normative. Its rendered README is
informative. Conformance corpora provide executable evidence for named
requirements, while implementation profiles describe backend constraints.
Files below `99-archive/planning/` are historical input and cannot be normative
dependencies of an active specification.

## Required language-spec structure

A conforming document declares:

1. identity, type, version, status, title, and summary;
2. the meta-spec document and version it conforms to;
3. scope and portable invariants;
4. an explicit section order and the corresponding sections;
5. its language forms;
6. typed references and conformance suites;
7. coverage and provenance where applicable.

Normative requirements carry stable qualified identifiers and one of
`:must`, `:must-not`, `:should`, `:should-not`, or `:may`.

## Validation

Validation checks identifier uniqueness, cross-reference integrity, section
ordering, requirement evidence, authority boundaries, and repository-local
paths. Unknown extension keys must be qualified.

The checker is implemented in HAL:

```clojure
(require [hara.metaspec.core :as metaspec])

(metaspec/conforms language-spec
                  {:metaspec language-metaspec})
```

Alternatively, register the exact metaspec coordinate once and call
`(metaspec/conforms language-spec)`. Reports use stable finding IDs, data paths,
and machine-readable repair actions. A report cannot pass with failed,
unknown, or blocked obligations.

## Rendering

A conforming renderer writes a companion `README.md` in document order,
includes requirement identifiers, preserves relative links, and labels the
EDN source as authoritative. A stale rendering is a validation failure.

## Promotion

A draft can become a candidate after structural validation, evidence coverage,
current rendering, and active-runtime reporting. A candidate can become stable
only after required conformance and parity pass and semantic ambiguities are
resolved.
