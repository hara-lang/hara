# Upstream provenance

`markdown-wasm` 1.2.0 (revision `0d99d1151ff4d929a8ac8f3a191bfec54a10a869`)
was evaluated as the initial guest implementation. Its upstream MD4C renderer
does not match the CommonMark 0.31.2 corpus: its default is GitHub-flavoured
Markdown and its HTML serialisation diverges from the reference examples.

The published Component therefore uses
[`Comrak` 0.54.0](https://github.com/kivikakk/comrak), pinned in `Cargo.lock`.
The guest enables Comrak's raw-HTML renderer option because CommonMark's own
examples require raw HTML to be retained. The standard 0.31.2 corpus fixture
passes all 652 examples through the built Component.

This preserves the standard, filesystem-free WIT contract instead of exposing
markdown-wasm's non-conforming output. Comrak is BSD-2-Clause; its upstream
notice is included as `LICENSE.comrak`.
