# Hara CLI application metaspec

The normative
[`cli-app-metaspec.edn`](cli-app-metaspec.edn) document defines the
machine-readable shape of a Hara CLI application specification.

It defines CLI applications in terms of stable route, option, handler, outcome,
request, and result entities. Routes contain data-only references to handlers;
executable functions and host implementation names are outside the document
model. A runtime resolves qualified handler IDs through a closed registry.

The metaspec also defines the laws and checkers used to validate deterministic
longest-prefix routing, alias equivalence, cross-reference integrity, structured
outcomes, and canonical EDN generation.

The metaspec is reusable. The particular public Hara command-line application
that conforms to it is defined by
[`../draft/hara-cli.edn`](../draft/hara-cli.edn).
