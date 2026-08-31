# Hara CLI application draft

The normative [`hara-cli.edn`](hara-cli.edn) document defines the public
command-line contract shared by the Rust `hara` runtime and the Truffle
`hara-truffle` runtime.

The reusable
[`CLI application metaspec`](../metaspec/README.md) defines the
machine-readable shape used for CLI application manifests. It is an artifact
specification conforming to the Hara artifact meta-spec. The EDN documents are
normative; this page is informative.

## Routing model

Arguments are resolved using deterministic longest-prefix matching. Canonical
routes and compatibility aliases produce the same stable `:route/id`. Handler
references are qualified keywords resolved by a closed runtime registry: a CLI
document cannot name or execute a Var, Java class, Rust function, project
namespace, or package entrypoint.

The public applications are:

- language evaluation and REPL operation;
- sessions and remote connections;
- project management;
- deterministic packages;
- specification linting and verification;
- extension package tooling.

Project operations use the canonical `hara project ...` hierarchy. The
historical flat verbs remain compatibility aliases for the draft contract.
Runtime diagnostics and benchmarks are deliberately outside the public parity
contract.

## Outcomes

Finite commands return ordinary data before it is rendered:

| Outcome | Exit |
| --- | ---: |
| success | 0 |
| completed failure | 1 |
| usage, read, resolution, unavailable or internal tool error | 2 |
| interrupted | 130 |

Required unknown or blocked obligations are completed failures and therefore
cannot report overall success.

## Conformance

The `conformance/` directory contains stable data-driven cases for routing,
option parsing and exit outcomes. Each native runtime consumes the same cases.
