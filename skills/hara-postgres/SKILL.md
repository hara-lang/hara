---
name: hara-postgres
description: Develop or review native Hara PostgreSQL DSL source, including `.hal` forms using defn.pg, deftype.pg, defsel.pg, defret.pg, pg/t:*, pg/g:*, pg/q:*, typed tables, graphs, queries, and source-owned SQL or RPC generation. Use for PostgreSQL code under Hara src-lang and test-lang trees.
---

# Hara PostgreSQL DSL

Use `$hara-development` for the native candidate-evaluate-test-write-run-test
cycle. Add these domain checks inside it.

## Required checks

1. Read the typed table/query definition and generator that own the operation.
2. Evaluate the complete `.hal` candidate with `hara --offline stdin`.
3. Inspect the typed analysis, emitted SQL, and return shape when the change
   affects generation.
4. Run the narrowest test under `core/lib/test-lang/postgres`.
5. After writing, run the changed source and repeat the focused test.

## DSL conventions

- Use `i-*` for typed inputs, `v-*` for derived values, `o-*` for operation or
  result bindings, `m` for the conventional input map, and `_` for ignored
  values where the surrounding source follows that convention.
- Keep procedural operations ordered and explicit. Bind database results before
  returning them when the generator expects a result shape.
- Treat `pg/t:*`, `pg/g:*`, and `pg/q:*` as typed Hara operations. Confirm
  columns, references, coercions, defaults, `:into`, and `:returning` against
  their definitions rather than assuming native PL/pgSQL behavior.
- Edit the owning `.hal` source and regenerate SQL or RPC artifacts; never
  patch generated artifacts directly.

Read [references/hara-postgres-dsl.md](references/hara-postgres-dsl.md) for
source locations and focused validation examples.
