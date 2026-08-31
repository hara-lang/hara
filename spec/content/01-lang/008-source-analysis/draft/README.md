# Hara source analysis

This draft defines the portable, non-evaluating analysis contract used by
`tool.lint`. The checked-in runtime profile is pinned to version
`alpha`; changing the rules or symbol catalog requires changing the
specification version and regenerating the profile.

The analyzer consumes recovering `std.block` trees, project structure from
`tool.project`, and portable schema relationships and inference from
`std.typed`. `source-analysis.edn` defines analyzer behaviour and finding
requirements; `std-typed.edn` defines the canonical schema grammar,
normalization, assignment, declaration registry, project index, precedence,
modes, and cross-runtime parity contract.

`defstruct` and `defmutable` declarations use their field vector as the sole
source of type metadata. Legacy symbol fields are `:any`; typed fields may
carry properties. The type and generated constructors are published from one
normalized `:struct` descriptor so linting and completion do not need a second
schema annotation.

Schema metadata is data only. Analysis does not evaluate project code,
instrument runtime calls, expand macros, or load dependency artifacts.
