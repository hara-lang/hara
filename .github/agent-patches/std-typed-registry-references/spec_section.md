
## Portable schema registries and named references

`std.typed.registry` owns immutable registry data. A canonical registry has the
portable shape:

```clojure
{:registry/type :std.typed.registry/registry
 :registry/namespace demo
 :registry/aliases {model app.model}
 :registry/refers {Id app.model/Id}
 :registry/entries {demo/Node [:map ...]}
 :registry/parents [...]}
```

Registry construction qualifies local entry names. Lookup checks local entries
first, then ordered parents from first to last. Aliases and refers are explicit
data; registry lookup and schema resolution do not evaluate Vars or project
source.

The portable schema layer provides:

```clojure
(schema/normalize-with surface registry)
(schema/reference-names surface registry)
(schema/resolve-reference surface registry)
(schema/resolve-recursive surface registry)
(schema/unresolved-references surface registry)
(schema/validate surface value registry)
(schema/valid? surface value registry)
```

Inside a registry context, a bare symbol and `(var Name)` both normalize to a
`:reference`. Unqualified names use the registry namespace, aliases rewrite a
qualified prefix, and refers map an unqualified name directly to a qualified
name.

Recursive resolution expands reachable definitions but leaves a reference at
a recursive edge. Runtime validation follows references lazily. Its cycle key
is `[qualified-reference value-path]`, so structural recursion that consumes a
map or collection is valid, while an alias-only cycle at one value path reports
`:std.typed.schema/cyclic-reference`. Missing definitions report
`:std.typed.schema/unresolved-reference`. Both are ordinary deterministic
findings, not runtime exceptions.
