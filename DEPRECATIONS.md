# Deprecations

## Top-level private definitions (`defn-`, `defmacro-`, and `^:private`)

**Status:** deprecated for Hara production source as of **19 August 2026**.

Hara is adopting namespace-level API boundaries. New production code must not add
top-level private functions, macros, or Vars. Existing private definitions remain
loadable only to support the tracked migration and external source compatibility.

The deprecated forms are:

```hara
(defn- helper ...)
(defmacro- helper ...)
(def ^:private helper ...)
(def ^{:private true} helper ...)
```

Lexically local implementation remains supported and is not deprecated:

```hara
(letfn [(helper [value] ...)]
  ...)

(let [helper (fn [value] ...)]
  ...)
```

### Replacement

Implementation Vars are ordinary public Vars in a namespace declared as
`:internal`:

```hara
(ns example.codec.parse
  (:config {:role :internal}))

(defn scan-token ...)
(defn parse ...)
```

A supported namespace publishes the intended API explicitly:

```hara
(ns example.codec
  (:config {:role :facade})
  (:require [example.codec.model]
            [example.codec.parse]))

(intern-all example.codec.model)
(intern-in example.codec.parse/parse)
```

`std.foundation` does not need to be required or aliased for `intern-all` or
`intern-in`.

Intentional cross-project use of an internal namespace is acknowledged at the
dependency edge:

```hara
(ns application.experiment
  (:require [example.codec.parse :as parse :access true]))
```

`:access true` does not publish the target namespace and does not make it
standard. It records that the caller knowingly depends on an implementation
surface.

### Enforcement

This is a firm source-policy deprecation:

1. New occurrences in production `.hal` source are prohibited.
2. The checked-in survey is the migration baseline; it may only shrink.
3. `tool.lint` will report:
   - `:tool.lint/private-top-level-definition`
   - `:tool.lint/private-top-level-macro`
   - `:tool.lint/private-top-level-var`
4. During the migration, existing baseline entries may be warnings while new
   entries are errors.
5. When the baseline reaches zero, all top-level private definitions are errors
   in Hara-owned production source.
6. Parser and runtime support may remain for external source compatibility.
   Continued parsing does not make the construct acceptable in Hara itself.

This deprecation does not authorize accidental API expansion. A private symbol
must be handled according to its owner:

- In a namespace becoming `:internal`, remove the private marker and test the
  symbol directly.
- In a `:standard` namespace, move implementation-only symbols to an internal
  domain or `.util` owner.
- Before an owner is selected with `intern-all`, move every symbol that should
  not become part of the supported facade.
- A `:facade` contains no implementation definitions; after `ns`, it contains
  only top-level `intern-all` and `intern-in` forms.

### Governing documents

- [`core/spec/std/porcelain-namespace-model.md`](core/spec/std/porcelain-namespace-model.md)
- [`core/spec/std/private-definition-namespace-survey.tsv`](core/spec/std/private-definition-namespace-survey.tsv)
- [`core/spec/std/private-symbol-migrations.edn`](core/spec/std/private-symbol-migrations.edn)

The survey is pinned to commit
`69cd5b7c444b6bfd9c73965b651ae54bd091ac30`. Regenerate it before each migration
wave and require zero unrecorded additions.
