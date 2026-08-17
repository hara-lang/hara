# Workflow definition families

`work.flow` is the definition-family layer between ordinary declarations and
ordinary work values. It does not schedule or execute work.

```
work.base / std.work     primitive work values and runtimes
work.flow                profile, definition, compile, and reload mechanism
work.flow.task           historical task declaration family
work.flow.make           reloadable make-plan declaration family
```

`std.work` remains the primitive dependency until the `work.base` move is
complete. The declaration formats introduced here do not depend on that future
namespace move.

## Profile activation

A namespace activates and optionally customises a family with `def.workflow`:

```clojure
(def.workflow [:task])

(def.workflow
  [:task :lint]
  {:extends [:task]
   :defaults {:execution {:parallel false}}
   :return {:select :summary}
   :flow/remove [[:defaults :return :package]]})
```

Profiles are immutable and namespace-local. Maps deep-merge, scalar values and
functions replace parent values, `nil` inherits, and `:flow/remove` is the only
explicit removal operation. A child path resolves the nearest registered flow
descriptor, so `[:task :lint]` can extend `[:task]` without registering a second
compiler.

`def.workflow` never defines a task or make value. Named declarations belong to
the family-specific macro.

## Task declarations

```clojure
(def.workflow [:task])

(def.task lint
  {:main {:fn lint-project
          :argcount 1}})
```

`def.task` selects the active `[:task]` profile, or a profile named by the
definition's `:workflow` key. It compiles through the historical task template,
which remains an ordinary `std.work` graph and preserves the zero-to-four
argument callable convention. `deftask` is a thin compatibility alias for
`def.task`.

Task redefinition compiles a complete candidate and replaces the previous work
value only after compilation succeeds.

## Make declarations

```clojure
(def.workflow [:make])

(def.make +project+
  {:root "."
   :compile-entry compile-entry
   :default [{:id :assets}]
   :sections {:docs [{:id :guide}]}
   :triggers ['app.core]})
```

A make definition compiles to an immutable plan containing target work graphs,
target source specifications, triggers, and the normalised declaration. The
public value is a live host containing that plan.

Redefinition follows this order:

1. normalise the complete declaration;
2. validate every target hook and entry compiler;
3. compile every target into ordinary `std.work` values;
4. retain the current host unchanged if any earlier stage fails;
5. atomically replace the installed plan;
6. reconcile triggers when the host is running.

The host's identity and running state survive successful reloads. The revision
number advances whenever a new plan is installed.

Make execution is explicit:

```clojure
(make/run +project+ :default)
(make/run +project+ :docs input)
(make/run +project+ [:default :assets])
```

Each make entry is a checkpointed `work/step`. Preparation and completion are
pure work. Entry compilers have the narrow signature:

```clojure
[definition entry input context] -> value
```

A compiler can be selected by the entry's `:compile`, the definition's `:types`
or `:formats` registry, the definition's `:compile-entry`, or a function-valued
entry `:main`.

Trigger installation defaults to `:on-define`. A profile may select
`:trigger-policy :manual`, after which `make/start!` and `make/stop!` own the
lifecycle explicitly.

## Flow descriptor contract

A definition family is an ordinary map. The generic machinery recognises:

```clojure
{:flow/path       [:family]
 :flow/version    1
 :flow/product    :work-or-host
 :flow/defaults   {...}
 :flow/extends    [:parent]
 :flow/merge      merge-function
 :flow/configure  configure-function
 :flow/normalise  normalise-function
 :flow/compile    compile-function
 :flow/reconcile  reconcile-function
 :flow/invoke     invoke-function}
```

`flow/define!` updates its definition registry only after normalisation,
compilation, and reconciliation all succeed. This is the shared atomic boundary
for replaceable task values and identity-preserving make hosts.
