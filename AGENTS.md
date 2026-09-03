# Hara source layout

- A namespace with a single implementation file owns its implementation
  directly. Do not create a matching `.<internal>` namespace merely to place
  ordinary functions behind a facade; `postgres.application` is the reference
  shape.
- Introduce an `:internal` implementation namespace only when the feature has
  a genuine multi-namespace split, such as a curated facade over several
  independently useful implementation modules or a real boundary that must
  remain unpublished.
- A `:facade` remains publication-only. Do not use a facade/internal pair as
  ceremony for a one-file library.
