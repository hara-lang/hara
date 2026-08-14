from pathlib import Path
import textwrap


schema_path = Path("core/lib/src/std/typed/schema.hal")
schema = schema_path.read_text()
old_require = "  (:require [std.foundation :refer :all :exclude [resolve]]))"
new_require = """  (:require [std.foundation
             :refer [+ any? assoc boolean boolean? bytes? char? concat conj
                     count empty? every? ex-data ex-info ex-message filter
                     first fn? get get-in has? inc integer? into keys keyword?
                     list? map map? merge name namespace nil? not nth number?
                     reduce rest second set? str string? symbol symbol? true?
                     vec vector?]]))"""
if schema.count(old_require) != 1:
    raise SystemExit("std.typed.schema dependency declaration changed unexpectedly")
schema_path.write_text(schema.replace(old_require, new_require, 1))


infer_path = Path("core/lib/src/std/typed/infer.hal")
infer_path.write_text(
    r'''/PLACEHOLDER'''.replace('/PLACEHOLDER', ''';; Pure, conservative type inference over ordinary Hara forms.
(ns std.typed.infer
  (:config {:blank true})
  (:require [std.foundation
             :refer [any? assoc boolean? char? conj count drop empty? every?
                     filter first get get-in has? integer? keyword? list? map
                     map? name namespace nil? not number? reduce rest second
                     set? str string? symbol symbol? vec vector vector? when]]
            [std.typed.schema :as schema]))

(defn unknown
  ([] (schema/unknown))
  ([surface] (schema/unknown surface)))

(defn unknown?
  [value]
  (schema/unknown? value))

(defn primitive
  [name]
  {:kind :primitive :name name})

(declare infer join)

(defn literal
  "Infers a normalized schema for a literal value."
  [value]
  (cond
    (nil? value) (primitive :nil)
    (boolean? value) (primitive :bool)
    (integer? value) (primitive :int)
    (number? value) (primitive :num)
    (string? value) (primitive :str)
    (char? value) (primitive :char)
    (keyword? value) (primitive :keyword)
    (vector? value)
    {:kind :vector
     :item (join (map literal value))}
    (map? value)
    {:kind :map
     :fields (vec (map (fn [entry]
                         {:name (first entry)
                          :type (literal (second entry))})
                       value))}
    (set? value) (primitive :set)
    (list? value) (primitive :list)
    (symbol? value) (primitive :symbol)
    :else (unknown value)))

(defn append-member
  [output value]
  (if (any? (fn [candidate] (= candidate value)) output)
    output
    (conj output value)))

(defn join
  "Joins concrete schemas into a flattened union. Any unknown makes the join unknown."
  [values]
  (let [types (vec values)]
    (if (or (empty? types) (any? unknown? types))
      (unknown)
      (let [members
            (reduce (fn [output value]
                      (if (= :union (:kind value))
                        (reduce append-member output (:types value))
                        (append-member output value)))
                    [] types)]
        (if (= 1 (count members))
          (first members)
          {:kind :union :types members})))))

(defn qualify-symbol
  [value context]
  (let [prefix (namespace value)]
    (if prefix
      (let [prefix-symbol (symbol prefix)
            target (or (get (:aliases context) prefix-symbol) prefix-symbol)]
        (symbol (str target) (name value)))
      (or (get (:refer-map context) value)
          (when (:namespace context)
            (symbol (str (:namespace context)) (name value)))
          value))))

(defn environment-type
  [value context]
  (or (get (:locals context) value)
      (get (:values context) value)
      (get (:values context) (qualify-symbol value context))
      (unknown value)))

(defn resolve-call-schema
  [value context]
  (or (get (:functions context) value)
      (get (:functions context) (qualify-symbol value context))))

(defn infer-sequence
  [values context]
  (loop [remaining (vec values)
         output (primitive :nil)]
    (if (empty? remaining)
      output
      (recur (vec (rest remaining))
             (infer (first remaining) context)))))

(defn binding-symbol
  [value]
  (if (symbol? value) value nil))

(defn infer-let
  [children context]
  (let [bindings (second children)
        body (vec (drop 2 children))]
    (if (not (vector? bindings))
      (unknown children)
      (let [nested
            (loop [remaining (vec bindings)
                   locals (:locals context)]
              (if (< (count remaining) 2)
                locals
                (let [binding (first remaining)
                      initializer (second remaining)
                      name (binding-symbol binding)
                      inferred (infer initializer (assoc context :locals locals))]
                  (recur (vec (drop 2 remaining))
                         (if name (assoc locals name inferred) locals)))))]
        (infer-sequence body (assoc context :locals nested))))))

(defn numeric-result
  [arguments context divide?]
  (let [types (vec (map (fn [argument]
                          (infer argument context))
                        arguments))]
    (if (or (empty? types) (any? unknown? types))
      (unknown)
      (if (every? (fn [value]
                    (schema/assignable-normal? (primitive :num) value))
                  types)
        (if divide?
          (primitive :num)
          (if (every? (fn [value] (= value (primitive :int))) types)
            (primitive :int)
            (primitive :num)))
        (unknown)))))

(defn infer-known-call
  [head arguments context]
  (let [function-schema (resolve-call-schema head context)
        arity (when function-schema
                (schema/matching-arity function-schema (count arguments)))]
    (if arity
      (:output arity)
      (unknown head))))

(defn infer-list
  [value context]
  (let [children (vec value)
        head (first children)
        arguments (vec (rest children))]
    (cond
      (empty? children) (primitive :list)
      (= head 'quote)
      (if (empty? arguments)
        (unknown value)
        (literal (first arguments)))
      (= head 'do) (infer-sequence arguments context)
      (= head 'if)
      (let [branches (vec (drop 1 arguments))
            branches (if (= 1 (count branches))
                       (conj branches nil)
                       branches)]
        (join (map (fn [branch]
                     (if (nil? branch)
                       (primitive :nil)
                       (infer branch context)))
                   branches)))
      (or (= head 'let) (= head 'loop))
      (infer-let children context)
      (has? '#{+ - * mod %} head)
      (numeric-result arguments context false)
      (= head '/)
      (numeric-result arguments context true)
      (has? '#{= not= < <= > >= identical? instance? nil? some?} head)
      (primitive :bool)
      (= head 'count) (primitive :int)
      (= head 'str) (primitive :str)
      (= head 'keyword) (primitive :keyword)
      (= head 'symbol) (primitive :symbol)
      (= head 'vector)
      {:kind :vector
       :item (join (map (fn [argument]
                          (infer argument context))
                        arguments))}
      (= head 'hash-map)
      {:kind :map
       :fields
       (loop [remaining arguments output []]
         (if (< (count remaining) 2)
           output
           (recur (vec (drop 2 remaining))
                  (conj output
                        {:name (first remaining)
                         :type (infer (second remaining) context)}))))}
      (symbol? head)
      (infer-known-call head arguments context)
      :else (unknown value))))

(defn infer-vector
  [value context]
  {:kind :vector
   :item (join (map (fn [item] (infer item context)) value))})

(defn infer-map
  [value context]
  {:kind :map
   :fields (vec (map (fn [entry]
                       {:name (first entry)
                        :type (infer (second entry) context)})
                     value))})

(defn infer
  "Infers one ordinary Hara form without evaluating it."
  ([value]
   (infer value {}))
  ([value context]
   (cond
     (symbol? value) (environment-type value context)
     (list? value) (infer-list value context)
     (vector? value) (infer-vector value context)
     (map? value) (infer-map value context)
     (set? value) (primitive :set)
     :else (literal value))))

(defn parameter-layout
  [parameters]
  (if (not (vector? parameters))
    {:fixed [] :rest nil}
    (loop [remaining (vec parameters)
           fixed []]
      (if (empty? remaining)
        {:fixed fixed :rest nil}
        (let [value (first remaining)]
          (if (= '& value)
            {:fixed fixed :rest (second remaining)}
            (recur (vec (rest remaining))
                   (conj fixed value))))))))

(defn declared-arity
  [declared layout]
  (first
   (filter
    (fn [arity]
      (and (= (count (:fixed layout))
              (count (get-in arity [:inputs :fixed])))
           (= (nil? (:rest layout))
              (nil? (get-in arity [:inputs :rest])))))
    (schema/project-arities-normal declared))))

(defn infer-function-arity
  [parameters body declared context]
  (let [layout (parameter-layout parameters)
        contract (when declared (declared-arity declared layout))
        fixed-types (if contract
                      (get-in contract [:inputs :fixed])
                      (vec (map (fn [_] (unknown)) (:fixed layout))))
        rest-type (if contract
                    (get-in contract [:inputs :rest])
                    (when (:rest layout) (unknown)))
        locals (reduce (fn [output pair]
                         (assoc output (first pair) (second pair)))
                       (:locals context)
                       (map vector (:fixed layout) fixed-types))
        locals (if (:rest layout)
                 (assoc locals (:rest layout)
                        {:kind :vector :item rest-type})
                 locals)]
    {:kind :fn
     :inputs {:fixed fixed-types :rest rest-type}
     :output (infer-sequence body (assoc context :locals locals))}))
'''))


lint_schema_path = Path("core/lib/src/tool/lint/schema.hal")
lint_schema = lint_schema_path.read_text()
adapter_anchor = "(defn normalized?\n  [value]\n"
adapters = '''(defn infer-block
  "Adapts one recovering std.block node to pure std.typed form inference."
  ([value]
   (infer-block value {}))
  ([value context]
   (cond
     (nil? value) (schema/unknown)
     (= :error (block/type value)) (schema/unknown (block/string value))
     :else (infer/infer (block/value value) context))))

(defn infer-function-arity
  "Adapts lint parameter/body blocks to pure std.typed function inference."
  [parameters body declared context]
  (infer/infer-function-arity
   (block/value parameters)
   (vec (map block/value body))
   declared
   context))

'''
if lint_schema.count(adapter_anchor) != 1:
    raise SystemExit("tool.lint.schema adapter anchor changed unexpectedly")
lint_schema = lint_schema.replace(adapter_anchor, adapters + adapter_anchor, 1)
old_call = "(infer/infer-function-arity (:parameters arity)\n                                            (:body arity)\n                                            declared\n                                            context)"
new_call = "(infer-function-arity (:parameters arity)\n                                      (:body arity)\n                                      declared\n                                      context)"
if lint_schema.count(old_call) != 1:
    raise SystemExit("tool.lint.schema function inference call changed unexpectedly")
lint_schema_path.write_text(lint_schema.replace(old_call, new_call, 1))


analyze_path = Path("core/lib/src/tool/lint/analyze.hal")
analyze = analyze_path.read_text()
require_line = "            [std.typed.infer :as infer]\n"
if analyze.count(require_line) != 1:
    raise SystemExit("tool.lint.analyze std.typed.infer import changed unexpectedly")
analyze = analyze.replace(require_line, "", 1)
if analyze.count("infer/infer-block") < 1:
    raise SystemExit("tool.lint.analyze no longer contains expected inference calls")
analyze_path.write_text(analyze.replace("infer/infer-block", "lint-schema/infer-block"))


infer_test_path = Path("core/lib/test/std/typed/infer_test.hal")
infer_test_path.write_text('''(ns std.typed.infer-test
  (:use code.test)
  (:require [std.typed.infer :as infer]
            [std.typed.schema :as schema]))

^{:refer 'std.typed.infer/infer
  :id 'literal-local-and-branch-inference}
(fact "infers ordinary literals, locals, collections, let and branch joins"
  [(infer/infer 1)
   (infer/infer [1 "two"])
   (infer/infer '(let [x 1] (+ x 2)))
   (infer/infer '(if true 1 "two"))
   (infer/infer '(if true 1))]
  => [{:kind :primitive :name :int}
      {:kind :vector
       :item {:kind :union
              :types [{:kind :primitive :name :int}
                      {:kind :primitive :name :str}]}}
      {:kind :primitive :name :int}
      {:kind :union
       :types [{:kind :primitive :name :int}
               {:kind :primitive :name :str}]}
      {:kind :union
       :types [{:kind :primitive :name :int}
               {:kind :primitive :name :nil}]}])

^{:refer 'std.typed.infer/infer
  :id 'unknown-suppresses-joins}
(fact "propagates unknown through joins and arithmetic"
  [(schema/unknown? (infer/infer '(if true missing 1)))
   (schema/unknown? (infer/infer '(+ missing 1)))]
  => [true true])

^{:refer 'std.typed.infer/infer
  :id 'known-call-results}
(fact "projects return facts from known function contracts"
  (infer/infer
   '(label 1)
   {:namespace 'demo
    :functions {'demo/label (schema/normalize [:fn [:int] :str])}})
  => {:kind :primitive :name :str})

(pr-str (run '[std.typed.infer-test]))
''')


parity_path = Path("core/lib/test/std/typed/parity_test.hal")
parity = parity_path.read_text()
old_infer = "(canonical-string (infer/infer-block (source-form (nth case 2))))"
new_infer = "(canonical-string (infer/infer (source-value (nth case 2))))"
if parity.count(old_infer) != 1:
    raise SystemExit("std.typed parity infer case changed unexpectedly")
parity = parity.replace(old_infer, new_infer, 1)
old_call = """(infer/infer-block
        (source-form (nth case 2))
        {:functions"""
new_call = """(infer/infer
        (source-value (nth case 2))
        {:functions"""
if parity.count(old_call) != 1:
    raise SystemExit("std.typed parity infer-call case changed unexpectedly")
parity_path.write_text(parity.replace(old_call, new_call, 1))


rust_schema_path = Path("core/rust/src/kernel/schema.rs")
rust_schema = rust_schema_path.read_text()
infer_marker = "fn function_arities("
tests_marker = "#[cfg(test)]\nmod tests {\n"
if rust_schema.count(infer_marker) != 1 or rust_schema.count(tests_marker) != 1:
    raise SystemExit("Rust schema module boundaries changed unexpectedly")
infer_at = rust_schema.index(infer_marker)
tests_at = rust_schema.index(tests_marker)
if tests_at <= infer_at:
    raise SystemExit("Rust schema tests precede inference unexpectedly")

rust_inference = rust_schema[infer_at:tests_at].rstrip() + "\n"
wrapped_tests = rust_schema[tests_at + len(tests_marker):]
if not wrapped_tests.endswith("}\n"):
    raise SystemExit("Rust schema test module is not the final item")
rust_tests = textwrap.dedent(wrapped_tests[:-2]).lstrip()
rust_tests = rust_tests.replace(
    '"../../../lib/test/std/typed/parity_corpus.hal"',
    '"../../../../lib/test/std/typed/parity_corpus.hal"',
)

rust_schema_dir = rust_schema_path.with_suffix("")
rust_schema_dir.mkdir(exist_ok=True)
rust_schema_path.write_text(
    rust_schema[:infer_at].rstrip()
    + "\n\n#[path = \"schema/infer.rs\"]\nmod infer;\n"
    + "pub use infer::{infer_function_types, infer_schema};\n\n"
    + "#[cfg(test)]\n#[path = \"schema/tests.rs\"]\nmod tests;\n"
)
rust_inference = rust_inference.replace(
    "super::super::core::form_without_metadata",
    "crate::core::form_without_metadata",
)
(rust_schema_dir / "infer.rs").write_text(
    "//! Conservative, non-evaluating inference for ordinary Hara forms.\n\n"
    "use super::{\n"
    "    assignable_schema, push_unique, resolve_schema, FunctionSchema, SchemaField,\n"
    "    SchemaType,\n"
    "};\n"
    "use crate::kernel::Form;\n"
    "use std::collections::HashMap;\n\n"
    + rust_inference
)
(rust_schema_dir / "tests.rs").write_text(rust_tests)
