from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]


def read(path):
    return (ROOT / path).read_text()


def write(path, content):
    (ROOT / path).write_text(content)


def replace_once(content, old, new, label):
    if old not in content:
        raise SystemExit(f"missing replacement marker: {label}")
    return content.replace(old, new, 1)


def regex_once(content, pattern, replacement, label):
    updated, count = re.subn(pattern, replacement, content, count=1, flags=re.S)
    if count != 1:
        raise SystemExit(f"expected one regex replacement for {label}, got {count}")
    return updated


# ---------------------------------------------------------------------------
# std.typed.schema: generic properties become first-class canonical schema data.
# ---------------------------------------------------------------------------
path = "core/lib/src/std/typed/schema.hal"
content = read(path)

content = regex_once(
    content,
    r"\(defn- normalize-field.*?(?=\(defn- canonical-normal\?)",
    r'''(defn- normalize-properties
  [kind properties allowed]
  (let [unknown
        (vec
         (filter (fn [key] (not (has? allowed key)))
                 (keys properties)))]
    (if (empty? unknown)
      properties
      (throw
       (ex-info
        (str "unsupported " kind " schema property")
        {:kind kind :properties unknown})))))

(defn- assoc-properties
  [schema properties]
  (if (empty? properties)
    schema
    (assoc schema :properties properties)))

(defn- normalize-field
  [field]
  (cond
    (map? field)
    (let [properties
          (normalize-properties
           :map-entry
           (or (:properties field) {})
           #{:optional})]
      (assoc-properties
       {:name (:name field)
        :type (normalize (:type field))}
       properties))

    (and (vector? field) (= 2 (count field)))
    {:name (first field)
     :type (normalize (second field))}

    (and (vector? field)
         (= 3 (count field))
         (map? (second field)))
    (assoc-properties
     {:name (first field)
      :type (normalize (nth field 2))}
     (normalize-properties :map-entry (second field) #{:optional}))

    :else
    (throw
     (ex-info
      "map schema entries must be [key schema] or [key properties schema]"
      {:entry field}))))

(defn- properties-normal?
  [schema]
  (or (not (has? schema :properties))
      (map? (:properties schema))))

''',
    "property-aware map fields",
)

content = replace_once(
    content,
    '''       (= kind :primitive)
       (and (has? schema :name)
            (keyword? (:name schema)))''',
    '''       (= kind :primitive)
       (and (has? schema :name)
            (keyword? (:name schema))
            (properties-normal? schema))''',
    "canonical primitive properties",
)
content = replace_once(
    content,
    '''       (= kind :vector)
       (canonical-normal? (:item schema))''',
    '''       (= kind :vector)
       (and (properties-normal? schema)
            (canonical-normal? (:item schema)))

       (= kind :set)
       (and (properties-normal? schema)
            (canonical-normal? (:item schema)))''',
    "canonical vector and set properties",
)
content = replace_once(
    content,
    '''       (= kind :map)
       (and
        (vector? (:fields schema))
        (every?
         (fn [field]
           (and (map? field)
                (has? field :name)
                (canonical-normal? (:type field))))
         (:fields schema)))''',
    '''       (= kind :map)
       (and
        (properties-normal? schema)
        (vector? (:fields schema))
        (every?
         (fn [field]
           (and (map? field)
                (has? field :name)
                (properties-normal? field)
                (canonical-normal? (:type field))))
         (:fields schema)))''',
    "canonical map properties",
)
content = replace_once(
    content,
    '''        (= kind :primitive)
        (normalize
         (if (has? schema :name)
           (:name schema)
           (first children)))''',
    '''        (= kind :primitive)
        (assoc-properties
         (normalize
          (if (has? schema :name)
            (:name schema)
            (first children)))
         (or (:properties schema) {}))''',
    "normalized primitive properties",
)
content = replace_once(
    content,
    '''        (= kind :vector)
        {:kind :vector
         :item (normalize
                (if (has? schema :item)
                  (:item schema)
                  (first children)))}''',
    '''        (= kind :vector)
        (assoc-properties
         {:kind :vector
          :item (normalize
                 (if (has? schema :item)
                   (:item schema)
                   (first children)))}
         (or (:properties schema) {}))

        (= kind :set)
        (assoc-properties
         {:kind :set
          :item (normalize
                 (if (has? schema :item)
                   (:item schema)
                   (first children)))}
         (or (:properties schema) {}))''',
    "normalized vector and set properties",
)
content = replace_once(
    content,
    '''        (= kind :map)
        {:kind :map
         :fields
         (vec
          (map normalize-field
               (if (has? schema :fields)
                 (:fields schema)
                 children)))}''',
    '''        (= kind :map)
        (assoc-properties
         {:kind :map
          :fields
          (vec
           (map normalize-field
                (if (has? schema :fields)
                  (:fields schema)
                  children)))}
         (or (:properties schema) {}))''',
    "normalized map properties",
)

content = replace_once(
    content,
    '''(defmethod normalize :vector [schema]
  {:kind :vector
   :item (normalize (second schema))})''',
    '''(defmethod normalize :vector [schema]
  (let [arguments (vec (rest schema))
        property-form? (and (not (empty? arguments))
                            (map? (first arguments)))
        properties
        (normalize-properties
         :vector
         (if property-form? (first arguments) {})
         #{:min-count :max-count :distinct})
        item
        (if property-form?
          (if (> (count arguments) 1) (second arguments) :any)
          (if (empty? arguments) :any (first arguments)))]
    (assoc-properties
     {:kind :vector :item (normalize item)}
     properties)))

(defmethod normalize :set [schema]
  (let [arguments (vec (rest schema))
        property-form? (and (not (empty? arguments))
                            (map? (first arguments)))
        properties
        (normalize-properties
         :set
         (if property-form? (first arguments) {})
         #{:min-count :max-count :distinct})
        item
        (if property-form?
          (if (> (count arguments) 1) (second arguments) :any)
          (if (empty? arguments) :any (first arguments)))]
    (assoc-properties
     {:kind :set :item (normalize item)}
     properties)))''',
    "surface vector and set properties",
)
content = replace_once(
    content,
    '''(defmethod normalize :map [schema]
  {:kind :map
   :fields (vec (map normalize-field (rest schema)))})''',
    '''(defmethod normalize :map [schema]
  (let [arguments (vec (rest schema))
        property-form? (and (not (empty? arguments))
                            (map? (first arguments)))
        properties
        (normalize-properties
         :map
         (if property-form? (first arguments) {})
         #{:closed :min-count :max-count})
        fields (if property-form? (rest arguments) arguments)]
    (assoc-properties
     {:kind :map
      :fields (vec (map normalize-field fields))}
     properties)))''',
    "surface map properties",
)
primitive_methods = r'''(defn- normalize-primitive-properties
  [schema]
  (if (and (= 2 (count schema))
           (map? (second schema)))
    (assoc-properties
     {:kind :primitive :name (first schema)}
     (normalize-properties
      (first schema)
      (second schema)
      #{:min-count :max-count :pattern :qualified :distinct}))
    (throw
     (ex-info
      "primitive schema properties require one property map"
      {:schema schema}))))

(defmethod normalize :str [schema] (normalize-primitive-properties schema))
(defmethod normalize :string [schema] (normalize-primitive-properties schema))
(defmethod normalize :keyword [schema] (normalize-primitive-properties schema))
(defmethod normalize :symbol [schema] (normalize-primitive-properties schema))
(defmethod normalize :list [schema] (normalize-primitive-properties schema))
(defmethod normalize :bytes [schema] (normalize-primitive-properties schema))
(defmethod normalize :int [schema] (normalize-primitive-properties schema))
(defmethod normalize :integer [schema] (normalize-primitive-properties schema))
(defmethod normalize :num [schema] (normalize-primitive-properties schema))
(defmethod normalize :number [schema] (normalize-primitive-properties schema))
(defmethod normalize :any [schema] (normalize-primitive-properties schema))

'''
content = replace_once(
    content,
    "(defmethod normalize :tuple [schema]\n  {:kind :tuple\n   :items (vec (map normalize (rest schema)))})\n\n",
    "(defmethod normalize :tuple [schema]\n  {:kind :tuple\n   :items (vec (map normalize (rest schema)))})\n\n" + primitive_methods,
    "primitive property normalizers",
)
content = replace_once(
    content,
    '''      (= kind :vector)
      (reference-names-normal (:item schema))''',
    '''      (or (= kind :vector) (= kind :set))
      (reference-names-normal (:item schema))''',
    "set reference names",
)
content = replace_once(
    content,
    '''      (= kind :vector)
      (assoc schema :item
             (resolve-normal (:item schema) registry-value visited))''',
    '''      (or (= kind :vector) (= kind :set))
      (assoc schema :item
             (resolve-normal (:item schema) registry-value visited))''',
    "set recursive resolution",
)

property_helpers = r'''(defn- property-finding
  [path expected value constraint expected-value]
  {:finding/type :std.typed.schema/property-violation
   :finding/path (vec path)
   :finding/expected expected
   :finding/value value
   :finding/context
   {:constraint constraint
    :expected expected-value}})

(defn- missing-key-finding
  [path field]
  (let [key (:name field)]
    {:finding/type :std.typed.schema/missing-key
     :finding/path (conj (vec path) key)
     :finding/expected (:type field)
     :finding/value nil
     :finding/context
     {:present? false
      :constraint :required
      :key key
      :parent-path (vec path)}}))

(defn- closed-key-finding
  [path expected key value]
  {:finding/type :std.typed.schema/closed-map-key
   :finding/path (conj (vec path) key)
   :finding/expected expected
   :finding/value value
   :finding/context {:constraint :closed :key key}})

(defn- qualified-keyword?
  [value]
  (and (keyword? value)
       (str/includes? (str value) "/")))

(defn- property-count
  [value]
  (cond
    (string? value) (str/length value)
    (or (vector? value) (set? value) (map? value) (list? value))
    (count value)
    :else nil))

(defn- distinct-values?
  [value]
  (if (or (vector? value) (list? value))
    (apply distinct? (vec value))
    true))

(defn- validate-properties
  [schema value path]
  (let [properties (or (:properties schema) {})
        count-value (property-count value)
        min-count (:min-count properties)
        max-count (:max-count properties)
        pattern (:pattern properties)]
    (vec
     (concat
      (if (and min-count count-value (< count-value min-count))
        [(property-finding path schema value :min-count min-count)]
        [])
      (if (and max-count count-value (> count-value max-count))
        [(property-finding path schema value :max-count max-count)]
        [])
      (if (and pattern
               (string? value)
               (not (re-matches pattern value)))
        [(property-finding path schema value :pattern pattern)]
        [])
      (if (and (= true (:qualified properties))
               (not (qualified-keyword? value)))
        [(property-finding path schema value :qualified true)]
        [])
      (if (and (= true (:distinct properties))
               (not (distinct-values? value)))
        [(property-finding path schema value :distinct true)]
        [])))))

(defn- field-optional?
  [field]
  (= true (:optional (:properties field))))

(defn- known-field?
  [schema key]
  (any? (fn [field] (= key (:name field))) (:fields schema)))

(defn- closed-map-findings
  [schema value path]
  (if (= true (:closed (:properties schema)))
    (vec
     (mapcat
      (fn [key]
        (if (known-field? schema key)
          []
          [(closed-key-finding path schema key (get value key))]))
      (sort-by str (keys value))))
    []))

'''
content = replace_once(
    content,
    "(defn- finding [path expected value]\n  {:finding/type :std.typed.schema/invalid-value\n   :finding/path (vec path)\n   :finding/expected expected\n   :finding/value value})\n\n",
    "(defn- finding [path expected value]\n  {:finding/type :std.typed.schema/invalid-value\n   :finding/path (vec path)\n   :finding/expected expected\n   :finding/value value})\n\n" + property_helpers,
    "property validation helpers",
)
content = replace_once(
    content,
    '''(defmethod validate-normal :primitive [schema value path]
  (if (primitive-valid? (:name schema) value)
    []
    [(finding path schema value)]))''',
    '''(defmethod validate-normal :primitive [schema value path]
  (if (primitive-valid? (:name schema) value)
    (validate-properties schema value path)
    [(finding path schema value)]))''',
    "primitive property validation",
)
content = regex_once(
    content,
    r"\(defmethod validate-normal :vector.*?(?=\(defmethod validate-normal :tuple)",
    r'''(defmethod validate-normal :vector [schema value path]
  (if (vector? value)
    (loop [index 0
           output (validate-properties schema value path)]
      (if (= index (count value))
        output
        (recur
         (inc index)
         (vec
          (concat output
                  (validate-normal (:item schema)
                                   (nth value index)
                                   (conj path index)))))))
    [(finding path schema value)]))

(defmethod validate-normal :set [schema value path]
  (if (set? value)
    (let [values (vec (sort-by str value))]
      (loop [index 0
             output (validate-properties schema value path)]
        (if (= index (count values))
          output
          (recur
           (inc index)
           (vec
            (concat output
                    (validate-normal (:item schema)
                                     (nth values index)
                                     (conj path index))))))))
    [(finding path schema value)]))

''',
    "collection property validation",
)
content = regex_once(
    content,
    r"\(defmethod validate-normal :map.*?(?=\(defn- reference-finding)",
    r'''(defmethod validate-normal :map [schema value path]
  (if (map? value)
    (let [field-findings
          (loop [remaining (:fields schema)
                 output []]
            (if (empty? remaining)
              output
              (let [field (first remaining)
                    key (:name field)]
                (recur
                 (rest remaining)
                 (cond
                   (has? value key)
                   (vec
                    (concat output
                            (validate-normal (:type field)
                                             (get value key)
                                             (conj path key))))
                   (field-optional? field) output
                   :else (conj output (missing-key-finding path field)))))))]
      (vec
       (concat
        (validate-properties schema value path)
        field-findings
        (closed-map-findings schema value path))))
    [(finding path schema value)]))

''',
    "map property validation",
)
content = regex_once(
    content,
    r"\(defn- validate-vector-with.*?(?=\(defn- validate-tuple-with)",
    r'''(defn- validate-vector-with
  [schema value path registry-value trail]
  (if (vector? value)
    (loop [index 0
           output (validate-properties schema value path)]
      (if (= index (count value))
        output
        (recur
         (inc index)
         (vec
          (concat output
                  (validate-normal-with (:item schema)
                                        (nth value index)
                                        (conj path index)
                                        registry-value
                                        trail))))))
    [(finding path schema value)]))

(defn- validate-set-with
  [schema value path registry-value trail]
  (if (set? value)
    (let [values (vec (sort-by str value))]
      (loop [index 0
             output (validate-properties schema value path)]
        (if (= index (count values))
          output
          (recur
           (inc index)
           (vec
            (concat output
                    (validate-normal-with (:item schema)
                                          (nth values index)
                                          (conj path index)
                                          registry-value
                                          trail)))))))
    [(finding path schema value)]))

''',
    "registry collection properties",
)
content = regex_once(
    content,
    r"\(defn- validate-map-with.*?(?=\(defn- validate-reference-with)",
    r'''(defn- validate-map-with
  [schema value path registry-value trail]
  (if (map? value)
    (let [field-findings
          (loop [remaining (:fields schema)
                 output []]
            (if (empty? remaining)
              output
              (let [field (first remaining)
                    key (:name field)]
                (recur
                 (rest remaining)
                 (cond
                   (has? value key)
                   (vec
                    (concat output
                            (validate-normal-with (:type field)
                                                  (get value key)
                                                  (conj path key)
                                                  registry-value
                                                  trail)))
                   (field-optional? field) output
                   :else (conj output (missing-key-finding path field)))))))]
      (vec
       (concat
        (validate-properties schema value path)
        field-findings
        (closed-map-findings schema value path))))
    [(finding path schema value)]))

''',
    "registry map properties",
)
content = replace_once(
    content,
    '''      (= kind :vector)
      (validate-vector-with schema value path registry-value trail)

      (= kind :tuple)''',
    '''      (= kind :vector)
      (validate-vector-with schema value path registry-value trail)

      (= kind :set)
      (validate-set-with schema value path registry-value trail)

      (= kind :tuple)''',
    "registry set dispatch",
)
content = replace_once(
    content,
    '''    (and (= :primitive (:kind expected))
         (= :any (:name actual))) true
    (= :unknown (:kind expected)) true''',
    '''    (and (= :primitive (:kind expected))
         (= :any (:name actual))) true
    (and (= :primitive (:kind expected))
         (= :primitive (:kind actual))
         (= (:name expected) (:name actual))) true
    (and (= :vector (:kind expected))
         (= :vector (:kind actual)))
    (compatible-normal? (:item expected) (:item actual))
    (and (= :set (:kind expected))
         (= :set (:kind actual)))
    (compatible-normal? (:item expected) (:item actual))
    (= :unknown (:kind expected)) true''',
    "property-compatible base types",
)
write(path, content)

# Undo the temporary side-effect namespace candidate; core schema owns grammar.
path = "core/lib/src/std/typed.hal"
content = read(path).replace("            [std.typed.properties]\n", "")
write(path, content)
properties_path = ROOT / "core/lib/src/std/typed/properties.hal"
if properties_path.exists():
    properties_path.unlink()

# ---------------------------------------------------------------------------
# tool.metaspec.schema: compile generic semantics to ordinary std.typed forms.
# ---------------------------------------------------------------------------
path = "core/lib/src/tool/metaspec/schema.hal"
content = read(path)
content = regex_once(
    content,
    r"\(defn qualified-keyword\?.*?(?=\(defn \^\{:schema \[:fn \[:map\] :map\]\}\n  declaration-catalog)",
    r'''(defn finding
  [status rule requirement path message repair]
  {:finding/id rule
   :rule/id rule
   :requirement/id requirement
   :finding/status status
   :finding/level :error
   :finding/path (vec path)
   :finding/message message
   :finding/repair repair})

(defn schema-finding
  [status path message repair]
  (finding status
           :tool.metaspec.rule/schema-validation
           :tool.metaspec/generated-document-conformance
           path message repair))

(defn- member?
  [value values]
  (any? (fn [candidate] (= candidate value)) values))

(defn- primitive-schema
  [type]
  (cond
    (= type :boolean) :bool
    (= type :number) :num
    (= type :integer) :int
    (= type :string) :str
    :else type))

(defn- primitive-type?
  [type]
  (member? type
           [:any :nil :bool :boolean :num :number :int :integer
            :str :string :keyword :symbol :list :vector :map :set :bytes]))

(defmethod std.typed.schema/normalize :tool.metaspec/unsupported-type
  [surface]
  {:kind :tool.metaspec/unsupported-type
   :type (second surface)})

(defmethod std.typed.schema/validate-normal :tool.metaspec/unsupported-type
  [schema value path]
  [{:finding/type :std.typed.schema/invalid-value
    :finding/path (vec path)
    :finding/expected schema
    :finding/value value}])

''',
    "remove generic metaspec validators",
)
content = regex_once(
    content,
    r"\(defn- refinement-options.*?(?=\(defn- compile-declaration)",
    r'''(defn- assoc-present
  [output key value]
  (if (nil? value) output (assoc output key value)))

(defn- schema-properties
  [declaration schema-type]
  (let [qualified
        (or (= schema-type :qualified-keyword)
            (= :qualified (:schema/constraint declaration)))
        min-count (or (:schema/min-count declaration)
                      (:schema/min-length declaration))
        max-count (or (:schema/max-count declaration)
                      (:schema/max-length declaration))
        pattern
        (if (= schema-type :version)
          (or (:schema/pattern declaration)
              "^[0-9]+\\.[0-9]+\\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?$")
          (:schema/pattern declaration))]
    (let [output {}
          output (if qualified (assoc output :qualified true) output)
          output (assoc-present output :min-count min-count)
          output (assoc-present output :max-count max-count)
          output (assoc-present output :pattern pattern)]
      (if (= true (:schema/unique declaration))
        (assoc output :distinct true)
        output))))

(defn- with-properties
  [base properties]
  (if (empty? properties) base [base properties]))

(defn- collection-form
  [head properties item]
  (if (empty? properties) [head item] [head properties item]))

''',
    "metaspec property compiler helpers",
)
content = regex_once(
    content,
    r"\(defn- compile-declaration.*?(?=\(defn \^\{:schema \[:fn \[:map :map\] :any\]\}\n  compile-schema)",
    r'''(defn- compile-declaration
  [declaration]
  (if-let [reference (:schema/ref declaration)]
    (reference-form reference)
    (let [schema-type (:schema/type declaration)
          properties (schema-properties declaration schema-type)]
      (cond
        (= schema-type :enum)
        (vec (concat [:enum] (or (:schema/values declaration) [])))

        (= schema-type :map)
        (let [required (vec (or (:schema/required declaration) []))
              root-properties
              (if (= true (:schema/closed declaration))
                (assoc properties :closed true)
                properties)
              fields
              (vec
               (map
                (fn [entry]
                  (let [key (key entry)
                        child (compile-declaration (val entry))]
                    (if (member? key required)
                      [key child]
                      [key {:optional true} child])))
                (or (:schema/properties declaration) {})))]
          (vec
           (concat [:map]
                   (if (empty? root-properties) [] [root-properties])
                   fields)))

        (has? declaration :schema/items)
        (let [item (:schema/items declaration)
              item-schema
              (if (map? item)
                (compile-declaration item)
                (reference-form item))]
          (cond
            (= schema-type :vector) (collection-form :vector properties item-schema)
            (= schema-type :set) (collection-form :set properties item-schema)
            :else [:or
                   (collection-form :vector properties item-schema)
                   (collection-form :set properties item-schema)]))

        (= schema-type :vector) (collection-form :vector properties :any)
        (= schema-type :set) (collection-form :set properties :any)
        (= schema-type :qualified-keyword)
        (with-properties :keyword (assoc properties :qualified true))
        (= schema-type :version) (with-properties :str properties)
        (nil? schema-type) (with-properties :any properties)
        (primitive-type? schema-type)
        (with-properties (primitive-schema schema-type) properties)
        :else [:tool.metaspec/unsupported-type schema-type]))))

''',
    "metaspec portable compiler",
)
content = replace_once(
    content,
    "    (cond\n      (= finding-type :std.typed.schema/invalid-value)",
    '''    (cond
      (= finding-type :std.typed.schema/missing-key)
      (let [context (:finding/context finding)
            parent-path (:parent-path context)
            key (:key context)]
        (schema-finding
         :fail parent-path
         (str "Missing required key: " key)
         {:action/type :add-key :action/path parent-path :action/key key}))

      (= finding-type :std.typed.schema/closed-map-key)
      (schema-finding
       :fail path
       (str "Key is not allowed by closed schema: "
            (:key (:finding/context finding)))
       {:action/type :remove-key :action/path path})

      (= finding-type :std.typed.schema/property-violation)
      (let [context (:finding/context finding)
            constraint (:constraint context)
            expected (:expected context)
            value (:finding/value finding)]
        (cond
          (= constraint :qualified)
          (schema-finding :fail path
                          (str "Expected qualified keyword, got " value)
                          {:action/type :qualify-value :action/path path})
          (= constraint :min-count)
          (schema-finding
           :fail path (str "Value has fewer than " expected " members")
           (if (string? value)
             {:action/type :replace-value :action/path path :action/min-length expected}
             {:action/type :add-members :action/path path :action/min-count expected}))
          (= constraint :max-count)
          (schema-finding
           :fail path (str "Value has more than " expected " members")
           (if (string? value)
             {:action/type :replace-value :action/path path :action/max-length expected}
             {:action/type :remove-members :action/path path :action/max-count expected}))
          (= constraint :distinct)
          (schema-finding :fail path "Collection members must be unique"
                          {:action/type :remove-duplicates :action/path path})
          (= constraint :pattern)
          (schema-finding
           :fail path (str "Value does not match pattern: " value)
           {:action/type :replace-value :action/path path
            :action/expected
            (if (= expected
                   "^[0-9]+\\.[0-9]+\\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?$")
              :semantic-version expected)})
          :else
          (schema-finding :fail path "Value violates schema property"
                          {:action/type :replace-value :action/path path
                           :action/expected expected})))

      (= finding-type :std.typed.schema/invalid-value)''',
    "metaspec typed property repair translation",
)
content = content.replace("      (= kind :tool.metaspec/map) :map\n", "")
content = content.replace("      (= kind :tool.metaspec/set) :set\n", "")
content = content.replace("      (= kind :tool.metaspec/items) :collection\n", "")
content = content.replace("      (= kind :tool.metaspec/refine) (expected-type (:base expected))\n", "")
write(path, content)

# ---------------------------------------------------------------------------
# Focused tests.
# ---------------------------------------------------------------------------
write("core/lib/test/std/typed/properties_test.hal", r'''(ns std.typed.properties-test
  (:use code.test)
  (:require [std.typed :as typed]
            [std.typed.registry :as registry]
            [std.typed.schema :as schema]))

^{:refer 'std.typed.schema/normalize :id 'canonical-property-ast}
(fact "normalizes properties into ordinary canonical std.typed kinds"
  [(schema/normalize [:str {:min-count 1}])
   (schema/normalize [:vector {:distinct true} :int])
   (:kind (schema/normalize [:set :keyword]))
   (:kind (schema/normalize
           [:map {:closed true}
            [:id :int]
            [:nickname {:optional true} :str]]))]
  => [{:kind :primitive :name :str :properties {:min-count 1}}
      {:kind :vector :item {:kind :primitive :name :int}
       :properties {:distinct true}}
      :set :map])

^{:refer 'std.typed.schema/valid? :id 'primitive-properties}
(fact "validates string and qualified-keyword properties"
  [(typed/valid? [:str {:min-count 2 :max-count 4 :pattern "^a.*$"}] "ab")
   (typed/valid? [:str {:min-count 2 :max-count 4 :pattern "^a.*$"}] "b")
   (typed/valid? [:keyword {:qualified true}] :demo/value)
   (typed/valid? [:keyword {:qualified true}] :value)]
  => [true false true false])

^{:refer 'std.typed.schema/valid? :id 'collection-properties}
(fact "validates vector bounds, distinctness, and typed sets"
  [(typed/valid? [:vector {:min-count 1 :max-count 3 :distinct true} :int] [1 2 3])
   (typed/valid? [:vector {:min-count 1 :max-count 3 :distinct true} :int] [1 1])
   (typed/valid? [:set :keyword] #{:a :b})
   (typed/valid? [:set :keyword] #{:a 2})]
  => [true false true false])

^{:refer 'std.typed.schema/validate :id 'map-properties}
(fact "validates optional entries and closed maps with exact typed findings"
  (let [contract [:map {:closed true}
                  [:id :int]
                  [:nickname {:optional true} :str]]
        missing (first (typed/validate contract {}))
        extra (first (typed/validate contract {:id 1 :extra true}))]
    [[(:finding/type missing) (:finding/path missing)
      (:present? (:finding/context missing))]
     [(:finding/type extra) (:finding/path extra)]
     (typed/valid? contract {:id 1})
     (typed/valid? contract {:id 1 :nickname "Ada"})])
  => [[:std.typed.schema/missing-key [:id] false]
      [:std.typed.schema/closed-map-key [:extra]] true true])

^{:refer 'std.typed.schema/valid? :id 'registry-property-traversal}
(fact "keeps references executable inside property-aware collections"
  (let [registry-value
        (registry/registry
         {'demo/Name [:str {:min-count 1}]
          'demo/User
          '[:map {:closed true}
            [:name (var demo/Name)]
            [:tags {:optional true} [:set :keyword]]]})]
    [(typed/valid? 'demo/User {:name "Ada"} registry-value)
     (typed/valid? 'demo/User {:name ""} registry-value)
     (typed/valid? 'demo/User {:name "Ada" :tags #{:a :b}} registry-value)])
  => [true false true])

(pr-str (run '[std.typed.properties-test]))
''')

path = "core/lib/test/tool/metaspec/schema_test.hal"
content = read(path)
content = replace_once(
    content,
    '''(defn- property?
  [normalized key]
  (any? (fn [property]
          (= key (:name property)))
        (:properties normalized)))''',
    '''(defn- field?
  [normalized key]
  (any? (fn [field] (= key (:name field))) (:fields normalized)))

(defn- required-fields
  [normalized]
  (vec
   (map :name
        (filter (fn [field]
                  (not (= true (:optional (:properties field)))))
                (:fields normalized)))))''',
    "metaspec field helpers",
)
content = replace_once(
    content,
    '''    [(:kind normalized)
     (:required normalized)
     (count (:properties normalized))
     (property? normalized :demo/name)
     (property? normalized :demo/status)
     (property? normalized :demo/ignored)
     (registry/names registry-value)])
  => [:tool.metaspec/map
      [:demo/name :demo/status]
      3
      true
      true
      true
      '[demo/document demo/name demo/status]])''',
    '''    [(:kind normalized)
     (required-fields normalized)
     (count (:fields normalized))
     (field? normalized :demo/name)
     (field? normalized :demo/status)
     (field? normalized :demo/ignored)
     (registry/names registry-value)])
  => [:map [:demo/name :demo/status] 3 true true true
      '[demo/document demo/name demo/status]])''',
    "metaspec canonical map expectation",
)
content = content.replace(
    'fact "keeps metaspec refinements as namespaced std.typed extensions"',
    'fact "lowers metaspec refinements into portable std.typed properties"',
)
insert = '''
^{:refer 'tool.metaspec.schema/compile-schema :id 'no-generic-metaspec-schema-heads}
(fact "does not emit generic :tool.metaspec schema extensions"
  (let [compiled (schema/compile-schema sample (:meta/document-schema sample))
        rendered (str compiled)]
    [(first compiled)
     (str/includes? rendered ":tool.metaspec/map")
     (str/includes? rendered ":tool.metaspec/set")
     (str/includes? rendered ":tool.metaspec/items")
     (str/includes? rendered ":tool.metaspec/refine")])
  => [:map false false false false])

'''
content = replace_once(
    content,
    "^{:refer 'std.typed.schema/valid?\n  :id 'compiled-schema-is-executable}",
    insert + "^{:refer 'std.typed.schema/valid?\n  :id 'compiled-schema-is-executable}",
    "no metaspec generic heads fact",
)
write(path, content)

path = ".github/workflows/std-typed-schema.yml"
content = read(path)
content = replace_once(
    content,
    '''      - name: Run portable explanation facts on Rust
        run: scripts/runtime/run-lib-tests core/lib/test/std/typed/explain_test.hal''',
    '''      - name: Run portable schema and explanation facts on Rust
        run: |
          scripts/runtime/run-lib-tests \\
            core/lib/test/std/typed/schema_test.hal \\
            core/lib/test/std/typed/properties_test.hal \\
            core/lib/test/std/typed/explain_test.hal''',
    "portable property parity tests",
)
write(path, content)

print("applied #832 canonical std.typed property promotion")
