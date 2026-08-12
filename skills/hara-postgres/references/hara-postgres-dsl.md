# Native Hara PostgreSQL reference

Canonical source is under `core/lib/src-lang/postgres`; focused tests are under
`core/lib/test-lang/postgres`.

## Validation sequence

```sh
./core/hara --project core --offline stdin < proposed-file.hal
./core/hara --project core --offline run core/lib/src-lang/postgres/path/file.hal
./core/hara --project core --offline project test core/lib/test-lang/postgres/path/file_test.hal
```

Use stdin only for an already assembled candidate kept outside the tracked
source path. The source gate supplies the candidate directly without a shell
redirect.

## Review questions

- Which `deftype.pg`, table, graph, or query owns every referenced field?
- Do input coercions, defaults, refs, and nullability match the typed source?
- Does `:returning` match the operation and caller shape?
- Is each operation result explicitly bound when the generator requires it?
- Is generated SQL/RPC output being changed through its source owner?

Object shorthand such as `#{o-name o-age}` is a Hara DSL construct. Confirm its
lowering in the generator rather than treating it as database-native syntax.
