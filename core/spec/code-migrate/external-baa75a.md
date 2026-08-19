# Foundation external dependency review: `foundation-baa75a`

- Pinned revision: `baa75aabd6a879753d7d5cb07271b1448271e7cb`
- Exact route occurrences: 649
- Unique external names: 83
- Reviewed route occurrences: 649
- Pending route occurrences: 0

## Dispositions

- `host-runtime-adapter`: 71
- `manual-boundary`: 15
- `missing`: 28
- `obsolete`: 128
- `portable-substitute`: 266
- `semantic-replacement`: 141

## Reusable rule admission

Only `clojure.string` is admitted as a portable substitute. Its exact Hara source and paired test blobs are recorded in the EDN evidence. `triml`, `trimr`, and `trim-newline` use explicit symbol mappings.

Reader types, JVM classes/interfaces, project integrations, and publication symbols remain diagnostic-only. No pending candidate is counted as reviewed.
