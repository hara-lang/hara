# `std.foundation/intern-all` actionable migration set

Audit baseline: `69cd5b7c444b6bfd9c73965b651ae54bd091ac30` (`main`, 2026-08-19).

This file is the condensed implementation list from the repository-wide façade audit. The full reasoning and exclusions are in [`intern-all-audit.md`](./intern-all-audit.md). Portable script publication is covered separately in [`intern-all-script-focus.md`](./intern-all-script-focus.md).

## Governing rule

Use `std.foundation/intern-all` only when a namespace deliberately republishes **every public Var** from a source namespace under the same public names.

Use `std.foundation/intern-in` when publication is selective or renamed. Keep an explicit `defn` or `defmacro` when it changes arguments, return values, exceptions, scheduling, capabilities, metadata, macro expansion, or compatibility behaviour.

`intern-all` is therefore a namespace-surface operation, not a generic replacement for every forwarding function.

## Apply now

### 1. `core/lib/src/std/format.hal`

`std.format` republishes the complete public surfaces of:

- `std.format.common`: `line`, `value-text`, `truncate`, `pad`
- `std.format.table`: `table-lines`
- `std.format.terminal`: `render-lines`, `render`, `emit-lines!`

Those aliases should be replaced by one bulk publication:

```clojure
(ns std.format
  (:require [std.foundation :as f]
            [std.format.common]
            [std.format.table]
            [std.format.report :as report-format]
            [std.format.terminal]))

(f/intern-all std.format.common
              std.format.table
              std.format.terminal)

(f/intern-in report-format/report-lines)

(defn report
  "Renders a report document as plain text by default or ANSI when enabled."
  ([document]
   (report document {}))
  ([document options]
   (terminal/render
    (report-format/report-lines document options)
    options)))
```

Keep the local `report` function. It adapts `std.format.report/report-lines` through the terminal renderer and is not a Var publication alias. Publish only `report-lines` from `std.format.report`; unrestricted `intern-all` would also import that namespace's plain-text `report` and conflict with the façade adapter.

Expected removed aliases: **8**.

### 2. `core/lib/src/workspace.hal`

`workspace` republishes the complete public surface of `workspace.core`:

- `create`
- `dispatch`
- `view`
- `result`

Replace those four aliases with:

```clojure
(ns workspace
  (:require [std.foundation :as f]
            [workspace.core]
            [workspace.model :as model]))

(f/intern-all workspace.core)

(f/intern-in model/area
             model/component
             model/component-view
             model/find-area
             model/component-contract)
```

Do **not** use `intern-all` for `workspace.model`. The façade intentionally omits public model helpers and constants including `normalize`, `default-area-id`, `workspace-type`, and `workspace-version`.

Expected removed aliases: **9**, replaced by one `intern-all` and one `intern-in` form.

## Already correct

### `core/lib/src-lang/postgres/core.hal`

This namespace already uses:

```clojure
(f/intern-all postgres.core.builtin
              postgres.core.addon)
```

and uses `f/intern-in` for its selected implementation, graph, view, binding, and application entrypoints. Preserve this as the reference pattern: whole owner namespaces use `intern-all`; curated cross-owner publication uses `intern-in`.

### `core/lib/src/code/migrate/script.hal`

The script migration ledger already includes `std.foundation/intern-all` in the target reference inventory. Publication changes for the `tahto.core.script*` family must be made through that ledger and the grammar-derived generators, not by editing generated macro families independently.

## Convert to `intern-in`, not `intern-all`

These files contain direct same-name aliases, but their source namespaces expose additional public Vars or the façade intentionally selects only part of the source surface:

| Target | Selected owners | Required treatment |
|---|---|---|
| `core/lib/src/std/typed.hal` | `std.typed.schema`, `std.typed.registry`, `std.typed.explain`, `std.typed.infer` | Replace alias blocks with explicit `intern-in` groups. Do not bulk-import constants and lower-level registry/inference helpers. |
| `core/lib/src/std/block.hal` | `std.block.base`, `construct`, `parse`, `type`, `value`, `layout` | Use grouped `intern-in`; retain renames such as `type <- block-type`, `string <- block-string`, and `layout <- layout-main`. |
| `core/lib/src/tool/lint.hal` | `tool.lint.analyze`, `flow`, `report` | Only same-name pure aliases are eligible for `intern-in`. Keep `lint-source`, `lint-file`, `lint-scans`, and `lint-project`: they now return native `Result` values. |
| `core/lib/src/code/test.hal` | test checker/runtime/work/report owners | Use `intern-in` for stable aliases only. Keep macro forwarding wrappers and comparison renames explicit. |
| `core/lib/src/work/base/runtime.hal` | model, memory, frame, coordinator, receipt | Use `intern-in` for selected compatibility exports; do not expose complete internal runtime owners. |
| `core/lib/src/work/base.hal` | `work.base.runtime` | Use `intern-in` for the selected runtime façade block. The namespace also owns the Work algebra and managed host API. |
| `core/lib/src/lang/runtime/basic.hal` | basic, oneshot, verify owners | Use `intern-in`; each owner has additional public implementation helpers and the façade performs registry installation. |
| `core/lib/src/std/config.hal` | global and resolve owners | Use `intern-in` for `load`, `resolve`, `global`, and registration functions. Keep session wrappers explicit. |
| `core/lib/src/std/block/heal.hal` | heal core | Use renamed `intern-in` for `heal <- heal-content`; keep rainbow rendering wrappers explicit. |
| `core/lib/src/work/flow/task.hal` | task engine | Use `intern-in` for its selected engine surface. Do not publish the engine's complete compilation and execution internals. |
| `core/lib/src/work/flow/make.hal` | make host | A singleton `intern-in` is acceptable for `host?`; `intern-all` is disproportionate and would expose host internals. |

## Keep explicit

The following prominent forwarding surfaces are semantic adapters, collision-avoiding façades, generated language definitions, or compatibility boundaries. They should not be converted directly to `intern-all`:

- `core/lib/src/code/vm.hal`: prefixes `interpreter-*`, `halc-*`, `bytecode-*`, and `conformance-*` deliberately avoid owner-name collisions.
- `core/lib/src/db/postgres.hal`: mixes connection aliases with managed lifecycle, Docker, temporary-database, and component behaviour.
- `core/lib/src/lang/core.hal`: mixes selected aliases, registry bootstrap, runtime selection, pointer registration, and grammar-derived macro publication.
- `core/lib/src/std/substrate.hal`: owns `SubstrateNode`, protocol implementations, lifecycle, and compatibility adapters around internal modules.
- `core/lib/src/code/translate.hal`: includes selected rule/translator aliases plus adapted namespace-shape operations and project work.
- `core/lib/src/code/manage.hal`: most public values are constructed task workflows, not aliases of source Vars.
- `core/lib/src/std/sandbox.hal`: direct native capability adapters should remain explicit.
- `core/lib/src-lang/xt/substrate.hal`: `def.xt` publication belongs to the target-language grammar and code-generation path.

## Validation required for the implementation PR

For each changed façade:

1. Capture the sorted keys of `ns-publics` before and after. The symbol set must remain identical unless a separate API change is declared.
2. Compare source and published Var metadata, especially `:doc`, `:arglists`, `:schema`, `:dynamic`, `:macro`, `:private`, `:deprecated`, and `:added`.
3. Exercise both `(:use target.namespace)` and qualified calls through the target namespace.
4. Verify collision handling is deterministic and no later local definition is silently replaced.
5. Run the focused façade tests plus the Foundation API/surface checks.
6. Run JVM and Rust loader tests for any bootstrap or script-publication change.
7. Regenerate migration and parity inventories and require zero unexplained gaps, duplicates, or stale targets.

## Suggested delivery order

1. Convert `std.format`.
2. Convert `workspace`.
3. Add a small façade-surface regression test helper based on sorted `ns-publics` and selected metadata.
4. Convert straightforward curated alias blocks to `intern-in` in separate, reviewable commits.
5. Refactor owner namespaces only where a deliberate whole-surface façade justifies a later `intern-all` conversion.
