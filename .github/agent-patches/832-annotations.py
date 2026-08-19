#!/usr/bin/env python3
"""Promote schema annotations to portable, lossless property data for #832."""

from pathlib import Path


def replace_once(path: Path, before: str, after: str) -> None:
    text = path.read_text()
    if after in text:
        return
    if before not in text:
        raise SystemExit(f"expected marker not found in {path}")
    path.write_text(text.replace(before, after, 1))


schema = Path("core/lib/src/std/typed/schema.hal")
replace_once(
    schema,
    """(defn- normalize-properties
  [kind properties allowed]
  (let [unknown
        (vec
         (filter (fn [key] (not (has? allowed key)))
                 (keys properties)))]
    (if (empty? unknown)
      properties
      (throw
       (ex-info
        (str \"unsupported \" kind \" schema property\")
        {:kind kind :properties unknown})))))
""",
    """(defn- normalize-properties
  [kind properties _allowed]
  (if (map? properties)
    properties
    (throw
     (ex-info
      (str kind \" schema properties must be a map\")
      {:kind kind :properties properties}))))
""",
)

properties_test = Path("core/lib/test/std/typed/properties_test.hal")
annotation_fact = """^{:refer 'std.typed.schema/normalize :id 'portable-schema-annotations}
(fact \"preserves schema and map-entry annotations without treating them as constraints\"
  (let [contract
        [:map {:title \"User record\"
               :version 2
               :owner :accounts
               :closed true}
         [:name {:required true
                 :description \"Display name\"
                 :default \"Anonymous\"}
          [:str {:title \"Display name\" :min-count 1}]]]
        normalized (schema/normalize contract)
        field (first (:fields normalized))]
    [(:properties normalized)
     (:properties field)
     (:properties (:type field))
     (typed/valid? contract {:name \"Ada\"})])
  => [{:title \"User record\"
       :version 2
       :owner :accounts
       :closed true}
      {:required true
       :description \"Display name\"
       :default \"Anonymous\"}
      {:title \"Display name\" :min-count 1}
      true])

"""
marker = "^{:refer 'std.typed.schema/valid? :id 'primitive-properties}\n"
text = properties_test.read_text()
if annotation_fact not in text:
    if marker not in text:
        raise SystemExit(f"expected annotation test marker not found in {properties_test}")
    properties_test.write_text(text.replace(marker, annotation_fact + marker, 1))

rust_schema = Path("core/rust/src/kernel/schema.rs")
replace_once(
    rust_schema,
    """fn supports_properties(head: &str) -> bool {
    matches!(head, \"str\" | \"keyword\" | \"vector\" | \"set\" | \"map\")
}
""",
    """fn supports_properties(head: &str) -> bool {
    matches!(
        head,
        \"str\"
            | \"string\"
            | \"keyword\"
            | \"symbol\"
            | \"list\"
            | \"bytes\"
            | \"int\"
            | \"integer\"
            | \"num\"
            | \"number\"
            | \"any\"
            | \"vector\"
            | \"set\"
            | \"map\"
    )
}
""",
)

java_schema = Path("core/java/src/main/java/hara/truffle/HalcSchema.java")
replace_once(
    java_schema,
    """  private static boolean supportsProperties(String head) {
    return List.of(\"str\", \"keyword\", \"vector\", \"set\", \"map\").contains(head);
  }
""",
    """  private static boolean supportsProperties(String head) {
    return List.of(
            \"str\",
            \"string\",
            \"keyword\",
            \"symbol\",
            \"list\",
            \"bytes\",
            \"int\",
            \"integer\",
            \"num\",
            \"number\",
            \"any\",
            \"vector\",
            \"set\",
            \"map\")
        .contains(head);
  }
""",
)


def insert_native_surfaces(path: Path, java: bool = False) -> None:
    text = path.read_text()
    if ':title \\"Age\\" :owner :accounts' in text:
        return
    lines = text.splitlines(keepends=True)
    matches = [i for i, line in enumerate(lines) if ":vendor/type" in line]
    if len(matches) != 1:
        raise SystemExit(f"expected one :vendor/type surface marker in {path}, found {len(matches)}")
    index = matches[0]
    if java:
        additions = [
            r'                      + "       (quote [:int {:title \"Age\" :owner :accounts}]) "' + "\n",
            r'                      + "       (quote [:map {:title \"User record\" :version 2 :owner :accounts} [:name {:required true :description \"Display name\" :default \"Anonymous\"} :str]]) "' + "\n",
        ]
    else:
        line = lines[index]
        prefix, suffix = line.split(":vendor/type", 1)
        additions = [
            prefix + r'(quote [:int {:title \"Age\" :owner :accounts}])' + suffix,
            prefix
            + r'(quote [:map {:title \"User record\" :version 2 :owner :accounts} [:name {:required true :description \"Display name\" :default \"Anonymous\"} :str]])'
            + suffix,
        ]
    lines[index + 1 : index + 1] = additions
    path.write_text("".join(lines))


insert_native_surfaces(Path("core/rust/tests/std_typed_schema.rs"))
insert_native_surfaces(Path("core/java/src/test/java/hara/truffle/StdTypedSchemaTest.java"), java=True)

rust_hbc = Path("core/rust/src/vm/artifact/tests.rs")
replace_once(
    rust_hbc,
    r'            properties: crate::kernel::parse("{:min-count 1 :max-count 32}").unwrap(),',
    r'            properties: crate::kernel::parse("{:title \"Display handle\" :version 2 :owner :accounts :min-count 1 :max-count 32}").unwrap(),',
)
replace_once(
    rust_hbc,
    r'                properties: Some(crate::kernel::parse("{:optional true}").unwrap()),',
    r'                properties: Some(crate::kernel::parse("{:required true :description \"Display nickname\" :default \"Anonymous\"}").unwrap()),',
)
replace_once(
    rust_hbc,
    r'            properties: crate::kernel::parse("{:closed true}").unwrap(),',
    r'            properties: crate::kernel::parse("{:title \"User profile\" :version 2 :owner :accounts :closed true}").unwrap(),',
)

java_hbc = Path("core/java/src/test/java/hara/truffle/bytecode/HbcCodecTest.java")
replace_once(
    java_hbc,
    r'                    HalcSchema.readSurface("{:min-count 1 :max-count 32}")),',
    r'                    HalcSchema.readSurface("{:title \"Display handle\" :version 2 :owner :accounts :min-count 1 :max-count 32}")),',
)
replace_once(
    java_hbc,
    r'                                HalcSchema.readSurface("{:optional true}"),',
    r'                                HalcSchema.readSurface("{:required true :description \"Display nickname\" :default \"Anonymous\"}"),',
)
replace_once(
    java_hbc,
    r'                    HalcSchema.readSurface("{:closed true}"))),',
    r'                    HalcSchema.readSurface("{:title \"User profile\" :version 2 :owner :accounts :closed true}"))),',
)

print("promoted #832 schema annotations as lossless portable data")
