# Native Hara XTalk reference

## Source matrix

- `core/lib/src/lang`: canonical `lang.*` compiler and emitter libraries.
- `core/lib/src-lang/xt`: portable XTalk libraries and language-facing source.
- `core/lib/test-lang/xt`: canonical XTalk tests.
- `core/lib/test-lang/xtbench`: generated target benches; do not edit directly.

## Validation sequence

```sh
./core/hara --project core --offline stdin < proposed-file.hal
./core/hara --project core --offline run core/lib/src-lang/xt/path/file.hal
./core/hara --project core --offline project test core/lib/test-lang/xt/path/file_test.hal
```

Use the project’s Hara seed-generation entrypoint for target benches. Inspect
the source metadata before choosing compatible, incomplete, generation, or
target-test operations; command shapes may differ by language package.

For emitter changes, capture the normalized/staged form and emitted text for a
small representative input before and after the edit. Compare generated output
with its source and ensure a second generation produces no diff.
