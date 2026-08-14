# Truffle CLI startup: lazy Foundation fallback

Issue: `hara-lang/hara#224`

## Change

Truffle no longer executes the portable `std.foundation` fallback at the
start of every source parse.

The context still installs Java runtime primitives and optimized Java library
exports during construction. Before source or HALC lowering, a conservative
demand pass asks whether the unit contains an unresolved unqualified Var or
macro after accounting for:

- top-level definitions and declarations;
- lexical bindings, function parameters, `letfn`, catches, and destructuring;
- quoted and syntax-quoted data;
- generated named-value constructors;
- protocol and multimethod declaration shapes;
- namespace boundaries inside a multi-form source unit.

Only a demanded unit calls `ensureEagerFallbacks()`. Bytecode input goes
directly to the HBC decoder and machine; qualified library references retain
the existing namespace loader path.

The pass is intentionally conservative. An unknown unqualified symbol may
materialize Foundation before ultimately producing an unbound-symbol error.
It must never omit materialization for a valid fallback Var or macro.

## Namespace ordering

Foundation-sensitive namespace declarations are materialized before analyzer
side effects. In particular, `ns`/`ns+` declarations with `:config :override`
or `:config :expose` run against the complete Foundation namespace and then
remove the bindings they deliberately omit.

This ordering prevents a later evaluation from activating the fallback and
reintroducing Vars excluded by the earlier namespace declaration. Blank
namespaces remain protected by the existing `blankNamespaces` refresh guard.

## Regression coverage

`FoundationFallbackDemandTest` verifies:

- `(+ 19 23)` and a closed same-unit `defn` do not expose a fallback-only Var;
- the first fallback function reference materializes Foundation and executes;
- the first fallback macro reference materializes Foundation before macro
  expansion;
- a selective namespace created before later fallback use retains its
  exclusion policy.

The focused test command is:

```sh
mvn -f core/java/pom.xml -Ptruffle \
  -Dtest=hara.truffle.FoundationFallbackDemandTest test
```

Broad verification remains:

```sh
mvn -f core/java/pom.xml -Ptruffle package
./scripts/runtime/run-lib-tests
bash scripts/runtime/build-truffle-native
```

## Startup evidence

Record cold process evidence with the ordinary launcher after packaging:

```sh
for run in $(seq 1 20); do
  /usr/bin/time -p ./core/hara eval '(+ 19 23)' >/dev/null
done
```

The benchmark record should identify the JDK, architecture, runtime mode,
commit, sample count, and median/minimum wall time. This issue treats the
measurement as release evidence rather than a hard CI threshold.

AppCDS and native-image distribution remain cumulative packaging options;
this change removes unnecessary guest fallback execution from the ordinary
JVM path independently of either option.
