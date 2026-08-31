# Hara Foundation bootstrap namespace

Status: **draft**

The authoritative document is
[`foundation-spec.edn`](foundation-spec.edn). It conforms to the
[`Hara Foundation meta-spec`](../metaspec/README.md).

This layer owns the root `std.foundation` namespace:

- automatic referral and blank namespaces;
- `:builtins`, `:rename`, and global namespace configuration;
- protocol and native-type aliases;
- portable root functions and macros;
- narrow evaluator-backed primitives;
- public metadata and return-value boundaries.

It specifies wrappers, not the facilities they wrap. Protocol dispatch belongs
to `002-protocol`, native method behavior belongs to `003-native`, and
qualified libraries belong to `005-foundation-annex`.

The explicitly required `std.sandbox` library is specified here as a thin
portable facade over the capability-gated native Sandbox descriptor. It is not
root-referred and adds no aliases or lifecycle duplicates to `std.foundation`;
namespace inspection uses the canonical `ns-*` names (`ns-current`, `ns-list`,
`ns-info`, `ns-vars`, `ns-find`, `ns-create`, `ns-name`, `ns-publics`, and
`ns-aliases`); only `env-snapshot` and `env-module` remain as environment
wrappers.

The surface catalog is a draft baseline. Candidate promotion requires an exact
fresh-context `ns-publics` comparison for JVM, Rust, and Wasm.

## Collection architecture locked by #666

`reduce-in` is a public portable HAL composition over `IReduce`,
`IToMutable`, and `IToPersistent`; it is not a `Base` method. `mapv` is portable
HAL with an explicit persistent-vector result, and `some` returns the first
truthy mapped value. Source-sensitive operations use the first source to select
result family and eager/lazy mode, with the explicit partition, split, sort,
and reduction lifecycle exceptions recorded in `foundation-spec.edn`. Test
configuration and Result inspection use `Test/*` directly; only the delayed
`test-check` assertion macro remains in Foundation. These rules are normative
for runtime and native-HAL conformance work.

## Spec-driven conformance

The authoritative effective-Var ledger is
`conformance/foundation-surface.edn`; its count is derived using
declaration-once semantics. The executable behavior and diagnostic authority is
`conformance/fixtures/foundation_behavioral.hal`. It classifies every Var as
portable, capability-specific with a named capability, or inventory-only with a
reviewable reason, and derives portable, capability-specific, inventory-only,
passed, failed, and skipped counts.

Runtime tests are adapters: they load the canonical Foundation modules, execute
this corpus, and compare their live surface with the ledger. Calibration probes
and migrated host assertions belong in the corpus so successful and failing
observations remain reviewable instead of living only in terminal transcripts.
The executable entrypoints, negative drift controls, adapter commands, and
derived-count rules are retained in `conformance/validation-matrix.edn`; a new
diagnostic check is incomplete until its source and normalized expectation are
recorded there or in `foundation-calibration-snippets`.
