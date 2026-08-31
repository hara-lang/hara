# Hara archive package format

The authoritative document is [`hara-harp.edn`](hara-harp.edn). A `.harp` is a deterministic safe archive generated from `project.edn` and a resolved lock. Its root `package.edn` is generated from the exact archive entries and is never authored separately.
