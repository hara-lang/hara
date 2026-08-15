from __future__ import annotations

import argparse
from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one anchor, found {count}")
    return text.replace(old, new, 1)


def insert_once(text: str, anchor: str, addition: str, label: str) -> str:
    return replace_once(text, anchor, addition + anchor, label)


FOUNDATION_TIMEOUT = r'''
(defn ^{:schema [:fn [[:result :any]] :bool]}
  res-timeout?
  "Returns true when result is an error Result classified as :result/timeout."
  [result]
  (and (res-error? result)
       (= :result/timeout
          (get (ex-data (res-error-value result)) :code))))

'''


SCHEMA_RESULT_EXTENSION = r'''

;; ---------------------------------------------------------------------------
;; Native Result schemas
;; ---------------------------------------------------------------------------

(def +result-property-names+
  #{:status :error-code})

(defn result-property-error
  [message surface reason]
  (schema-error message surface reason))

(defn normalize-result-properties
  [properties surface]
  (if (not (map? properties))
    (result-property-error ":result schema properties must be a map"
                           surface
                           :invalid-result-properties)
    (let [unknown-properties
          (vec (filter (fn [key]
                         (not (has? +result-property-names+ key)))
                       (keys properties)))
          status-present? (has? properties :status)
          code-present? (has? properties :error-code)
          requested-status (get properties :status)
          error-code (get properties :error-code)]
      (cond
        (not (empty? unknown-properties))
        (result-property-error
         (str "unsupported :result schema properties: " unknown-properties)
         surface
         :unsupported-result-property)

        (and status-present?
             (not (has? #{:success :error} requested-status)))
        (result-property-error
         ":result :status must be :success or :error"
         surface
         :invalid-result-status)

        (and code-present? (not (keyword? error-code)))
        (result-property-error
         ":result :error-code must be a keyword"
         surface
         :invalid-result-error-code)

        (and code-present? (= :success requested-status))
        (result-property-error
         ":result :error-code cannot refine a successful Result"
         surface
         :conflicting-result-properties)

        :else
        (let [status (if code-present? :error requested-status)
              with-status (if status {:status status} {})]
          (if code-present?
            (assoc with-status :error-code error-code)
            with-status))))))

(defn normalize-result-schema
  [surface context]
  (let [arguments (vec (rest surface))]
    (cond
      (= 1 (count arguments))
      {:kind :result
       :properties {}
       :data (normalize-in (first arguments) context)}

      (and (= 2 (count arguments))
           (map? (first arguments)))
      {:kind :result
       :properties (normalize-result-properties (first arguments) surface)
       :data (normalize-in (second arguments) context)}

      :else
      (schema-error
       ":result schema expects a data schema and optional property map"
       surface
       :invalid-result-arity))))

(def +normalize-in-before-result+ normalize-in)

(defn normalize-in
  "Normalizes the portable schema surface, including native Result schemas."
  [schema context]
  (if (and (vector? schema)
           (not (empty? schema))
           (= :result (first schema)))
    (normalize-result-schema schema context)
    (+normalize-in-before-result+ schema context)))

(def +resolve-normal-before-result+ resolve-normal)

(defn resolve-normal
  [schema definitions visited]
  (if (= :result (:kind schema))
    (assoc schema :data (resolve-normal (:data schema) definitions visited))
    (+resolve-normal-before-result+ schema definitions visited)))

(def +reference-names-normal-before-result+ reference-names-normal)

(defn reference-names-normal
  [schema]
  (if (= :result (:kind schema))
    (reference-names-normal (:data schema))
    (+reference-names-normal-before-result+ schema)))

(defn result-error-code
  [value]
  (if (res-error? value)
    (get (ex-data (res-error-value value)) :code)
    nil))

(defn validate-result-normal
  [schema value path]
  (if (not (res? value))
    [(finding path schema value)]
    (let [properties (:properties schema)
          expected-status (get properties :status)
          expected-code (get properties :error-code)
          actual-status (res-status value)
          status-findings
          (if (and expected-status (not= expected-status actual-status))
            [(finding (conj path :status) expected-status actual-status)]
            [])
          code-findings
          (if (and expected-code
                   (not= expected-code (result-error-code value)))
            [(finding (conj path :error :code)
                      expected-code
                      (result-error-code value))]
            [])
          data-findings
          (if (res-success? value)
            (validate-normal (:data schema)
                             (res-data value)
                             (conj path :data))
            [])]
      (vec (concat status-findings code-findings data-findings)))))

(def +validate-normal-before-result+ validate-normal)

(defn validate-normal
  [schema value path]
  (if (= :result (:kind schema))
    (validate-result-normal schema value path)
    (+validate-normal-before-result+ schema value path)))

(defn result-status-domain
  [schema]
  (let [status (get-in schema [:properties :status])]
    (cond
      (= status :success) #{:success}
      (= status :error) #{:error}
      :else #{:success :error})))

(defn result-status-overlap?
  [left right]
  (any? (fn [status]
          (has? (result-status-domain right) status))
        (result-status-domain left)))

(defn result-success-overlap?
  [left right]
  (and (has? (result-status-domain left) :success)
       (has? (result-status-domain right) :success)))

(defn result-error-code-compatible?
  [left right]
  (let [left-code (get-in left [:properties :error-code])
        right-code (get-in right [:properties :error-code])]
    (or (nil? left-code)
        (nil? right-code)
        (= left-code right-code))))

(defn compatible-result?
  [left right]
  (and (result-status-overlap? left right)
       (result-error-code-compatible? left right)
       (or (not (result-success-overlap? left right))
           (compatible-normal? (:data left) (:data right)))))

(def +compatible-normal-before-result+ compatible-normal?)

(defn compatible-normal?
  [expected actual]
  (if (and (= :result (:kind expected))
           (= :result (:kind actual)))
    (compatible-result? expected actual)
    (+compatible-normal-before-result+ expected actual)))

(defn assignable-result?
  [expected actual]
  (let [expected-statuses (result-status-domain expected)
        actual-statuses (result-status-domain actual)
        expected-code (get-in expected [:properties :error-code])
        actual-code (get-in actual [:properties :error-code])]
    (and
     (every? (fn [status] (has? expected-statuses status)) actual-statuses)
     (or (not (has? actual-statuses :success))
         (assignable-normal? (:data expected) (:data actual)))
     (or (nil? expected-code)
         (= expected-code actual-code)))))

(def +assignable-normal-before-result+ assignable-normal?)

(defn assignable-normal?
  [expected actual]
  (if (and (= :result (:kind expected))
           (= :result (:kind actual)))
    (assignable-result? expected actual)
    (+assignable-normal-before-result+ expected actual)))
'''


INFER_RESULT_EXTENSION = r'''

;; ---------------------------------------------------------------------------
;; Native Result transfer and narrowing rules
;; ---------------------------------------------------------------------------

(def +result-transfer-table+
  {'std.foundation/res-success :result/success
   'std.foundation/res-error :result/error
   'std.foundation/res-synchronize :result/synchronize
   'std.foundation/res? :result/result?
   'std.foundation/res-success? :result/success?
   'std.foundation/res-error? :result/error?
   'std.foundation/res-timeout? :result/timeout?
   'std.foundation/res-status :result/status
   'std.foundation/res-data :result/data
   'std.foundation/res-error-value :result/error-value
   'std.foundation/res-context :result/context
   'std.foundation/res-with-context :result/with-context
   'std.foundation/deref :result/deref})

(defn result-schema
  ([data]
   (result-schema data {}))
  ([data properties]
   {:kind :result
    :properties properties
    :data data}))

(defn result-schema?
  [schema]
  (= :result (:kind schema)))

(defn promise-schema?
  [schema]
  (or (= :promise (:kind schema))
      (and (= :primitive (:kind schema))
           (= :promise (:name schema)))))

(defn result-operation
  [head context]
  (if (symbol? head)
    (let [qualified (qualify-symbol head context)
          foundation-name (symbol "std.foundation" (name head))]
      (or (get +result-transfer-table+ qualified)
          (when (nil? (namespace head))
            (get +result-transfer-table+ foundation-name))))
    nil))

(defn result-data-transfer
  [schema]
  (if (result-schema? schema)
    (let [status (get-in schema [:properties :status])]
      (cond
        (= status :success) (:data schema)
        (= status :error) (primitive :nil)
        :else (schema/union-normal [(:data schema) (primitive :nil)])))
    (unknown)))

(defn result-error-value-transfer
  [schema]
  (if (result-schema? schema)
    (if (= :success (get-in schema [:properties :status]))
      (primitive :nil)
      (primitive :any))
    (unknown)))

(defn result-status-transfer
  [schema]
  (if (result-schema? schema)
    (let [status (get-in schema [:properties :status])]
      {:kind :enum
       :values (if status [status] [:success :error])})
    (unknown)))

(defn result-synchronize-transfer
  [schema]
  (cond
    (result-schema? schema) schema
    (= :promise (:kind schema)) (result-schema (or (:item schema) (primitive :any)))
    (promise-schema? schema) (result-schema (primitive :any))
    :else (result-schema schema)))

(defn result-transfer
  "Applies one pure native Result transfer operation to normalized argument schemas."
  [operation argument-schemas]
  (let [first-argument (if (empty? argument-schemas)
                         (unknown)
                         (first argument-schemas))]
    (cond
      (= operation :result/success)
      (result-schema first-argument {:status :success})

      (= operation :result/error)
      (result-schema (primitive :any) {:status :error})

      (= operation :result/synchronize)
      (result-synchronize-transfer first-argument)

      (has? #{:result/result?
              :result/success?
              :result/error?
              :result/timeout?}
            operation)
      (primitive :bool)

      (= operation :result/status)
      (result-status-transfer first-argument)

      (= operation :result/data)
      (result-data-transfer first-argument)

      (= operation :result/error-value)
      (result-error-value-transfer first-argument)

      (= operation :result/context)
      (primitive :map)

      (= operation :result/with-context)
      first-argument

      (= operation :result/deref)
      (if (result-schema? first-argument)
        (:data first-argument)
        (unknown))

      :else
      (unknown))))

(def +infer-known-call-before-result+ infer-known-call)

(defn infer-known-call
  [head arguments context]
  (let [operation (result-operation head context)]
    (if operation
      (result-transfer
       operation
       (vec (map (fn [argument]
                   (infer-block argument context))
                 arguments)))
      (+infer-known-call-before-result+ head arguments context))))

(defn narrow-result-schema
  "Narrows a normalized Result schema for a successful predicate branch."
  [schema operation]
  (let [current (if (result-schema? schema)
                  schema
                  (result-schema (primitive :any)))
        properties (:properties current)]
    (cond
      (= operation :result/result?) current
      (= operation :result/success?)
      (assoc current :properties (assoc properties :status :success))
      (= operation :result/error?)
      (assoc current :properties (assoc properties :status :error))
      (= operation :result/timeout?)
      (assoc current :properties
             (assoc (assoc properties :status :error)
                    :error-code :result/timeout))
      :else current)))

(defn assoc-environment-schema
  [context target schema]
  (if (and (:locals context) (has? (:locals context) target))
    (assoc context :locals (assoc (:locals context) target schema))
    (assoc context :values (assoc (or (:values context) {}) target schema))))

(defn result-condition-context
  [condition context]
  (if (and condition (= :list (block/tag condition)))
    (let [children (code-children condition)
          head-block (first children)
          argument-block (second children)
          head (when head-block (block/value head-block))
          target (when argument-block (block/value argument-block))
          operation (result-operation head context)]
      (if (and (symbol? target)
               (has? #{:result/result?
                       :result/success?
                       :result/error?
                       :result/timeout?}
                     operation))
        (assoc-environment-schema
         context
         target
         (narrow-result-schema (environment-type target context) operation))
        context))
    context))

(defn infer-result-if
  [value context]
  (let [children (code-children value)
        arguments (vec (rest children))
        condition (first arguments)
        then-branch (second arguments)
        else-branch (nth arguments 2 nil)]
    (if (nil? then-branch)
      (unknown value)
      (join
       [(infer-block then-branch (result-condition-context condition context))
        (if else-branch
          (infer-block else-branch context)
          (primitive :nil))]))))

(def +infer-list-before-result+ infer-list)

(defn infer-list
  [value context]
  (let [children (code-children value)
        head-block (first children)
        head (when head-block (block/value head-block))]
    (if (= head 'if)
      (infer-result-if value context)
      (+infer-list-before-result+ value context))))
'''


FOUNDATION_TEST = r'''(ns std.foundation-result-timeout-test
  (:use code.test))

(fact "identifies only :result/timeout error Results"
  (let [timeout (res-error
                 (ex-info "timed out"
                          {:code :result/timeout
                           :timeout 25}))
        unsupported (res-error
                     (ex-info "unsupported"
                              {:code :result/timeout-unsupported}))
        ordinary (res-error
                  (ex-info "boom" {:code :demo/boom}))
        success (res-success 42)]
    [(res-timeout? timeout)
     (res-timeout? unsupported)
     (res-timeout? ordinary)
     (res-timeout? success)])
  => [true false false false])

(pr-str (run '[std.foundation-result-timeout-test]))
'''


SCHEMA_TEST = r'''(ns std.typed.result-schema-test
  (:use code.test)
  (:require [std.typed.schema :as schema]))

(fact "normalizes Result status and timeout error-code refinements"
  [(schema/normalize [:result :int])
   (schema/normalize
    [:result {:status :error
              :error-code :result/timeout}
     :any])]
  => [{:kind :result
       :properties {}
       :data {:kind :primitive :name :int}}
      {:kind :result
       :properties {:status :error
                    :error-code :result/timeout}
       :data {:kind :primitive :name :any}}])

(fact "validates successful data and timeout error classification"
  (let [success (res-success 42)
        wrong-data (res-success "42")
        timeout (res-error
                 (ex-info "timed out"
                          {:code :result/timeout}))
        ordinary-error (res-error
                        (ex-info "boom"
                                 {:code :demo/boom}))]
    [(schema/valid? [:result {:status :success} :int] success)
     (schema/valid? [:result {:status :success} :int] wrong-data)
     (schema/valid? [:result {:status :error
                              :error-code :result/timeout}
                     :any]
                    timeout)
     (schema/valid? [:result {:status :error
                              :error-code :result/timeout}
                     :any]
                    ordinary-error)])
  => [true false true false])

(fact "checks Result compatibility and directional assignment"
  [(schema/compatible?
    [:result {:status :error :error-code :result/timeout} :any]
    [:result {:status :error :error-code :demo/boom} :any])
   (schema/assignable?
    [:result {:status :error} :any]
    [:result {:status :error :error-code :result/timeout} :any])
   (schema/assignable?
    [:result {:status :error :error-code :result/timeout} :any]
    [:result {:status :error} :any])]
  => [false true false])

(fact "rejects conflicting Result properties"
  (get-in
   (schema/normalize-result
    [:result {:status :success
              :error-code :result/timeout}
     :any])
   [:error :reason])
  => :conflicting-result-properties)

(pr-str (run '[std.typed.result-schema-test]))
'''


INFER_TEST = r'''(ns std.typed.result-infer-test
  (:use code.test)
  (:require [std.typed.infer :as infer]
            [std.typed.schema :as schema]))

(def +int+ (schema/normalize :int))
(def +result-int+ (schema/normalize [:result :int]))
(def +timeout-result+
  (schema/normalize
   [:result {:status :error
             :error-code :result/timeout}
    :int]))

(fact "transfers Result constructors and synchronization without flattening"
  [(infer/result-transfer :result/success [+int+])
   (infer/result-transfer :result/synchronize [+result-int+])
   (infer/result-transfer
    :result/synchronize
    [{:kind :promise :item +result-int+}])]
  => [{:kind :result
       :properties {:status :success}
       :data {:kind :primitive :name :int}}
      +result-int+
      {:kind :result
       :properties {}
       :data +result-int+}])

(fact "refines successful, error, and timeout predicate branches"
  [(get-in (infer/narrow-result-schema +result-int+ :result/success?)
           [:properties :status])
   (get-in (infer/narrow-result-schema +result-int+ :result/error?)
           [:properties :status])
   (:properties
    (infer/narrow-result-schema +result-int+ :result/timeout?))]
  => [:success
      :error
      {:status :error
       :error-code :result/timeout}])

(fact "refines Result accessors from known status"
  [(infer/result-transfer
    :result/data
    [(infer/narrow-result-schema +result-int+ :result/success?)])
   (infer/result-transfer :result/data [+timeout-result+])
   (infer/result-transfer :result/status [+timeout-result+])]
  => [{:kind :primitive :name :int}
      {:kind :primitive :name :nil}
      {:kind :enum :values [:error]}])

(pr-str (run '[std.typed.result-infer-test]))
'''


def transform_foundation(text: str) -> str:
    if "  res-timeout?\n" in text:
        return text
    marker = ";; ---------------------------------------------------------------------------\n;; Functions and composition\n;; ---------------------------------------------------------------------------\n"
    return insert_once(text, marker, FOUNDATION_TIMEOUT, "Foundation timeout predicate")


def transform_schema(text: str) -> str:
    if "+result-property-names+" in text:
        return text
    return text.rstrip() + "\n" + SCHEMA_RESULT_EXTENSION.strip() + "\n"


def transform_infer(text: str) -> str:
    if "+result-transfer-table+" in text:
        return text
    return text.rstrip() + "\n" + INFER_RESULT_EXTENSION.strip() + "\n"


def candidates() -> dict[str, str]:
    return {
        "foundation.hal": transform_foundation(
            Path("core/lib/src/std/foundation.hal").read_text()
        ),
        "schema.hal": transform_schema(
            Path("core/lib/src/std/typed/schema.hal").read_text()
        ),
        "infer.hal": transform_infer(
            Path("core/lib/src/std/typed/infer.hal").read_text()
        ),
    }


def write_candidates() -> None:
    root = Path("core/target/result-timeout-typing")
    root.mkdir(parents=True, exist_ok=True)
    for name, content in candidates().items():
        (root / name).write_text(content)
    print(root)


def apply() -> None:
    generated = candidates()
    source_targets = {
        "foundation.hal": [
            Path("core/lib/src/std/foundation.hal"),
            Path("core/rust/hal-src/std/foundation.hal"),
        ],
        "schema.hal": [
            Path("core/lib/src/std/typed/schema.hal"),
            Path("core/rust/hal-src/std/typed/schema.hal"),
        ],
        "infer.hal": [
            Path("core/lib/src/std/typed/infer.hal"),
            Path("core/rust/hal-src/std/typed/infer.hal"),
        ],
    }
    for name, targets in source_targets.items():
        for target in targets:
            target.write_text(generated[name])

    Path("core/lib/test/std/foundation_result_timeout_test.hal").write_text(
        FOUNDATION_TEST
    )
    Path("core/lib/test/std/typed/result_schema_test.hal").write_text(SCHEMA_TEST)
    Path("core/lib/test/std/typed/result_infer_test.hal").write_text(INFER_TEST)
    print("timeout-aware Result typing applied")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=("candidate", "apply"))
    arguments = parser.parse_args()
    if arguments.command == "candidate":
        write_candidates()
    else:
        apply()


if __name__ == "__main__":
    main()
