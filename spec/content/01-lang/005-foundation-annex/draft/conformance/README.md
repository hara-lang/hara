# Foundation library conformance

This directory is the only source of truth for cross-runtime conformance of
Foundation-origin libraries.

`libraries/index.edn` is the registry manifest. It points to the reusable
historical review import at `reviews/index.edn` and to each full-path family
publication.

The review import preserves earlier Clojure-core and external-dependency review
evidence in the current registry shape. Historical classifications remain
candidates until reconciled with the active family observation, current Hara
ownership and specifications, and applicable runtime execution.

Each full-path family has three documents beneath the family directory:

- `origin.edn` records the pinned Foundation source namespace and original test
  paths, `clojure.core` references, and external `clojure.*` dependency
  observations with complete extraction accounting. It is provenance, not a
  Hara expectation.
- `conformance.edn` is the reviewed Hara contract. It names the Hara target
  namespaces, reviewed core-symbol and external-dependency dispositions, and
  the Hara test files that every runtime adapter must execute.
- `metadata.edn` records the exact source pins, source/test pairs, imported
  review identities, and generated artifact digests used to detect publication
  drift.

Family directories use the complete path identifier (`std_lib_zip`,
`std_block`, `std_lib_project`, and so on). Runtime adapters must consume the
manifest and contract paths; per-namespace probes and hand-maintained runtime
lists are not part of this protocol.
