from pathlib import Path


Path("core/lib/test/std/typed/schema_test.hal").write_text(r'''(ns std.typed.schema-test
  (:require [std.typed.schema :as schema]))

(defn check
  [id pass]
  {:id id :pass pass})

(pr-str
 [(check
   :variadic-and-multi-arity-annotations
   (and
    (= [{:kind :fn
         :inputs {:fixed [{:kind :primitive :name :str}]
                  :rest {:kind :primitive :name :any}}
         :output {:kind :primitive :name :str}}]
       (schema/project-arities '[:fn [:str & :any] :str]))
    (= 2
       (count
        (schema/project-arities
         '[:function [:fn [:int] :int]
                     [:fn [:str & :any] :str]])))))

  (check
   :typed-map-validation
   (and
    (schema/valid? [:map [:name :str] [:count :int]]
                   {:name "demo" :count 2})
    (not
     (schema/valid? [:map [:name :str] [:count :int]]
                    {:name "demo" :count "two"}))))

  (check
   :collection-finding-path
   (= [1]
      (:finding/path
       (first (schema/validate [:vector :int] [1 "two"])))))

  (check
   :union-and-enum-validation
   (and
    (schema/compatible? [:maybe :int] :nil)
    (schema/valid? [:enum :must :may] :must)
    (not (schema/valid? [:enum :must :may] :never))))

  (check
   :typed-relation-tuple
   (and
    (schema/valid? [:tuple :keyword :int :str] [:age 42 "years"])
    (not
     (schema/valid? [:tuple :keyword :int :str]
                    [:age "42" "years"]))))

  (check
   :strict-canonical-normalization
   (= [{:kind :primitive :name :int}
       {:kind :union
        :types [{:kind :primitive :name :int}
                {:kind :primitive :name :str}]}
       :invalid-arity
       :unsupported-form]
      [(schema/normalize :integer)
       (schema/normalize [:or :int [:or :str :int] :str])
       (get-in (schema/normalize-result [:vector]) [:error :reason])
       (get-in (schema/normalize-result [:vec :any]) [:error :reason])]))

  (check
   :directional-numeric-assignment
   (= [true false true]
      [(schema/assignable? :num :int)
       (schema/assignable? :int :num)
       (schema/compatible? :int :num)]))

  (check
   :cycle-safe-reference-resolution
   (let [node (schema/normalize
               [:map [:next '(var Node)]]
               {:namespace 'demo})
         definitions {'demo/Node node}]
     (= [{:kind :reference :name 'demo/Customer}
         {:kind :map
          :fields [{:name :next
                    :type {:kind :reference :name 'demo/Node}}]}]
        [(schema/normalize '(var Customer) {:namespace 'demo})
         (schema/resolve (schema/normalize '(var demo/Node)) definitions)])))

  (check
   :function-variance
   (= [true false true]
      [(schema/assignable? [:fn [:int] :num]
                           [:fn [:num] :int])
       (schema/assignable? [:fn [:num] :int]
                           [:fn [:int] :num])
       (schema/assignable? '[:fn [:int & :int] :num]
                           '[:fn [:num & :num] :int])]))])
''')


Path("core/lib/test/std/typed/infer_test.hal").write_text(r'''(ns std.typed.infer-test
  (:require [std.typed.infer :as infer]
            [std.typed.schema :as schema]))

(defn check
  [id pass]
  {:id id :pass pass})

(pr-str
 [(check
   :literal-local-and-branch-inference
   (= [{:kind :primitive :name :int}
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
                {:kind :primitive :name :nil}]}]
      [(infer/infer 1)
       (infer/infer [1 "two"])
       (infer/infer '(let [x 1] (+ x 2)))
       (infer/infer '(if true 1 "two"))
       (infer/infer '(if true 1))]))

  (check
   :unknown-suppresses-joins
   (and
    (schema/unknown? (infer/infer '(if true missing 1)))
    (schema/unknown? (infer/infer '(+ missing 1)))))

  (check
   :known-call-results
   (= {:kind :primitive :name :str}
      (infer/infer
       '(label 1)
       {:namespace 'demo
        :functions {'demo/label
                    (schema/normalize [:fn [:int] :str])}})))])
''')


Path("core/lib/test/std/typed/parity_test.hal").write_text(r'''(ns std.typed.parity-test
  (:require [std.typed.infer :as infer]
            [std.typed.parity-corpus :as corpus]
            [std.typed.schema :as schema]))

(defn join-strings
  [separator values]
  (reduce (fn [output value]
            (if (empty? output)
              value
              (str output separator value)))
          ""
          values))

(declare canonical-string)

(defn canonical-arity
  [arity]
  (str "fn(fixed=["
       (join-strings "," (map canonical-string
                                (get-in arity [:inputs :fixed])))
       "],rest="
       (if (get-in arity [:inputs :rest])
         (canonical-string (get-in arity [:inputs :rest]))
         "none")
       ",output=" (canonical-string (:output arity)) ")"))

(defn canonical-string
  [value]
  (let [kind (:kind value)]
    (cond
      (= kind :primitive)
      (str "primitive(" (pr-str (:name value)) ")")

      (= kind :reference)
      (str "reference(" (:name value) ")")

      (= kind :union)
      (str "union["
           (join-strings "," (map canonical-string (:types value)))
           "]")

      (= kind :vector)
      (str "vector(" (canonical-string (:item value)) ")")

      (= kind :tuple)
      (str "tuple["
           (join-strings "," (map canonical-string (:items value)))
           "]")

      (= kind :map)
      (str "map["
           (join-strings
            ","
            (map (fn [field]
                   (str (pr-str (:name field)) "="
                        (canonical-string (:type field))))
                 (:fields value)))
           "]")

      (= kind :fn) (canonical-arity value)

      (= kind :function)
      (str "function["
           (join-strings "," (map canonical-arity (:arities value)))
           "]")

      (= kind :enum)
      (str "enum[" (join-strings "," (map pr-str (:values value))) "]")

      (= kind :unknown) "unknown"
      :else (str "unsupported(" (pr-str value) ")"))))

(defn source-value
  [source]
  (Edn/read source))

(defn run-case
  [case]
  (let [operation (first case)]
    (cond
      (= operation :normalize)
      (canonical-string (schema/normalize (source-value (nth case 2))))

      (= operation :error)
      (get-in (schema/normalize-result (source-value (nth case 2)))
              [:error :reason])

      (= operation :assignable)
      (schema/assignable? (source-value (nth case 2))
                          (source-value (nth case 3)))

      (= operation :compatible)
      (schema/compatible? (source-value (nth case 2))
                          (source-value (nth case 3)))

      (= operation :infer)
      (canonical-string (infer/infer (source-value (nth case 2))))

      (= operation :infer-call)
      (canonical-string
       (infer/infer
        (source-value (nth case 2))
        {:functions {(symbol (nth case 3))
                     (schema/normalize (source-value (nth case 4)))}}))

      :else :unknown-case)))

(defn expected-case
  [case]
  (last case))

(let [actual (vec (map run-case corpus/+cases+))
      expected (vec (map expected-case corpus/+cases+))]
  (pr-str [{:id :shared-runtime-parity-corpus
            :pass (= expected actual)}]))
''')
