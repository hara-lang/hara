# Hara code.query conformance

Status: **draft**

This section stores the canonical `code.query` Foundation parity corpus and the
latest run results.

## Artifacts

- [`conformance/code-query-conformance.edn`](conformance/code-query-conformance.edn) —
  the shared fixture corpus used by both the Hara harness
  (`core/lib/src/code/query/conformance.hal`) and the Foundation Clojure runner
  (`scripts/code_query_conformance.clj`). Each fixture is an expression evaluated
  against a `code.query` namespace with an expected EDN value.
- [`conformance/code-query-conformance-results.edn`](conformance/code-query-conformance-results.edn) —
  the most recent Foundation parity run.

## Run procedure

From the `technology/hara` workspace on `agent/code-query-conformance`:

```shell
hara --project core --offline test code.query.conformance-test
```

The focused test loads the corpus, evaluates every fixture in Hara, invokes the
Foundation runner, and asserts `:conformant? true`.
