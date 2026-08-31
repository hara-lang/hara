# Tahto compiler pipeline

Status: **draft**

The authoritative document is
[`tahto-compiler-spec.edn`](tahto-compiler-spec.edn).

This layer owns the boundary between a materialized Tahto grammar selection,
entry preparation, pure form compilation, diagnostics, provenance, and explicit
publication. Grammar source storage and resolution remain owned by the registry
protocols. Target emitters and language models implement the phase contracts.

The provenance result is an ordered, portable worklog recording the selected
grammar and the transformations performed by each compiler phase.
