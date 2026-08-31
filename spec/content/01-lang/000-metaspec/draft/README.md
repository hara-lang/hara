# Hara metaspec

Status: **draft**
Version: **0.1.0**

The authoritative document is [`metaspec-metaspec.edn`](metaspec-metaspec.edn). It is the
self-describing contract for authoring, linting, verifying, and repairing Hara
metaspec documents.

This directory defines metaspecs themselves. It does not define the shape of a
language specification or an artifact specification; those specialized
metaspecs live beside the document families they govern.

The executable cases in
[`conformance/metaspec.edn`](conformance/metaspec.edn) verify self-description,
required sections, qualified extension keys, identifier uniqueness, reference
resolution, and structured repair actions.

## Hara checker

The portable checker is `hara.metaspec.core/conforms`. Bootstrap validation
passes this document as both the document and its metaspec:

```clojure
(require [hara.metaspec.core :as metaspec])

(metaspec/conforms hara-metaspec
                  {:metaspec hara-metaspec})
```

The result is a structured `:hara/metaspec-conformance` report. It passes only
when every schema, reference, and declared checker obligation passes. Missing
metaspecs and unavailable exact-version checker packages produce
`:report/status :blocked`; they are never silently accepted.
