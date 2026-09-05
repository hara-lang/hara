# `docs.markdown`

`docs.markdown` is Hara's pure `hara:markdown@0.1.0` Component. Its only
public operation is `markdown.render(source) -> html`.

The Component imports no WASI or Hara host interface. Its guest uses
[Comrak 0.54.0](https://github.com/kivikakk/comrak) with the renderer option
that preserves raw HTML, as CommonMark requires. The checked-in official
CommonMark 0.31.2 corpus at `test/commonmark-0.31.2.json` passes all 652
rendering examples. Rendering is not sanitisation; callers that display
untrusted Markdown must sanitise the returned HTML for their own context.

`Cargo.lock` pins the Component build. Comrak is distributed under the BSD
2-Clause license; its notice is included in `LICENSE.comrak`.

Build with the standard Component toolchain:

```sh
cargo component build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/hara_docs_markdown.wasm markdown.component.wasm
```

After changing `wit/markdown.wit`, update its SHA-256 in `project.edn` and
rebuild the checked-in `markdown.component.wasm`.
