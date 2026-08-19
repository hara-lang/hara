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


# Public facade loads the property grammar so callers of std.typed get the
# complete portable schema surface.
path = "core/lib/src/std/typed.hal"
content = read(path)
content = replace_once(
    content,
    "            [std.typed.infer :as inference]\n            [std.typed.registry :as reg]",
    "            [std.typed.infer :as inference]\n            [std.typed.properties]\n            [std.typed.registry :as reg]",
    "std.typed properties require",
)
write(path, content)


# Make the new namespace part of the standard library inventory.
path = "core/rust/standard-library.namespaces"
content = read(path)
content = replace_once(
    content,
    "std.typed.infer\nstd.typed.registry",
    "std.typed.infer\nstd.typed.properties\nstd.typed.registry",
    "standard library inventory",
)
write(path, content)


# Metaspec now consumes the portable property grammar instead of owning it.
path = "core/lib/src/tool/metaspec/schema.hal"
content = read(path)
content = replace_once(
    content,
    "  (:require [std.typed.registry :as registry]\n            [std.typed.schema :as typed]))",
    "  (:require [std.typed.properties]\n            [std.typed.registry :as registry]\n            [std.typed.schema :as typed]))",
    "metaspec property require",
)

# Delete generic map/set/items/refinement implementations. The unsupported-type
# adapter remains metaspec-specific and intentionally stays here.
content = regex_once(
    content,
    r"\(defn- validate-item-values.*?(?=\(defmethod std\.typed\.schema/normalize :tool\.metaspec/unsupported-type)",
    "",
    "remove generic metaspec schema extensions",
)

# Replace the old refinement-options/custom-wrapper compiler with direct
# std.typed property data.
content = regex_once(
    content,
    r"\(defn- refinement-options.*?(?=\(defn- compile-declaration)",
    r'''(defn- assoc-present
  [output key value]
  (if (nil? value)
    output
    (assoc output key value)))

(defn- schema-properties
  [declaration schema-type]
  (let [qualified
        (or (= schema-type :qualified-keyword)
            (= :qualified (:schema/constraint declaration)))
        min-count
        (or (:schema/min-count declaration)
            (:schema/min-length declaration))
        max-count
        (or (:schema/max-count declaration)
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
  (if (empty? properties)
    base
    [base properties]))

(defn- collection-form
  [head properties item]
  (if (empty? properties)
    [head item]
    [head properties item]))

''',
    "replace refinement compiler helpers",
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
            (= schema-type :vector)
            (collection-form :vector properties item-schema)

            (= schema-type :set)
            (collection-form :set properties item-schema)

            :else
            [:or
             (collection-form :vector properties item-schema)
             (collection-form :set properties item-schema)]))

        (= schema-type :vector)
        (collection-form :vector properties :any)

        (= schema-type :set)
        (collection-form :set properties :any)

        (= schema-type :qualified-keyword)
        (with-properties :keyword (assoc properties :qualified true))

        (= schema-type :version)
        (with-properties :str properties)

        (nil? schema-type)
        (with-properties :any properties)

        (primitive-type? schema-type)
        (with-properties (primitive-schema schema-type) properties)

        :else
        [:tool.metaspec/unsupported-type schema-type]))))

''',
    "compile metaspec declarations to std.typed properties",
)

# The normalized forms now come from std.typed.properties.
content = content.replace("(= kind :tool.metaspec/map) :map", "(= kind :std.typed/map) :map")
content = content.replace("(= kind :tool.metaspec/set) :set", "(= kind :std.typed/set) :set")
content = content.replace("(= kind :tool.metaspec/items) :collection", "(= kind :std.typed/vector) :vector")
content = replace_once(
    content,
    "      (= kind :tool.metaspec/unsupported-type) (:type expected)\n      :else kind)))",
    "      (= kind :std.typed/refine) (expected-type (:base expected))\n      (= kind :tool.metaspec/unsupported-type) (:type expected)\n      :else kind)))",
    "expected typed property kinds",
)

# Translate typed property findings back into the existing metaspec repair
# protocol. This preserves the public metaspec envelope while std.typed owns
# value-shape execution.
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
         {:action/type :add-key
          :action/path parent-path
          :action/key key}))

      (= finding-type :std.typed.schema/closed-map-key)
      (let [context (:finding/context finding)
            key (:key context)]
        (schema-finding
         :fail path
         (str "Key is not allowed by closed schema: " key)
         {:action/type :remove-key
          :action/path path}))

      (= finding-type :std.typed.schema/property-violation)
      (let [context (:finding/context finding)
            constraint (:constraint context)
            expected (:expected context)
            value (:finding/value finding)]
        (cond
          (= constraint :qualified)
          (schema-finding
           :fail path
           (str "Expected qualified keyword, got " value)
           {:action/type :qualify-value
            :action/path path})

          (= constraint :min-count)
          (schema-finding
           :fail path
           (str "Value has fewer than " expected " members")
           (if (string? value)
             {:action/type :replace-value
              :action/path path
              :action/min-length expected}
             {:action/type :add-members
              :action/path path
              :action/min-count expected}))

          (= constraint :max-count)
          (schema-finding
           :fail path
           (str "Value has more than " expected " members")
           (if (string? value)
             {:action/type :replace-value
              :action/path path
              :action/max-length expected}
             {:action/type :remove-members
              :action/path path
              :action/max-count expected}))

          (= constraint :distinct)
          (schema-finding
           :fail path
           "Collection members must be unique"
           {:action/type :remove-duplicates
            :action/path path})

          (= constraint :pattern)
          (schema-finding
           :fail path
           (str "Value does not match pattern: " value)
           {:action/type :replace-value
            :action/path path
            :action/expected expected})

          :else
          (schema-finding
           :fail path
           "Value violates schema property"
           {:action/type :replace-value
            :action/path path
            :action/expected expected})))

      (= finding-type :std.typed.schema/invalid-value)''',
    "translate property findings",
)
write(path, content)


# Update the metaspec-focused contract for the new typed-owned normalized form.
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
  (any? (fn [field]
          (= key (:name field)))
        (:fields normalized)))

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
  => [:std.typed/map
      [:demo/name :demo/status]
      3
      true
      true
      true
      '[demo/document demo/name demo/status]])''',
    "metaspec normalized map expectation",
)
content = content.replace(
    'fact "keeps metaspec refinements as namespaced std.typed extensions"',
    'fact "lowers metaspec refinements into portable std.typed properties"',
)
insert = '''
^{:refer 'tool.metaspec.schema/compile-schema
  :id 'no-generic-metaspec-schema-heads}
(fact "does not emit generic :tool.metaspec schema extensions"
  (let [compiled
        (schema/compile-schema
         sample
         (:meta/document-schema sample))
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
    "metaspec no-extension fact",
)
write(path, content)


# Add focused portable property facts.
properties_test = r'''(ns std.typed.properties-test
  (:use code.test)
  (:require [std.typed :as typed]
            [std.typed.registry :as registry]
            [std.typed.schema :as schema]))

^{:refer 'std.typed.schema/valid?
  :id 'primitive-properties}
(fact "validates portable string and qualified-keyword properties"
  [(typed/valid? [:str {:min-count 2 :max-count 4 :pattern "^a.*$"}] "ab")
   (typed/valid? [:str {:min-count 2 :max-count 4 :pattern "^a.*$"}] "b")
   (typed/valid? [:keyword {:qualified true}] :demo/value)
   (typed/valid? [:keyword {:qualified true}] :value)]
  => [true false true false])

^{:refer 'std.typed.schema/valid?
  :id 'collection-properties}
(fact "validates vector bounds, distinctness, and typed sets"
  [(typed/valid? [:vector {:min-count 1 :max-count 3 :distinct true} :int]
                 [1 2 3])
   (typed/valid? [:vector {:min-count 1 :max-count 3 :distinct true} :int]
                 [1 1])
   (typed/valid? [:set :keyword] #{:a :b})
   (typed/valid? [:set :keyword] #{:a 2})]
  => [true false true false])

^{:refer 'std.typed.schema/validate
  :id 'map-properties}
(fact "validates optional entries and closed maps with exact typed findings"
  (let [contract
        [:map {:closed true}
         [:id :int]
         [:nickname {:optional true} :str]]
        missing (first (typed/validate contract {}))
        extra (first (typed/validate contract {:id 1 :extra true}))]
    [[(:finding/type missing)
      (:finding/path missing)
      (:present? (:finding/context missing))]
     [(:finding/type extra)
      (:finding/path extra)]
     (typed/valid? contract {:id 1})
     (typed/valid? contract {:id 1 :nickname "Ada"})])
  => [[:std.typed.schema/missing-key [:id] false]
      [:std.typed.schema/closed-map-key [:extra]]
      true
      true])

^{:refer 'std.typed.schema/valid?
  :id 'registry-property-traversal}
(fact "keeps registry references executable inside property-aware maps"
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
'''
write("core/lib/test/std/typed/properties_test.hal", properties_test)


# Keep the focused parity workflow authoritative for the new grammar.
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
    "rust portable property tests",
)
content = replace_once(
    content,
    '''            core/lib/src/std/typed/registry.hal \\
            core/lib/src/std/typed/schema.hal \\
            core/lib/src/std/typed/infer.hal \\''',
    '''            core/lib/src/std/typed/registry.hal \\
            core/lib/src/std/typed/schema.hal \\
            core/lib/src/std/typed/properties.hal \\
            core/lib/src/std/typed/infer.hal \\''',
    "HAL property probe source",
)
write(path, content)

print("applied #832 product patch")
