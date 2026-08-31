# Hara code migration meta-specification

Status: **draft**

[`code-migration-metaspec.edn`](code-migration-metaspec.edn) defines the
machine-readable contract for executable Clojure/Foundation-to-Hara migration
catalogs. A namespace is promotable only when its source and focused test are
regenerated without manual fixups and pass in fresh native Hara processes.
