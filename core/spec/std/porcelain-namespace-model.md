# Porcelain namespace model

Status: **approved migration specification**  
Baseline: `hara-lang/hara@69cd5b7c444b6bfd9c73965b651ae54bd091ac30`  
Date: 19 August 2026

## Purpose

Hara places implementation visibility at the namespace boundary instead of the
individual Var boundary. This makes implementation functions directly testable,
keeps supported APIs explicit, and removes hand-written forwarding definitions
from porcelain namespaces.

The model has three namespace roles and one explicit dependency acknowledgement.

## Namespace roles

### `:standard`

`:standard` is the default when `:role` is omitted.

A standard namespace:

- may define ordinary Vars, functions, macros, protocols, and types;
- may use `intern-all` and `intern-in`;
- is a supported dependency surface unless package policy says otherwise;
- must not contain top-level `defn-`, `defmacro-`, or `^:private` definitions
  after its migration entry is closed.

Example:

```hara
(ns std.codec.hex)

(defn encode ...)
(defn decode ...)
```

Implementation helpers for a standard namespace belong in an internal domain or
utility owner:

```hara
(ns std.codec.hex.util
  (:config {:role :internal}))

(defn input-bytes ...)
(defn digit ...)
```

### `:internal`

An internal namespace contains ordinary public, inspectable, directly testable
implementation Vars:

```hara
(ns std.format.table.util
  (:config {:role :internal}))

(defn column-width ...)
```

Internal means unsupported for normal external consumers. It does not mean
unresolvable, reflective, or untestable.

Rules:

- source and tests in the same project may require the namespace normally;
- a cross-project require must include `:access true`;
- package and API documentation omit internal namespaces from the supported
  surface;
- an internal namespace may itself use `intern-all` or `intern-in`, but remains
  internal;
- top-level private definitions are invalid after migration.

### `:facade`

A facade is publication-only:

```hara
(ns std.format
  (:config {:role :facade})
  (:require [std.format.common]
            [std.format.table]
            [std.format.terminal]))

(intern-all std.format.common
            std.format.table
            std.format.terminal)
```

After the `ns` declaration, a facade may contain only top-level `intern-all` and
`intern-in` forms. Comments and whitespace are naturally allowed. It may not
contain:

- `def`, `defn`, `defmacro`, protocols, types, or declarations;
- load-time initialization;
- adapter bodies;
- arbitrary `do`, `let`, or evaluation forms.

Adapters and macros move to an internal API owner and are then published.

## Internal access acknowledgement

The require grammar gains one option:

```hara
[example.codec.parse :as parse :access true]
```

`:access` accepts only the literal boolean `true`.

It means that the requiring namespace knowingly depends on an internal
namespace. It does not:

- re-export any Var;
- change the target role;
- suppress unrelated lint findings;
- promise compatibility for the internal namespace.

Same-project access does not require the option, although it may be used as
documentation. Cross-project access requires it.

Re-exporting another project's internal surface requires both:

```hara
(:require [other.project.internal :access true])
```

and an explicit publication form:

```hara
(intern-in other.project.internal/selected)
```

The package manifest records `:internal-use` and `:internal-reexport`
separately.

## Publication

`intern-all` and `intern-in` remain explicit source forms. There is no
`:export true` require option.

`intern-all` is used only for a coherent owner whose complete public surface is
intended to become part of the target API:

```hara
(intern-all std.format.table)
```

`intern-in` is used for selected or renamed publication:

```hara
(intern-in std.typed.schema/normalize-with
           [schema std.typed.schema/normalize])
```

Supported, recommended API Vars are marked at their owning definitions:

```hara
^{:public true}
(defn encode [value] ...)
```

`:public true` is a priority signal for autocomplete, documentation, and API
discovery tools. It does not alter Var visibility, namespace role, access, or
publication. An ordinary unmarked definition remains resolvable and directly
testable but is not prioritized as recommended API. Internal namespaces remain
internal regardless of the marker, and facades still publish exclusively with
`intern-all` and `intern-in`. Publication must preserve the owner Var's
`:public` metadata.

The implementation must provide:

1. deterministic symbol ordering;
2. collision preflight before target mutation;
3. source-Var metadata and macro preservation;
4. publication provenance;
5. reload reconciliation, including stale publication removal;
6. identical behaviour across the supported JVM and Rust loaders.

An `intern-all` owner must first move every public helper that should not be
exported. Changing `defn-` to `defn` inside an `intern-all` owner without such a
move is an API expansion and is prohibited.

## Deprecation rules

The following top-level forms are deprecated:

```hara
defn-
defmacro-
(def ^:private ...)
(def ^{:private true} ...)
```

The deprecation applies to both production roots:

```text
core/lib/src
core/lib/src-lang
```

It does not apply to lexical functions created by `fn`, `letfn`, macro
expansion internals, or symbols appearing only as quoted compatibility data.

### Migration classes

Every surveyed namespace has one migration class.

#### `:promote-in-place`

The namespace becomes or remains internal. Remove the private marker, retain the
symbol and behaviour, and add direct tests.

#### `:extract-symbols`

The namespace remains standard, becomes a facade owner, or is selected by
`intern-all`. Move the symbols named in
`private-symbol-migrations.edn` to their specified internal owners.

#### `:facade-conversion`

Move all implementation and load effects to internal owners. Replace forwarding
definitions with explicit `intern-all` and `intern-in` forms. Verify the public
surface before and after.

#### `:generator-or-internalise`

The source belongs to `src-lang` or another generated family. Change the owning
generator or mark the materialized namespace internal. Do not patch generated
output independently.

#### `:bootstrap-ledger`

Foundation, `lang.core`, and migration generators require JVM/Rust bootstrap
parity and deterministic regeneration. Their private forms remain baseline
exceptions only until a dedicated migration wave closes them.

#### `:owner-review`

The namespace is currently standard and has no approved cross-namespace move.
The family owner must choose either an internal role or a specific internal
destination before removing the private marker.

## Survey scope

The baseline examines **203 namespaces**:

- **184** namespaces containing at least one tracked private mechanism;
- **19** additional publication roots and compatibility boundaries;
- **183** files containing `defn-`;
- **2** files containing `defmacro-`;
- **7** files containing a top-level `^:private` Var;
- **139** approved cross-namespace symbol moves in the initial placement ledger.

Private-bearing namespaces by source family:

| Family | Namespaces |
|---|---:|
| `lang` | 54 |
| `std` | 29 |
| `postgres` (`src-lang`) | 27 |
| `db` | 25 |
| `tool` | 19 |
| `work` | 15 |
| `code` | 12 |
| `xt` (`src-lang`) | 2 |
| `workspace` | 1 |

## Symbol move contract

`private-symbol-migrations.edn` is the authoritative cross-namespace move
ledger. Every entry preserves the unqualified symbol name.

For each move:

1. Create the target internal namespace.
2. Move the complete definition and its metadata without semantic edits.
3. Update all qualified references.
4. Add or move direct tests to the target owner.
5. Keep public callers going through the supported standard/facade namespace.
6. Compare source and target behaviour.
7. Compare the supported facade's sorted public symbols and selected metadata.
8. Remove the source definition only after all references and tests use the
   target.
9. Close the ledger entry in the same changeset.

Renaming, schema alteration, argument adaptation, exception reshaping, or API
surface changes require a separate approved migration entry.

## Survey and baseline discipline

`private-definition-namespace-survey.tsv` lists every namespace examined by this
survey, including publication roots without a current private form.

The checked-in baseline is monotonic:

- a migration may remove rows or private mechanisms;
- a newly discovered historical occurrence may be added only with evidence that
  it existed at the pinned baseline;
- new private definitions after the baseline are errors;
- generated inventories must be stable under two consecutive runs.

The syntax-aware implementation should ultimately use `std.block` rather than
regular expressions. Until that is available, the audit script is a
deterministic guard and every implementation PR must inspect the affected
source forms directly.

## Lint and runtime integration

`tool.lint` adds:

```hara
:tool.lint/private-top-level-definition
:tool.lint/private-top-level-macro
:tool.lint/private-top-level-var
:tool.lint/internal-access-unacknowledged
:tool.lint/facade-definition
:tool.lint/facade-non-publication-form
:tool.lint/publication-collision
:tool.lint/intern-all-noncoherent-surface
```

The namespace declaration model must retain:

```hara
{:namespace/name example.codec
 :namespace/role :facade
 :namespace/requires
 [{:namespace example.codec.parse
   :alias parse
   :access true}]}
```

Java and Rust must parse and expose the same role/access data and produce
equivalent errors.

`tool.project` resource descriptors gain:

```hara
:resource/project
:resource/role
:resource/test?
:resource/requires
```

so cross-project access can be checked against project ownership.

## Delivery phases

### P0 — policy and pilots

- land the deprecation, survey, and move ledger;
- implement role/access parsing and warning-mode lint;
- migrate `std.format` and `workspace`;
- prohibit new private definitions relative to the baseline.

### P1 — standard library and clear facades

- migrate codecs, formatting, time, filesystem, collection, DOM, Datalog,
  `std.block`, `std.config`, `std.typed`, and `tool.lint`;
- convert clear roots to publication-only facades.

### P2 — work, database, tools, and generated libraries

- migrate `work.base`, `work.agent`, `db.postgres`, runtime-basic, code-test,
  tool and database implementation families;
- update `src-lang` generators and PostgreSQL/XTalk ownership.

### P3 — bootstrap closeout

- remove Foundation and `lang.core` baseline exceptions;
- require JVM/Rust loader parity and deterministic regeneration;
- make all top-level private-definition findings errors for Hara-owned
  production source.

## Completion criteria

The migration is complete when:

- the audit reports zero production `defn-`, `defmacro-`, and private top-level
  Vars;
- every implementation namespace is directly testable;
- every facade contains publication forms only;
- every cross-project internal access is acknowledged;
- API manifests exclude internal namespaces;
- public surfaces and metadata match the approved baselines;
- Java and Rust namespace behaviour is conformant.
