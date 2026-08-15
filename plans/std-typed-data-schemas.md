# `std.typed` Data Schemas, Result Refinement, and `code.test`

Issue: #667  
Parent integration issue: #641  
Status: proposed design

## 1. Decision

`std.typed` will become Hara's portable schemas-as-data subsystem.

It will follow the architectural direction proven by Malli without copying Malli's exact syntax or native implementation model:

- a schema is ordinary immutable Hara data;
- surface forms normalize into one portable schema AST;
- properties, children, entries, references, and registries are introspectable;
- validation and explanation are separate operations;
- validator and explainer functions may be compiled and cached, but the schema AST remains authoritative;
- metadata annotations remain passive data;
- optional runtime instrumentation is a separate adapter;
- static inference consumes the same schema data but remains conservative and non-evaluating;
- source-block parsing belongs to `tool.lint`, not `std.typed`;
- completed checks return one native `Result<boolean>`; nested diagnostics remain ordinary Failure data.

Hara will retain its current `[:fn [arguments] return]` and `[:function ...]` function schema syntax. It will not switch to Malli's `:=>`/`:cat` syntax in this migration. Existing metadata, compiler schema models, and Hara's function-oriented notation already depend on `:fn`, and a second equivalent function grammar would add ambiguity without adding capability.

## 2. Current state and required corrections

The current implementation already supplies a useful base:

- `std.typed.schema` accepts portable surface forms and normalizes them to `{:kind ...}` maps;
- unions flatten and deduplicate;
- named references resolve without evaluation;
- `compatible?` and directional `assignable?` exist;
- `std.typed.infer` performs shallow literal, binding, branch, collection, arithmetic, and known-call inference;
- schema metadata feeds `tool.lint`.

The migration must correct four architectural problems.

### 2.1 `std.typed.infer` owns source-block knowledge

It currently imports `std.block` and dispatches on block type, tag, children, and value. That makes the type system depend on one source representation and prevents use by ordinary runtime values, compiler forms, remote tools, and non-block parsers.

The hard boundary will be:

```text
source text
  -> tool.lint block reader
  -> tool.lint.schema block/form adapter
  -> ordinary annotated forms
  -> std.typed.infer
```

No namespace under `std.typed.*` may import `std.block`.

### 2.2 Runtime validation returns flat findings

`std.typed.schema/validate` currently returns flat `:finding/*` maps. A union mismatch loses branch explanations, missing keys are represented only as `nil`, and composite failures cannot be rendered or traversed structurally.

The canonical runtime explanation becomes a recursive Failure tree. A temporary compatibility adapter may flatten Failure leaves into the old finding shape while internal consumers migrate.

### 2.3 `code.test` stores executable checkers in result maps

`Checker` currently stores its function, expected value, and form together; `verify` emits legacy `{:type :code/test ...}` maps; collection checkers collapse nested comparisons to booleans. These maps cannot express recursive explanations and are unsuitable for transport because they may retain executable checker values.

The checker object remains a local executable value, but every checker must expose a separate portable descriptor. A comparison returns a native Result whose context stores only the descriptor and Failure data.

### 2.4 Test timeout and evaluation outcomes are conflated

`code.test.base.process` currently owns `TimeoutValue`, a custom timeout race, evaluation maps, comparison maps, and fact aggregation. These concerns must be separated:

```text
evaluation completion -> native Result
comparison             -> native Result<boolean>
fact/batch summary     -> ordinary aggregate map
```

`res-synchronize` owns waiting, timeout, and cancellation. `TimeoutValue` is removed after consumers migrate.

## 3. Goals

1. Make schemas reusable across metadata, runtime checking, static inference, lint, tests, transport descriptors, and future instrumentation.
2. Preserve the existing Hara schema grammar wherever it is already unambiguous.
3. Add Malli-style property maps, map-entry properties, registries, recursive references, AST/form round trips, and explainers.
4. Use one strict portable Failure contract for typed checks and test comparisons.
5. Model native Result values and `res-*` operations precisely enough for useful inference and occurrence narrowing without adding general type variables or unification.
6. Migrate `code.test` comparisons to `Result<boolean>` while leaving fact, namespace, batch, event, and report aggregates as ordinary maps.
7. Keep pure schema operations pure. Result is used for completed check boundaries, not as a replacement for every return value.

## 4. Non-goals for the first implementation

- exact Malli syntax compatibility;
- Hindley–Milner inference, unification, polymorphic type variables, or whole-program proof obligations;
- implicit global mutable schema registration;
- coercion, decoding, generation, shrinking, or property-based test generation;
- automatic instrumentation of every annotated function;
- arbitrary executable functions inside portable or transported schemas;
- changing browser or XTalk APIs to return Hara Result values;
- replacing optional values, streams, progress, lifecycle state, events, or aggregates with Result;
- making every ordinary expected vector or map in `code.test` mean a schema.

## 5. Namespace ownership

### 5.1 `std.typed`

The curated public facade. It exports the stable common operations without becoming an implementation owner.

Proposed facade:

```clojure
schema
schema?
form
ast
from-ast
valid?
explain
check
validator
explainer
failure?
failure-seq
failure-count
registry
composite-registry
```

### 5.2 `std.typed.schema`

Owns:

- surface grammar parsing;
- canonical normalization;
- AST and form round trips;
- type, properties, children, and entries;
- structural walking and transformation;
- reference descriptors;
- compatibility and assignability;
- function arity projection.

It does not inspect source blocks, evaluate predicates, emit lint findings, or return test results.

### 5.3 `std.typed.registry`

Owns immutable registry composition and lookup.

Registry values are maps from qualified names to schema forms or normalized schemas:

```clojure
{'demo/Address
 [:map {:closed true}
  [:street :str]
  [:postcode :str]]

 'demo/User
 [:map {:closed true}
  [:name :str]
  [:address [:ref 'demo/Address]]]}
```

Operations:

```clojure
(registry definitions)
(composite-registry project local builtins)
(lookup registry name)
(schemas registry)
```

Lookup precedence is explicit and left-to-right. There is no ambient mutable registry in portable operations. A future explicit runtime registry adapter may provide mutation, but it is not the default or the source of truth.

### 5.4 `std.typed.explain`

Owns:

- validator and explainer compilation;
- runtime value checking;
- canonical Failure construction;
- deterministic Failure traversal;
- the Result-returning `check` boundary.

### 5.5 `std.typed.infer`

Owns conservative inference over ordinary forms and values.

Its input form model is data, for example:

```clojure
{:op :invoke
 :form '(res-success 42)
 :children [{:op :var :name 'res-success}
            {:op :literal :value 42}]
 :source {:file "src/demo.hal" :row 4 :col 3}}
```

The exact adapter shape may be smaller, but it must not expose `std.block` objects or protocols.

### 5.6 `tool.lint.schema`

Owns translation from source blocks to plain annotated forms and from typed Failure or inference data to source-oriented lint findings.

It may depend on `std.block`; `std.typed` may not depend back on it.

### 5.7 `code.test.checker.schema`

Owns the explicit typed checker:

```clojure
(conforms User)
(conforms [:map {:closed true}
           [:name :str]
           [:age [:int {:min 0}]]])
```

It adapts `std.typed/check` to `IMatch` and provides a portable checker descriptor.

## 6. Schema representation

There are three layers, but only one source of truth.

### 6.1 Surface form

The value a user writes in metadata, a schema Var, a test, or a registry.

```clojure
[:map {:closed true :title "User"}
 [:id :int]
 [:nickname {:optional true} [:str {:min-count 1}]]]
```

### 6.2 Portable normalized AST

The canonical ordinary-data representation consumed by validation, inference, lint, and introspection.

```clojure
{:kind :map
 :properties {:closed true :title "User"}
 :form [:map {:closed true :title "User"}
        [:id :int]
        [:nickname {:optional true} [:str {:min-count 1}]]]
 :entries
 [{:key :id
   :properties {:optional false}
   :schema {:kind :primitive
            :name :int
            :properties {}
            :form :int}}
  {:key :nickname
   :properties {:optional true}
   :schema {:kind :primitive
            :name :str
            :properties {:min-count 1}
            :form [:str {:min-count 1}]}}]}
```

The existing `:kind` representation is retained and extended. Current consumers can migrate incrementally instead of converting to a second unrelated model.

Every normalized schema has:

- `:kind`;
- `:properties`;
- `:form`.

Composite schemas additionally have one role-specific collection:

- `:types` for `:or` and `:and`;
- `:item` for homogeneous collections;
- `:items` for tuples;
- `:entries` for maps;
- `:key` and `:value` for `:map-of`;
- `:inputs` and `:output` for functions;
- `:arities` for multi-arity functions;
- `:target` for references;
- `:data` for Result schemas.

### 6.3 Compiled validator or explainer

A local function produced from the normalized AST and an explicit registry/options map.

```clojure
(def validate-user (typed/validator User {:registry app-registry}))
(def explain-user  (typed/explainer User {:registry app-registry}))
```

Compiled functions are caches and execution aids. They are never inserted into the AST, schema form, Result context, HTA frame, or JSON envelope.

## 7. Surface grammar

A property map, when allowed, is the first argument after the schema head.

### 7.1 Primitives

Existing primitive names remain canonical:

```clojure
:any :nil :bool :num :int :float :decimal :str :char :regex
:keyword :symbol :list :vector :map :set :fn :atom :bytes :promise
```

Existing aliases continue normalizing:

```clojure
:boolean -> :bool
:number  -> :num
:integer -> :int
:string  -> :str
```

A primitive keyword is shorthand for a primitive with empty properties:

```clojure
:int
[:int {}]
```

### 7.2 Scalar constraints

Initial standardized properties:

```clojure
[:int {:min 0 :max 100}]
[:num {:min-exclusive 0}]
[:str {:min-count 1 :max-count 64}]
[:str {:pattern "^[a-z]+$"}]
[:enum :draft :published]
[:= 42]
```

Unknown unqualified properties are schema-definition errors. Namespaced properties are retained as extension data and ignored by the core validator unless a registered extension handles them.

### 7.3 Alternatives and conjunctions

```clojure
[:or :str :nil]
[:maybe :str]
[:and :int [:int {:min 0}]]
[:not :nil]
```

`:maybe` remains normalization sugar for `[:or T :nil]`.

`:or` preserves declaration order for explanations. It may still deduplicate exactly equal branches, but must not reorder them.

### 7.4 Collections

```clojure
[:vector :int]
[:vector {:min-count 1 :max-count 10 :distinct true} :int]
[:set :keyword]
[:list :any]
[:tuple :keyword :int :str]
[:map-of :keyword :any]
```

Bare primitive `:vector`, `:set`, `:list`, and `:map` continue to mean only the collection type. Parameterized forms add member constraints.

### 7.5 Maps

```clojure
[:map
 [:name :str]
 [:age :int]]

[:map {:closed true}
 [:name :str]
 [:nickname {:optional true} :str]]
```

Map entry form:

```clojure
[key schema]
[key entry-properties schema]
```

Initial entry properties:

- `:optional` boolean, default false;
- `:default` retained as data for future decoding but does not make validation mutate input;
- namespaced extension properties.

Map-level `:closed true` rejects unexpected keys. The default remains open for compatibility.

Duplicate entry keys are schema-definition errors.

### 7.6 Function schemas

Existing syntax remains:

```clojure
[:fn [:int :int] :int]
[:fn [:str & :any] :str]
[:function
 [:fn [:int] :int]
 [:fn [:str & :any] :str]]
```

Function properties may include documentation, purity, or effect descriptors as ordinary data, but runtime call instrumentation is separate.

### 7.7 References and local registries

Canonical reference form:

```clojure
[:ref 'demo/User]
```

Existing `(var User)` forms remain accepted and normalize to `:reference` AST nodes.

A local schema root may carry a registry:

```clojure
[:schema
 {:registry
  {'Node
   [:map
    [:value :int]
    [:next {:optional true} [:ref 'Node]]]}}
 [:ref 'Node]]
```

Normalization does not recursively inline references. Validators and explainers resolve one reference step at execution time, so recursive schemas remain finite data.

Reference resolution options retain the existing namespace, alias, and refer-map context.

### 7.8 Native Result schemas

```clojure
[:result :int]
[:result {:status :success} :int]
[:result {:status :error} :int]
```

Semantics:

- `[:result T]` accepts a success whose data conforms to `T`, or any structurally valid error Result.
- `[:result {:status :success} T]` accepts only a successful Result whose data conforms to `T`.
- `[:result {:status :error} T]` accepts only an error Result containing a native Hara Error. `T` is retained as the Result's static data parameter but is not checked for an error value.
- `:status` may be absent, `:success`, or `:error`; any other value is a schema-definition error.
- Result context is not part of Result identity and is not validated by default.

A later extension may add an explicit `:context-schema` property. It is excluded from the first implementation to avoid making diagnostic context part of ordinary outcome validity.

### 7.9 Promise schemas

Extend the current primitive with a parameterized form:

```clojure
:promise
[:promise :int]
```

Bare `:promise` normalizes to `[:promise :any]`. Static inference may preserve the promised value type. Runtime validation checks only that the value is a Promise; it does not wait for settlement.

## 8. Schema API

### 8.1 Construction and introspection

```clojure
(schema form)
(schema form options)
(schema? value)
(form schema)
(ast schema)
(from-ast ast)
(type schema)
(properties schema)
(children schema)
(entries schema)
(walk schema pre post)
```

`schema` and the current `normalize` are pure and may throw a native Error for an invalid schema definition. Invalid schema syntax is a programmer/configuration error, not an ordinary value mismatch.

For callers that need a completed boundary outcome:

```clojure
(schema-result form)
(schema-result form options)
```

returns `Result<Schema>`. This wrapper catches normalization and registry errors. It does not replace the pure `schema` function.

### 8.2 Validation and explanation

```clojure
(valid? schema value)
(explain schema value)
(check schema value)
(validator schema)
(explainer schema)
```

Semantics:

- `valid?` returns a plain boolean and may throw for invalid schemas or internal execution errors;
- `explain` returns `nil` for a valid value or an ordinary portable explanation map;
- `check` is the completed boundary and returns one `Result<boolean>`;
- `validator` and `explainer` compile local functions.

This preserves Result's role as a completed outcome rather than turning every pure helper into a Result-producing API.

### 8.3 Compatibility API

During migration:

- current `normalize` delegates to `schema` and returns the normalized AST;
- current `valid?` delegates to the new validator;
- current `validate` flattens deterministic Failure leaves into legacy `:finding/*` maps;
- current `normalize-result` remains temporarily available as a compatibility map adapter, while new code uses `schema-result`;
- `compatible?`, `assignable?`, `matching-arity`, and `project-arities` operate on the extended AST.

The compatibility map adapters are removed only after all repository consumers migrate.

## 9. Canonical explanation and Failure model

`explain` returns:

```clojure
{:schema schema-form
 :value actual-value
 :failures [Failure ...]}
```

A Failure always has every field:

```clojure
{:failure/code keyword
 :failure/path vector
 :failure/in vector
 :failure/actual any
 :failure/expected any
 :failure/message string
 :failure/context map
 :failure/children [Failure ...]}
```

### 9.1 Path semantics

`:failure/path` follows the normalized schema structure rather than raw surface-vector indexes. This keeps paths stable when an optional property map is added.

Examples:

```clojure
[:entries :age]   ;; map entry schema
[:item]           ;; homogeneous collection member
[:items 2]        ;; tuple item
[:types 1]        ;; second :or or :and branch
[:data]           ;; Result success data
```

`:failure/in` follows the actual value:

```clojure
[:user :age]
[3 :name]
```

### 9.2 Missing and unexpected values

Missing required input:

```clojure
{:failure/code :typed/missing-key
 :failure/path [:entries :name]
 :failure/in [:name]
 :failure/actual nil
 :failure/expected :str
 :failure/message "required key :name is missing"
 :failure/context {:present? false}
 :failure/children []}
```

Unexpected closed-map key:

```clojure
{:failure/code :typed/unexpected-key
 :failure/path []
 :failure/in [:debug]
 :failure/actual true
 :failure/expected nil
 :failure/message "closed map does not allow key :debug"
 :failure/context {:present? true}
 :failure/children []}
```

### 9.3 Composite failures

A map, collection, tuple, conjunction, or Result-data mismatch creates a parent node when grouping adds meaning.

An alternative failure always creates one parent with one child tree per attempted branch:

```clojure
{:failure/code :typed/no-alternative
 :failure/path []
 :failure/in []
 :failure/actual value
 :failure/expected [:or branch-a branch-b]
 :failure/message "value did not match any alternative"
 :failure/context {:branches 2}
 :failure/children [branch-a-failure branch-b-failure]}
```

Children retain declaration order.

### 9.4 Failure traversal

```clojure
(failure? value)
(failure-seq failures)
(failure-count failures)
```

`failure-seq` returns depth-first leaves only. A node with no children is a leaf. `failure-count` counts the same sequence.

### 9.5 Execution errors

The following are not ordinary Failure values:

- malformed schema forms;
- unresolved required references;
- registry cycles that cannot be executed safely;
- extension compiler errors;
- validator or explainer crashes;
- comparison timeouts.

They become native `Result/error` outcomes when called through `check` or `Test/compare`. If explanation had already produced valid partial Failure data, it may be attached under `:failures` in Result context.

## 10. `typed/check`

Successful conformance:

```clojure
(res-success
 true
 {:typed {:schema schema-form}
  :failures []})
```

Ordinary mismatch:

```clojure
(res-success
 false
 {:typed {:schema schema-form}
  :failures [failure ...]})
```

Schema compilation or checking failure:

```clojure
(res-error
 error
 {:typed {:schema schema-form}
  :failures partial-failures})
```

An ordinary mismatch is never a Result error. Dereferencing a completed successful check always returns `true` or `false`.

## 11. Static schema model for Result

The public metadata signatures stay conservative. Precision comes from builtin inference transfer rules.

### 11.1 Conservative metadata

```clojure
res-success
[:function
 [:fn [:any] [:result :any]]
 [:fn [:any :map] [:result :any]]]

res-error
[:function
 [:fn [:any] [:result :any]]
 [:fn [:any :map] [:result :any]]]

res-synchronize
[:function
 [:fn [:any] [:result :any]]
 [:fn [:any :map] [:result :any]]]

res?
[:fn [:any] :bool]

res-success?
[:fn [[:result :any]] :bool]

res-error?
[:fn [[:result :any]] :bool]

res-status
[:fn [[:result :any]] [:enum :success :error]]

res-data
[:fn [[:result :any]] :any]

res-error-value
[:fn [[:result :any]] [:maybe :hara/Error]]

res-context
[:fn [[:result :any]] :map]

res-with-context
[:fn [[:result :any] :map] [:result :any]]
```

### 11.2 Internal transfer operations

`std.typed.infer` keeps a data table keyed by qualified function symbol:

```clojure
{'std.foundation/res-success     {:transfer :result/success}
 'std.foundation/res-error       {:transfer :result/error}
 'std.foundation/res-synchronize {:transfer :result/synchronize}
 'std.foundation/res-data        {:transfer :result/data}
 'std.foundation/res-error-value {:transfer :result/error-value}
 'std.foundation/res-with-context {:transfer :result/with-context}}
```

The transfer key selects a small pure rule. Executable transfer functions are implementation-local and are not stored in schema metadata or transported registries.

### 11.3 Transfer rules

Given inferred argument schema `T`:

```text
(res-success x)
  -> Result<success, T>

(res-error e)
  -> Result<error, any>

(res-synchronize ordinary-T)
  -> Result<T>

(res-synchronize Result<T>)
  -> Result<T>                 ; existing Result is preserved

(res-synchronize Promise<T>)
  -> Result<T>

(res-synchronize Promise<Result<T>>)
  -> Result<Result<T>>         ; Promise payload is not flattened

(res-with-context Result<T> map)
  -> Result<T>
```

A dereferenceable container whose dereferenced schema is `T` produces `Result<T>`. If `T` itself is Result-shaped, it remains nested data.

### 11.4 Accessors

```text
res-status Result<T>
  -> enum(success,error)

res-status Result<success,T>
  -> enum(success)

res-data Result<T>
  -> maybe(T)

res-data Result<success,T>
  -> T

res-data Result<error,T>
  -> nil

res-error-value Result<T>
  -> maybe(hara/Error)

res-error-value Result<success,T>
  -> nil

res-error-value Result<error,T>
  -> hara/Error

res-context Result<T>
  -> map
```

Dereferencing `Result<T>` has value schema `T`. The possibility of throwing the contained Error is control flow and is not encoded as a union return type.

### 11.5 Occurrence narrowing

Predicate tests emit propositions:

```clojure
(res? x)
-> {:op :is-schema
    :binding 'x
    :schema [:result :any]}

(res-success? x)
-> {:op :result-status
    :binding 'x
    :status :success}

(res-error? x)
-> {:op :result-status
    :binding 'x
    :status :error}
```

In a true branch:

```clojure
(if (res-success? outcome)
  (inc (res-data outcome))
  (log-error (res-error-value outcome)))
```

`outcome` is refined to successful Result in the first branch and error Result in the second. This precision is implemented as builtin occurrence rules; it does not require public generic type variables.

## 12. `code.test` comparison model

A test assertion has three distinct values:

1. evaluation outcome — a value or native Error;
2. comparison outcome — one native `Result<boolean>`;
3. diagnostic context — identity, actual, expected descriptor, and Failure trees.

### 12.1 Explicit schema checker

Plain values retain their current exact/satisfies interpretation. Schema checking is explicit:

```clojure
(fact "loads a valid user"
  (load-user)
  => (conforms
      [:map {:closed true}
       [:name :str]
       [:age [:int {:min 0}]]]))
```

This avoids making every expected vector such as `[1 2 3]` ambiguous between exact data and a schema form.

### 12.2 Checker descriptor split

A local Checker may retain executable fields, but must expose a portable descriptor:

```clojure
{:checker/type :typed/schema
 :checker/form [:map [:name :str]]}
```

Other examples:

```clojure
{:checker/type :exactly
 :checker/form 42}

{:checker/type :throws
 :checker/form {:error/type :hara/Error
                :error/message "boom"}}
```

A Result context stores the descriptor, never the Checker object or its function.

A revised local shape may be:

```clojure
(defstruct Checker
  [tag form compare explain display])
```

Only `tag` and `form` are portable. `compare`, `explain`, and `display` are local functions.

### 12.3 `IMatch`

The protocol contract becomes:

```clojure
(match-value matcher actual) -> Result<boolean>
```

All built-in checkers migrate together. During implementation, an adapter may normalize a legacy boolean return to `res-success`, but the adapter is temporary and must not remain as a public dual contract.

### 12.4 Native Test API

```clojure
(Test/compare actual expected)
(Test/result name actual expected)
(Test/result name actual expected comparison-result)
(Test/passed? result)
(Test/actual result)
(Test/expected result)
(Test/failures result)
(Test/failure-seq result)
(Test/failure-count result)
(Test/failure code path in actual expected message context children)
(Test/failure? value)
```

`Test/compare`:

- invokes `IMatch` for Checker values;
- otherwise performs ordinary equality/satisfaction comparison as defined by the test surface;
- returns success/true for a match;
- returns success/false with Failure data for a mismatch;
- returns error for checker crashes, malformed checker responses, or unexpected comparison execution failure.

`Test/result` adds test identity and display to an existing Result without recomputing comparison. It shallow-merges adapter-owned top-level context while preserving `:typed` and `:failures` supplied by typed checkers.

### 12.5 Comparison context

Pass:

```clojure
Result[
 :success
 true
 nil
 {:test {:name name
         :actual actual
         :expected portable-descriptor}
  :failures []}]
```

Mismatch:

```clojure
Result[
 :success
 false
 nil
 {:test {:name name
         :actual actual
         :expected portable-descriptor}
  :typed {:schema schema-form}
  :failures [failure ...]}]
```

Checker execution failure:

```clojure
Result[
 :error
 nil
 native-error
 {:test {:name name
         :actual actual
         :expected portable-descriptor}
  :failures partial-failures}]
```

`Test/passed?` is equivalent to:

```clojure
(and (res-success? result)
     (= true (res-data result)))
```

### 12.6 Evaluation and `throws`

`code.test.base.process/check` becomes an orchestration adapter:

```text
invoke actual thunk
  -> catch direct throw as Result/error
  -> res-synchronize returned value with timeout/context
  -> inspect expected checker
```

If evaluation is an error Result and expected is `throws` or `throws-info`:

- pass `res-error-value` as the actual checker input;
- compare normally;
- a matching error is success/true;
- a nonmatching error is success/false with Failure data.

If evaluation is an error Result and the checker is not an error checker, propagate the error Result after attaching test identity. It is not converted to success/false.

If evaluation succeeds but `throws` was expected, comparison is success/false.

This removes the current special `{:status :exception :data error}` evaluation envelope.

### 12.7 Collection and logical checkers

`contains`, `just`, `all`, and `any` no longer discard nested mismatch detail.

- `contains` and `just` create parent failures containing child comparison failures at key/index paths;
- `all` creates a conjunction parent when children fail;
- `any` creates a no-alternative parent with one child tree per attempted checker;
- declaration order is preserved wherever input order is meaningful;
- unordered matching records the selected actual indexes in Failure context so rendering is deterministic.

Generic checker Failure codes use the `:test/*` namespace. Typed schema failures keep `:typed/*` codes. Both use the same strict Failure map and native Test traversal functions.

## 13. `code.test.base.process`

### 13.1 New `check` output

`check` returns one `Result<boolean>`.

It no longer returns:

```clojure
{:pass ...
 :status ...
 :actual ...
 :expected ...}
```

### 13.2 Fact aggregate

A fact remains ordinary domain data:

```clojure
{:namespace namespace
 :name name
 :meta metadata
 :status :passed|:failed|:error|:timeout|:skipped|:cancelled
 :checks [Result<boolean> ...]
 :elapsed milliseconds}
```

Status reduction:

```text
skipped/cancelled fact policy first
otherwise any timeout Result error -> :timeout
otherwise any non-timeout Result error -> :error
otherwise any success false -> :failed
otherwise all success true -> :passed
```

Timeout detection reads the normalized Error code, not a `TimeoutValue` instance.

### 13.3 Hooks

Setup and teardown completion also use `res-synchronize`, but hook outcomes are not inserted as comparison Results. A hook error becomes the fact's execution error and may be recorded in ordinary fact context/report data.

### 13.4 Removed compatibility

After migration:

- remove `TimeoutValue`;
- remove `{:type :code/test :status ... :data ...}` evaluation maps;
- remove legacy `verify` result maps;
- replace `checks-pass?`, `checks-error?`, and `checks-timeout?` with Result-aware reducers;
- update printers/reporters to call Test accessors rather than map fields.

## 14. Lint integration

The lint adapter converts blocks into ordinary forms before inference. It also converts typed analysis into source-oriented findings.

Runtime Failure and lint Finding remain different contracts:

```clojure
Failure
{:failure/code ...
 :failure/path ...
 :failure/in ...}

Finding
{:file ...
 :row ...
 :col ...
 :level ...
 :type ...
 :message ...}
```

A lint finding may retain a portable Failure under a namespaced key for editor drill-down, but `std.typed` never gains file/line/block dependencies.

The dependency audit must fail when any `core/lib/src/std/typed/**` file contains a `std.block` require or qualified call.

## 15. Schema metadata and definitions

Existing metadata remains valid:

```clojure
(defn ^{:schema [:fn [:int] :int]}
  increment
  [value]
  (+ value 1))
```

Named data schemas may be ordinary Vars:

```clojure
(def ^{:schema/definition true}
  User
  [:map {:closed true}
   [:name :str]
   [:age [:int {:min 0}]]])
```

A future `defschema` convenience macro may expand to this data definition, but it must not mutate an implicit registry. Project indexing collects definitions into an explicit registry.

Function metadata schemas and named value schemas share the same parser, AST, registry, and explanation model.

## 16. Transport rules

Schema forms and Failure trees are intended to be portable.

Portable schema forms may contain:

- nil, booleans, signed integers, strings;
- keywords and symbols when transported through HTA;
- vectors, maps, sets where supported by the selected transport;
- qualified reference names;
- namespaced extension properties whose values are portable.

JSON Result projection has a stricter value subset. Adapters requiring JSON must project keyword/symbol schema forms through an explicit schema JSON representation rather than relying on ordinary strict JSON writing. That projection is a later transport adapter and does not change the canonical Hara form.

Executable functions, native handles, Promises, atoms, streams, and local validator closures are never portable schema descriptors. Encoding fails explicitly when they appear outside the local-only `:display` exception already defined for Result context.

## 17. Implementation tranches

### Tranche 1 — contract and compatibility skeleton

- land this design;
- add issue-linked namespace/API ledger;
- add dependency audit for `std.block` under `std.typed`;
- define canonical Failure constructors and predicates;
- preserve existing schema tests unchanged through adapters.

### Tranche 2 — schema AST, properties, and registries

- extend normalizer with property maps;
- implement map entry properties and closed maps;
- add `:and`, `:not`, `:set`, `:map-of`, `:=`, `:ref`, `:schema`, `:result`, and parameterized `:promise`;
- implement form/AST round trips and introspection;
- implement immutable/composite registries and recursive resolution;
- add malformed-schema coverage.

### Tranche 3 — recursive explanation

- implement validator/explainer compilation;
- implement strict Failure trees;
- implement deterministic leaf traversal and counts;
- adapt current `validate` to flatten leaves;
- implement `typed/check` as `Result<boolean>`.

### Tranche 4 — source boundary and inference

- define the plain annotated-form input model;
- move block conversion into `tool.lint.schema`;
- remove `std.block` from `std.typed.infer`;
- preserve current lint behavior through focused tests;
- add Result, Promise, and predicate narrowing transfer rules.

### Tranche 5 — native Test parity

- implement Test comparison Result construction in Rust and Java;
- implement Test accessors and Failure traversal;
- test equality mismatch, typed mismatch, checker crash, timeout, and throws behavior;
- keep Rust/Java context and display behavior equivalent.

### Tranche 6 — checker migration

- split Checker executable state from portable descriptors;
- add `conforms`;
- migrate exactly, satisfies, approx, stores, throws, throws-info, contains, just, all, and any;
- make `IMatch/match-value` return Result;
- preserve public `code.test` syntax.

### Tranche 7 — process and aggregate migration

- make `check` return Result;
- use `res-synchronize` for returned Promises and timeouts;
- remove `TimeoutValue`;
- update fact status reducers, reporters, artifacts, CLI, work integration, and listeners;
- retain ordinary aggregate maps.

### Tranche 8 — parity and cleanup

- update canonical and Rust-mirrored HAL sources;
- update namespace manifests;
- remove legacy result maps and adapters;
- run typed, lint, native Test, code.test, Result transport, Rust, and Java suites;
- document the stable public surface.

## 18. Required examples

### 18.1 Runtime schema check

```clojure
(def User
  [:map {:closed true}
   [:name [:str {:min-count 1}]]
   [:age [:int {:min 0}]]])

(typed/valid? User {:name "Ada" :age 42})
;; true

(typed/check User {:name "Ada" :age -1})
;; Result/success false with :typed and :failures context
```

### 18.2 Recursive schema

```clojure
(def NodeRegistry
  (typed/registry
   {'Node
    [:map
     [:value :int]
     [:children [:vector [:ref 'Node]]]]}))

(typed/check [:ref 'Node]
             {:value 1
              :children [{:value "bad" :children []}]}
             {:registry NodeRegistry})
```

The leaf failure has:

```clojure
:failure/path [:entries :children :item :entries :value]
:failure/in   [:children 0 :value]
```

### 18.3 Result refinement

```clojure
(defn read-count
  [outcome]
  (if (res-success? outcome)
    (inc (res-data outcome))
    0))
```

For input schema `[:result :int]`, the true branch treats `res-data` as `:int`.

### 18.4 Test schema checker

```clojure
(fact "returns a valid user"
  (load-user)
  => (conforms User))
```

The fact's `:checks` vector contains one Result. A negative age is a successful comparison outcome containing false and typed Failure context, so the fact status is `:failed`, not `:error`.

### 18.5 Throws checker

```clojure
(fact "rejects missing identity"
  (load-user nil)
  => (throws :hara/Error "identity required"))
```

The evaluation Error is passed as actual checker input. A matching Error produces success/true. An unexpected runtime Error with a non-error checker remains Result/error.

## 19. Acceptance gates

### Schema data

- existing schema forms continue to normalize;
- property maps have one canonical placement;
- AST -> form -> AST is stable;
- local and composite registries resolve deterministically;
- recursive references do not expand infinitely;
- unsupported unqualified properties fail early;
- executable values cannot enter portable descriptors.

### Explanation

- every Failure field is present;
- missing input records `{:present? false}`;
- `:or` retains one child tree per branch in declaration order;
- map, collection, tuple, conjunction, closed-map, Result-status, and Result-data failures are covered;
- DFS leaves and counts are deterministic;
- invalid schema/checker execution is Result/error, not mismatch.

### Inference

- no `std.block` dependency under `std.typed`;
- current literal, let, if, arithmetic, and call inference remains;
- Result success/error predicates narrow branches;
- Result accessors preserve/refine data and Error types;
- Promise synchronization preserves nested Result payloads;
- no public generic-variable or unification machinery is introduced.

### Testing

- every completed comparison is one Result<boolean>;
- pass is success/true;
- mismatch is success/false;
- checker crash, evaluation failure, and timeout are error Results;
- throws checkers consume captured native Errors;
- Result context stores portable checker/schema descriptors, not functions;
- `Test/result` never recomputes comparison;
- facts and reports remain ordinary aggregates;
- `TimeoutValue` and legacy code.test comparison maps are removed.

### Runtime parity

- Rust and Java implement the same Test accessor and comparison contracts;
- HTA and JSON Result transport continue passing;
- generated HAL mirrors remain synchronized;
- focused and relevant full suites are green.

## 20. Final public mental model

```text
Schema form
  -> portable normalized schema AST
  -> validator / explainer / inference / lint projection

Value + schema
  -> plain boolean via valid?
  -> portable Failure tree via explain
  -> completed Result<boolean> via check

Test expression
  -> evaluation Result
  -> comparison Result<boolean>
  -> ordinary fact/batch/report aggregates
```

A Result says whether a completed operation succeeded or failed. A boolean inside a successful Result says whether a comparison matched. Failure trees explain a false comparison. These three concepts remain separate throughout `std.typed` and `code.test`.
