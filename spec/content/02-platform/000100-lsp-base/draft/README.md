# Hara LSP base profile

Status: **draft**
Profile version: **0.1.0**
Wire protocol: **Language Server Protocol 3.18**

The authoritative document is
[`hara-lsp-base.edn`](hara-lsp-base.edn). This README is its human-readable
companion and does not add requirements beyond that EDN document.

## Purpose

This profile defines what a portable Hara language server exposes to editors.
It fixes the protocol boundary, source and workspace model, common analysis
facts, namespace resolution, required IDE features, safety rules, and
conformance evidence.

The base profile deliberately keeps runtime evaluation and REPL session
management outside LSP. Those facilities can use a separate runtime protocol
without making ordinary editor analysis execute workspace code.

## Required feature set

A conforming base server provides:

- incremental document synchronization and versioned unsaved overlays;
- parse, resolution, binding, arity, and available schema/type diagnostics;
- scope-aware and namespace-aware completion plus signature help;
- hover, definition, references, document symbols, and workspace symbols;
- prepare-rename and identity-safe workspace rename;
- full-document semantic tokens;
- safe diagnostic code actions.

Document highlighting, folding, selection ranges, semantic-token deltas, and
lazy code-action resolution are recommended. Formatting is conditional: a
server must not advertise it unless the syntax representation preserves
comments, metadata, reader forms, and semantics.

## Analysis architecture

All features consume the same immutable, versioned analysis snapshot. The
minimum fact model covers forms and spans, lexical scopes, namespaces and
aliases, definitions and references, resolved identities, documentation,
signatures, schemas or types, and diagnostics.

`std.typed` is the preferred portable source of schema inference and checking
facts. The base syntactic and namespace features still work before that layer
is available. `std.logic.kanren` may provide relational queries over normalized facts,
especially for cross-file features, but basic lookup and resolution cannot
depend on a logic engine.

## Namespace correctness

Qualified completion is resolved before candidates are collected. Given:

```clojure
(ns app.core
  (:require [std.foundation.promise :as promise]))

(promise/)
```

completion offers visible members such as `promise/run`, `promise/new`, and
`promise/from`. It must not attach `Coroutine`, native types, or unrelated
foundation/global symbols to the `promise/` qualifier merely because those
names exist in the runtime.

An unresolved qualifier such as `co/` does not fall back to a global inventory.
It may instead produce no candidates, a missing-alias diagnostic, and a safe
action to add a require when there is a unique intended namespace.

`std.protocol` and all `std.protocol.*` namespaces are ordinary indexed Hara
namespaces. Their protocol definitions and members participate in completion,
hover, definition, references, symbols, semantic tokens, signature help, and
rename with stable symbol identities.

## Safety boundary

Parsing and analysis do not evaluate project top-level forms, invoke ambient
host capabilities, install packages, or run untrusted analyzer extensions.
Executable plugins, macro expanders, and lint hooks require explicit trusted
workspace configuration.

## Conformance

[`conformance/lsp-base.edn`](conformance/lsp-base.edn) supplies portable
scenarios for lifecycle, incremental overlays, position conversion,
namespace-isolated completion, protocol indexing, malformed source recovery,
resolved rename, and the no-evaluation safety boundary.

The external normative wire reference is the
[Language Server Protocol 3.18 specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/).
