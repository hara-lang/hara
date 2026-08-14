# Truffle code.vm production corpus

Issue: `hara-lang/hara#406`

This slice runs the checked-in `core/rust/assets/code-vm-conformance.edn`
corpus through the production Truffle evaluator and its existing
`EvaluationJournal` instrumentation.

## Runtime boundary

The Truffle runner owns the interpreter leg only. HALC encoding and bytecode
execution remain the Rust production implementations and are reported as
explicitly unsupported within the Truffle-local document rather than being
simulated or replaced.

Every interpreter-required case uses a fresh Graal polyglot `Context`, so
namespace definitions and mutable runtime state do not leak between fixtures.
Compile-only bytecode cases remain explicit `unsupported` observations.

## Evidence

The runner emits the shared
`hal.code-vm-conformance-runtime/0-alpha` report schema with:

- canonical corpus, fixture, namespace, resource, and source identifiers;
- returned values or normalized runtime error categories;
- bounded production Evaluation Journal events;
- explicit truncation that does not stop evaluation;
- contiguous sequence checks;
- deterministic teaching annotations derived from journal events;
- explicit no-fallback HALC and bytecode stage classifications;
- a terminal-neutral browser view.

A dedicated CI workflow generates the Rust and Truffle reports from the same
corpus and compares interpreter outcomes and source identities.

## Commands

```sh
mvn -B -Ptruffle -Dtest=CodeVmConformanceTest \
  test --file core/java/pom.xml

mvn -q -Ptruffle \
  -Dexec.mainClass=hara.truffle.CodeVmConformance \
  -Dexec.args=check \
  exec:java --file core/java/pom.xml
```

For a JSON report, use `-Dexec.args=report`. Set
`-Dhara.codeVmReport=/path/report.json` to write the document directly without
mixing it with Maven output.

This work does not claim instruction-like interpreter stepping. The journal
currently records authoritative root/function operations; extracting the live
semantic boundary for lexical bindings, special forms, continuation frames,
and namespace mutation remains issue #403.
