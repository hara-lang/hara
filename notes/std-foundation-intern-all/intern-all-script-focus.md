# `intern-all` and portable script publication

Audit baseline: `69cd5b7c444b6bfd9c73965b651ae54bd091ac30` (`main`, 2026-08-19).

This note narrows the repository-wide `std.foundation/intern-all` audit to the portable script migration governed by `code.migrate.script`, including the `tahto.core.script*` source family and the current `lang.core` / `lang.core.script` target ownership boundary.

## Result

`std.foundation/intern-all` is the correct primitive for publishing an **entire already-materialized public namespace surface**. It is not a replacement for the grammar-derived machinery that creates script definitions, target-language pointers, macro families, module registrations, or runtime coordinates.

For the script migration:

- keep `code.migrate.script` as the governing ledger;
- keep grammar generation as the authority for `def.*`, `def$.*`, `defmacro.*`, and `!.*` families;
- use `intern-all` only after a source namespace's complete public surface has been deliberately assigned to one target façade;
- use `intern-in` for curated or renamed publication;
- do not hand-edit generated language families to mimic bulk interning.

The ledger already lists `std.foundation/intern-all` among its target references. The remaining work is to make publication decisions explicit in the ledger and generator inputs, then validate the generated result across loaders.

## Governing source inventory

`core/lib/src/code/migrate/script.hal` records six Foundation source units:

1. `tahto.core.script`
2. `tahto.core.script-annex`
3. `tahto.core.script-control`
4. `tahto.core.script-def`
5. `tahto.core.script-lint`
6. `tahto.core.script-macro`

These units do not map one-to-one onto a single passive Hara façade. Their responsibilities are distributed across language registration, module configuration, runtime lifecycle, grammar publication, target-language pointer construction, compilation, and tests.

That distribution is why a mechanical source-file-to-`intern-all` conversion would be wrong.

## Current target ownership

### `lang.core`

`lang.core` owns the high-level language boundary, including:

- language and runtime installation;
- book/registry access;
- runtime constructors and bootstrap selection;
- code-pointer creation and invocation;
- language definition registration;
- fragment and macro-fragment registration;
- grammar-derived definition and macro forms;
- synchronization of generated language definitions.

These functions are active registration and generation operations. They are not a re-export-only namespace.

### `lang.core.script`

`lang.core.script` owns script/module state and lifecycle, including:

- module loading;
- module configuration and normalization;
- script registration;
- script compilation and evaluation;
- module-loader registration;
- restart and stop behaviour;
- active script and module registries.

This boundary may publish selected stable helpers, but its complete public surface is not automatically the public surface of `lang.core`.

### `lang.common.compiler`

The migration ledger maps compiler-facing source responsibilities to the canonical `compile-entry` path. This remains an adapted implementation boundary rather than namespace publication.

### `std.foundation`

`std.foundation/intern-all` supplies the portable Var-publication primitive. It should replace legacy script-family bulk publication logic only when that logic means exactly “publish all public Vars from this already-loaded namespace into the current namespace.”

It should not absorb grammar hydration, pointer creation, target-language definition registration, or module lifecycle.

## Publication decision rules

Every script-ledger entry involving publication should be classified with one of these dispositions.

### Whole-surface publication

Use `std.foundation/intern-all` when:

- the source namespace has already been materialized;
- every public source Var belongs in the target namespace;
- names are unchanged;
- no source public conflicts with a target local definition or another imported owner;
- automatic propagation of future source publics is intended.

The ledger should name both the source namespace and target façade and record a surface-equality validation.

### Curated publication

Use `std.foundation/intern-in` when:

- only some source Vars are public in the target;
- a target name differs from its source name;
- multiple owners would collide under unrestricted publication;
- source implementation helpers must remain hidden.

The selected symbol list should be ledger data or generated from an authoritative grammar/publication descriptor, not repeated manually in several runtime-specific files.

### Adapted implementation

Keep an explicit target function or generator when it:

- changes arguments or return values;
- registers module/runtime state;
- constructs target-language pointers;
- wraps evaluation or compilation;
- creates macro expansion templates;
- changes error or lifecycle semantics;
- coordinates loader-specific behaviour.

These entries should remain `:adapted` rather than being relabelled as direct publication.

### Obsolete compatibility path

Mark a source path obsolete when the Hara architecture has made it unnecessary. Do not preserve a legacy publication helper merely because it once called `intern` internally.

## `tahto.core.script-macro/intern-in`

The Foundation source inventory includes `tahto.core.script-macro/intern-in`. Its Hara target should be the canonical `std.foundation` publication machinery, but the exact disposition depends on semantics:

- if the source helper only copied selected Vars, map it to `std.foundation/intern-in`;
- if a surrounding source operation copied an entire public namespace, map that operation to `std.foundation/intern-all`;
- if it also generated `def$`, `defmacro`, `!`, free, highlight, or top-level families, retain the generator mapping and use the Foundation macros only at the final publication boundary.

The source helper name alone is not evidence that every adjacent legacy function should collapse into `intern-all`.

## Grammar-derived families

The current target creates language-specific forms from grammar data. This must remain the single source of truth for:

- top-level definition families;
- `def.<tag>` and compatibility aliases;
- `def$.<tag>` fragment families;
- `defmacro.<tag>` families;
- `!.<tag>` evaluation macros;
- highlight macros;
- language tags and compatibility tags;
- target pointer and fragment metadata.

A generated family can emit `intern-all` or `intern-in` forms when the publication contract calls for it, but checked-in generated definitions should not be independently edited. Deterministic regeneration must reproduce the committed result exactly.

## `lang.core` versus `lang.core.script`

The overlap between these namespaces must be resolved explicitly rather than by importing all of one into the other.

A practical ownership rule is:

- `lang.core` exposes stable language-level construction, registry, runtime-selection, pointer, and generated macro entrypoints;
- `lang.core.script` exposes script/module state, loading, compilation, evaluation, and lifecycle;
- compatibility aliases are individually published with `intern-in` or explicit adapted definitions;
- no unrestricted `intern-all` crosses this boundary unless one namespace is intentionally reduced to a pure façade over the other.

This prevents accidental exposure of mutable registries, loader controls, or internal generation helpers through the root language API.

## Required ledger fields for publication entries

For each direct or selected publication, record:

- source repository, revision, path, namespace, and symbol where applicable;
- target namespace and symbol or whole-surface marker;
- disposition: `:direct`, `:adapted`, `:obsolete`, `:test-vector`, or `:new-only`;
- publication mode: `:intern-all`, `:intern-in`, `:generated`, or `:explicit`;
- reason for any non-direct mapping;
- expected target public surface or selected symbol set;
- validation entrypoint;
- runtime profiles/loaders covered.

The ledger checker should reject an `:intern-all` disposition without an expected source namespace and surface validation.

## Validation matrix

### Ledger completeness

Require zero:

- source references without dispositions;
- duplicate source references;
- stale target references;
- unexplained target-only references;
- ambiguous mappings between `lang.core` and `lang.core.script`.

### Deterministic generation

Run the materializer twice from a clean source state and require byte-identical output. The committed generated forms must match the materializer output.

### Namespace surfaces

For every `intern-all` result, compare sorted public symbols before and after. For every `intern-in` result, compare the selected target symbol set and verify that omitted source publics remain absent.

### Var metadata

Compare at least:

- `:doc`;
- `:arglists`;
- `:macro`;
- `:dynamic`;
- `:schema` and static schema metadata;
- target-language tag/coordinate metadata;
- fragment and standalone metadata;
- deprecation and compatibility metadata.

### Representative generated forms

Validate at least:

- one `def.xt` definition;
- one `def.pg` definition;
- one `def$.<tag>` fragment;
- one `defmacro.<tag>` macro;
- one `!.<tag>` evaluation macro;
- one highlight/publication macro.

### Loader parity

Load and invoke representative generated definitions through both JVM and Rust-supported paths. Verify that:

- namespace public surfaces match;
- macros are available at expansion time;
- target-language pointers carry equivalent coordinates;
- module loaders resolve the same definitions;
- restart/stop state is not duplicated by publication.

### Bootstrap ordering

Confirm that source namespaces are loaded before `intern-all` expansion and that introducing publication macros does not create a bootstrap dependency cycle between `std.foundation`, `lang.core`, and `lang.core.script`.

## Recommended next changes

1. Extend `code.migrate.script` ledger entries with an explicit publication mode.
2. Identify legacy publication loops whose exact semantics are whole-namespace Var copying and map only those to `std.foundation/intern-all`.
3. Map selected legacy Var publication to `std.foundation/intern-in`.
4. Keep grammar/macro generation entries adapted and make the generator emit publication forms where appropriate.
5. Add ledger assertions for surface completeness and collisions.
6. Run deterministic regeneration and JVM/Rust loader parity, including representative `def.xt` and `def.pg` cases.
7. Remove obsolete compatibility helpers only after their ledger dispositions and downstream references are closed.

## Conclusion

`intern-all` should simplify the final façade-publication step of the portable script system, not replace the script system itself. The migration remains ledger-led and grammar-derived. The safe endpoint is a smaller set of explicit generators and runtime owners, with `intern-all` used at exact whole-surface boundaries and `intern-in` used everywhere the public contract is curated.
