# Foundation-derived conformance source corpora

Status: scaffold for [#1019](https://github.com/hara-lang/hara/issues/1019).

This document defines the first, corpus-only boundary for turning pinned
Foundation tests into reviewable `conformance.edn` source evidence. It does not
declare Foundation behavior to be the Hara contract and it does not authorize
implementation changes.

## Ownership

```text
code.migrate.conformance
  document shape
  family and fixture validation
  deterministic ordering and rendering
  provenance and accounting laws

code.migrate
  source-analysis and extraction policy

code.manage
  host orchestration, file writes, and CI tasks

Hara specifications and implementation
  reviewed Hara expectations and final semantic ownership
```

The Foundation repository remains immutable source evidence pinned by
`code.migrate.profile/+foundation-baa75a+`:

```text
repository  zcaudate-xyz/foundation-base
revision    baa75aabd6a879753d7d5cb07271b1448271e7cb
tree        26d494f60c4970df56eba8ac40f92affeee4e159
```

## Source document

The initial document has type
`:code-migrate-conformance-source`, version `1`, and status
`:unreviewed-source-evidence`.

Each family records:

- a stable family identifier and extraction wave;
- exact test paths or path-prefix selectors;
- provisional Foundation-to-Hara ownership mappings;
- notes that constrain extraction;
- accounting totals for discovered, emitted, skipped, and manual-review tests.

Every family must satisfy:

```text
discovered = emitted + skipped + manual-review
```

The committed baseline is
`foundation-conformance-families.edn`. Zero totals mean that the selector has
not yet been scanned; they do not mean that the Foundation family has no tests.

## Fixture boundary

An emitted source fixture has this minimum shape:

```clojure
{:id "std/lib/zip/left-element-000"
 :family/id :std-lib-zip
 :ns std.lib.zip
 :expr "(left-element ...)"
 :foundation/expected 3
 :foundation/source
 {:path "test/std/lib/zip_test.clj"
  :namespace "std.lib.zip-test"
  :form-id "left-element-000"
  :line 42}}
```

`:foundation/expected` preserves the result asserted by the pinned source. It
must never be overwritten merely because Hara later chooses a different
expectation.

The generic keys `:expected` and `:contract` are rejected during source
extraction because they incorrectly imply that the Foundation result is
already normative for Hara.

## Determinism and drift

The validator requires:

1. exact profile, repository, revision, and tree identity;
2. deterministic family ordering by wave and family identifier;
3. deterministic fixture ordering by fixture identifier;
4. duplicate-free family and fixture identifiers;
5. complete source provenance for every emitted fixture;
6. a known family for every fixture;
7. family and document accounting that matches emitted fixture counts;
8. canonical bytes from `code.migrate.conformance/conformance-source`.

The focused HAL test loads the committed family ledger, validates it, renders
it again, and requires byte equality. The Foundation migration workflow runs
that test before the complete library and Rust runtime suites.

## Initial delivery waves

| Wave | Family | First responsibility |
| --- | --- | --- |
| 0 | `code.query.*` | Recreate the prototype corpus without its bespoke harness |
| 0 | `std.lib.zip` | Recreate the 76-fixture prototype from pinned source |
| 1 | context | Inventory historical `std.lib.context*` and current Hara owners |
| 2 | block | Account for `std.block*` and related protocol evidence |
| 3 | testing | Separate pure `code.test` behavior from runtime and Work concerns |
| 4 | `lang.common.*` | Extract evidence without deciding semantic ownership for #781 |

The selectors and provisional mappings are discovery inputs. Their zero
accounting totals must be replaced by exact emitted, skipped, and manual-review
records as each wave is implemented.

## Corpus-only pull request law

A corpus-generation pull request may add or update:

- native Hara extraction and validation code;
- generated `conformance.edn` source documents;
- provenance and accounting reports;
- generator tests and drift checks;
- declarative family mappings.

It must not include:

- changes to the measured library implementation;
- changes made only to force Foundation parity;
- a per-family runtime comparison harness;
- a registry `:conformant` result;
- a reviewed Hara expectation presented as if it came from Foundation.

Runtime comparison, independently justified implementation fixes, and registry
publication follow only after the source corpus has been reviewed against the
current Hara specifications and ownership model.
