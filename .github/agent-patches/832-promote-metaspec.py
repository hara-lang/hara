from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]


def read(path):
    return (ROOT / path).read_text()


def write(path, text):
    (ROOT / path).write_text(text)


def replace_once(path, old, new):
    text = read(path)
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one replacement marker, found {count}")
    write(path, text.replace(old, new, 1))


def sub_once(path, pattern, replacement):
    text = read(path)
    output, count = re.subn(pattern, replacement, text, count=1, flags=re.S)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one regex replacement, found {count}: {pattern}")
    write(path, output)


# ---------------------------------------------------------------------------
# Rust canonical schema model
# ---------------------------------------------------------------------------
RUST_SCHEMA = "core/rust/src/kernel/schema.rs"
replace_once(
    RUST_SCHEMA,
    """pub struct SchemaField {\n    pub name: Form,\n    pub value_type: SchemaType,\n}\n""",
    """pub struct SchemaField {\n    pub name: Form,\n    pub properties: Option<Form>,\n    pub value_type: SchemaType,\n}\n""",
)
replace_once(
    RUST_SCHEMA,
    """    Vector(Box<SchemaType>),\n    Tuple(Vec<SchemaType>),\n    Map(Vec<SchemaField>),\n""",
    """    Vector(Box<SchemaType>),\n    Set(Box<SchemaType>),\n    Tuple(Vec<SchemaType>),\n    Map(Vec<SchemaField>),\n    WithProperties {\n        schema: Box<SchemaType>,\n        properties: Form,\n    },\n""",
)
replace_once(
    RUST_SCHEMA,
    """        SchemaType::Vector(item) => {\n            Form::Vector(vec![Form::Keyword(\"vector\".into()), nested(item)])\n        }\n        SchemaType::Tuple(items) =>""",
    """        SchemaType::Vector(item) => {\n            Form::Vector(vec![Form::Keyword(\"vector\".into()), nested(item)])\n        }\n        SchemaType::Set(item) => {\n            Form::Vector(vec![Form::Keyword(\"set\".into()), nested(item)])\n        }\n        SchemaType::Tuple(items) =>""",
)
replace_once(
    RUST_SCHEMA,
    """                    .chain(fields.iter().map(|field| {\n                        Form::Vector(vec![field.name.clone(), nested(&field.value_type)])\n                    }))\n""",
    """                    .chain(fields.iter().map(|field| {\n                        let mut pair = vec![field.name.clone()];\n                        if let Some(properties) = &field.properties {\n                            pair.push(properties.clone());\n                        }\n                        pair.push(nested(&field.value_type));\n                        Form::Vector(pair)\n                    }))\n""",
)
replace_once(
    RUST_SCHEMA,
    """        SchemaType::Enum(values) => Form::Vector(\n            std::iter::once(Form::Keyword(\"enum\".into()))\n                .chain(values.iter().cloned())\n                .collect(),\n        ),\n        SchemaType::Extension { head, arguments } =>""",
    """        SchemaType::Enum(values) => Form::Vector(\n            std::iter::once(Form::Keyword(\"enum\".into()))\n                .chain(values.iter().cloned())\n                .collect(),\n        ),\n        SchemaType::WithProperties { schema, properties } => {\n            let Form::Vector(mut values) = nested(schema) else {\n                return nested(schema);\n            };\n            values.insert(1, properties.clone());\n            Form::Vector(values)\n        }\n        SchemaType::Extension { head, arguments } =>""",
)

sub_once(
    RUST_SCHEMA,
    r"fn normalize_longhand_field\(field: &Form\) -> Result<SchemaField, String> \{.*?\n\}\n\nfn normalize_function_inputs",
    """fn normalize_longhand_field(field: &Form) -> Result<SchemaField, String> {\n    let Form::Map(entries) = field else {\n        return Err(\"map schema fields must be {:name name :type schema} maps\".into());\n    };\n    let name = longhand_value(entries, \"name\")\n        .ok_or_else(|| \"map schema field requires :name\".to_string())?;\n    let value_type = longhand_value(entries, \"type\")\n        .ok_or_else(|| \"map schema field requires :type\".to_string())?;\n    let properties = match longhand_value(entries, \"properties\") {\n        None => None,\n        Some(Form::Map(values)) => Some(Form::Map(values.clone())),\n        Some(_) => return Err(\"map schema field :properties must be a map\".into()),\n    };\n    Ok(SchemaField {\n        name: name.clone(),\n        properties,\n        value_type: normalize_schema(value_type)?,\n    })\n}\n\nfn normalize_function_inputs""",
)

sub_once(
    RUST_SCHEMA,
    r"fn normalize_longhand\(entries: &\[\(Form, Form\)\]\) -> Result<SchemaType, String> \{.*?\n\}\n\n/// Infers conservative",
    """fn normalize_longhand(entries: &[(Form, Form)]) -> Result<SchemaType, String> {\n    let Some(Form::Keyword(kind)) = longhand_value(entries, \"kind\") else {\n        return Ok(SchemaType::Unknown(Form::Map(entries.to_vec())));\n    };\n    let children = longhand_children(entries)?;\n    let normalized = match kind.as_str() {\n        \"primitive\" => {\n            let value = longhand_value(entries, \"name\").or_else(|| children.first());\n            match value {\n                Some(Form::Keyword(name)) => Ok(SchemaType::Primitive(name.clone())),\n                _ => Err(\"primitive schema requires one keyword name\".into()),\n            }\n        }\n        \"reference\" => {\n            let value = longhand_value(entries, \"name\").or_else(|| children.first());\n            value\n                .ok_or_else(|| \"reference schema requires :name\".to_string())\n                .and_then(normalize_reference_name)\n        }\n        \"union\" | \"or\" => normalize_union_forms(longhand_sequence(entries, \"types\", children)?),\n        \"vector\" => {\n            let value = longhand_value(entries, \"item\").or_else(|| children.first());\n            value\n                .ok_or_else(|| \"vector schema requires :item\".to_string())\n                .and_then(normalize_schema)\n                .map(|value| SchemaType::Vector(Box::new(value)))\n        }\n        \"set\" => {\n            let value = longhand_value(entries, \"item\").or_else(|| children.first());\n            value\n                .ok_or_else(|| \"set schema requires :item\".to_string())\n                .and_then(normalize_schema)\n                .map(|value| SchemaType::Set(Box::new(value)))\n        }\n        \"tuple\" => longhand_sequence(entries, \"items\", children)?\n            .iter()\n            .map(normalize_schema)\n            .collect::<Result<Vec<_>, _>>()\n            .map(SchemaType::Tuple),\n        \"map\" => {\n            if longhand_value(entries, \"fields\").is_some() {\n                longhand_sequence(entries, \"fields\", &[])?\n                    .iter()\n                    .map(normalize_longhand_field)\n                    .collect::<Result<Vec<_>, _>>()\n                    .map(SchemaType::Map)\n            } else {\n                children\n                    .iter()\n                    .map(normalize_map_field)\n                    .collect::<Result<Vec<_>, _>>()\n                    .map(SchemaType::Map)\n            }\n        }\n        \"fn\" => normalize_longhand_function(entries).map(|arity| SchemaType::Function(vec![arity])),\n        \"function\" => {\n            normalize_longhand_functions(longhand_sequence(entries, \"arities\", children)?)\n        }\n        \"enum\" => Ok(SchemaType::Enum(\n            longhand_sequence(entries, \"values\", children)?.to_vec(),\n        )),\n        \"extension\" => {\n            let head = longhand_value(entries, \"head\")\n                .or_else(|| longhand_value(entries, \"name\"))\n                .ok_or_else(|| \"extension schema requires :head\".to_string())?;\n            let Form::Keyword(head) = head else {\n                return Err(\"extension schema :head must be a keyword\".into());\n            };\n            Ok(SchemaType::Extension {\n                head: head.clone(),\n                arguments: longhand_sequence(entries, \"arguments\", children)?.to_vec(),\n            })\n        }\n        \"unknown\" => Ok(SchemaType::Unknown(\n            longhand_value(entries, \"surface\")\n                .or_else(|| children.first())\n                .cloned()\n                .unwrap_or_else(|| Form::Map(entries.to_vec())),\n        )),\n        _ => Err(format!(\"unsupported longhand schema kind: {kind}\")),\n    }?;\n    match longhand_value(entries, \"properties\") {\n        None => Ok(normalized),\n        Some(Form::Map(values)) => Ok(SchemaType::WithProperties {\n            schema: Box::new(normalized),\n            properties: Form::Map(values.clone()),\n        }),\n        Some(_) => Err(\"schema :properties must be a map\".into()),\n    }\n}\n\n/// Infers conservative""",
)

replace_once(
    RUST_SCHEMA,
    """fn resolve_type<'a>(\n    schema: &'a SchemaType,\n    definitions: &'a HashMap<String, SchemaType>,\n) -> Option<&'a SchemaType> {\n    let mut current = schema;\n    let mut visited = std::collections::HashSet::new();\n    while let SchemaType::Reference(name) = current {\n        if !visited.insert(name) {\n            return Some(current);\n        }\n        current = definitions.get(name)?;\n    }\n    Some(current)\n}\n""",
    """fn resolve_type<'a>(\n    schema: &'a SchemaType,\n    definitions: &'a HashMap<String, SchemaType>,\n) -> Option<&'a SchemaType> {\n    let mut current = schema;\n    let mut visited = std::collections::HashSet::new();\n    loop {\n        match current {\n            SchemaType::WithProperties { schema, .. } => current = schema,\n            SchemaType::Reference(name) => {\n                if !visited.insert(name) {\n                    return Some(current);\n                }\n                current = definitions.get(name)?;\n            }\n            _ => return Some(current),\n        }\n    }\n}\n""",
)
replace_once(
    RUST_SCHEMA,
    """                .map(|(name, value)| SchemaField {\n                    name: name.clone(),\n                    value_type: infer_expression(value, environment),\n                })\n                .collect(),\n        ),\n        Form::Set(_) => SchemaType::Extension {\n            head: \"set\".into(),\n            arguments: Vec::new(),\n        },\n""",
    """                .map(|(name, value)| SchemaField {\n                    name: name.clone(),\n                    properties: None,\n                    value_type: infer_expression(value, environment),\n                })\n                .collect(),\n        ),\n        Form::Set(values) => SchemaType::Set(Box::new(join_types(\n            values\n                .iter()\n                .map(|value| infer_expression(value, environment)),\n        ))),\n""",
)

sub_once(
    RUST_SCHEMA,
    r"fn normalize_composite\(items: &\[Form\]\) -> Result<SchemaType, String> \{.*?\n\}\n\nfn normalize_function",
    """fn normalize_map_field(argument: &Form) -> Result<SchemaField, String> {\n    let Form::Vector(pair) = argument else {\n        return Err(\":map schema fields must be [name type] or [name properties type]\".into());\n    };\n    match pair.as_slice() {\n        [name, value_type] => Ok(SchemaField {\n            name: name.clone(),\n            properties: None,\n            value_type: normalize_schema(value_type)?,\n        }),\n        [name, Form::Map(properties), value_type] => Ok(SchemaField {\n            name: name.clone(),\n            properties: Some(Form::Map(properties.clone())),\n            value_type: normalize_schema(value_type)?,\n        }),\n        _ => Err(\":map schema fields must be [name type] or [name properties type]\".into()),\n    }\n}\n\nfn supports_properties(head: &str) -> bool {\n    matches!(head, \"str\" | \"keyword\" | \"vector\" | \"set\" | \"map\")\n}\n\nfn normalize_composite(items: &[Form]) -> Result<SchemaType, String> {\n    let Form::Keyword(head) = &items[0] else {\n        return Ok(SchemaType::Unknown(Form::Vector(items.to_vec())));\n    };\n    let raw_arguments = &items[1..];\n    let (properties, arguments) = if supports_properties(head) {\n        match raw_arguments.first() {\n            Some(Form::Map(values)) => (Some(Form::Map(values.clone())), &raw_arguments[1..]),\n            _ => (None, raw_arguments),\n        }\n    } else {\n        (None, raw_arguments)\n    };\n    let normalized = match head.as_str() {\n        \"or\" => normalize_union_forms(arguments),\n        \"maybe\" => {\n            require_count(head, arguments, 1)?;\n            let mut members = Vec::new();\n            push_unique(&mut members, normalize_schema(&arguments[0])?);\n            push_unique(&mut members, SchemaType::Primitive(\"nil\".into()));\n            Ok(SchemaType::Union(members))\n        }\n        \"vector\" => {\n            require_count(head, arguments, 1)?;\n            Ok(SchemaType::Vector(Box::new(normalize_schema(&arguments[0])?)))\n        }\n        \"set\" => {\n            require_count(head, arguments, 1)?;\n            Ok(SchemaType::Set(Box::new(normalize_schema(&arguments[0])?)))\n        }\n        \"tuple\" => arguments\n            .iter()\n            .map(normalize_schema)\n            .collect::<Result<Vec<_>, _>>()\n            .map(SchemaType::Tuple),\n        \"map\" => arguments\n            .iter()\n            .map(normalize_map_field)\n            .collect::<Result<Vec<_>, _>>()\n            .map(SchemaType::Map),\n        \"fn\" => normalize_function(items).map(|arity| SchemaType::Function(vec![arity])),\n        \"function\" => {\n            if arguments.is_empty() {\n                return Err(\":function schema requires at least one :fn schema\".into());\n            }\n            arguments\n                .iter()\n                .map(|argument| {\n                    let Form::Vector(function) = argument else {\n                        return Err(\":function members must be :fn schemas\".into());\n                    };\n                    normalize_function(function)\n                })\n                .collect::<Result<Vec<_>, _>>()\n                .map(SchemaType::Function)\n        }\n        \"enum\" => Ok(SchemaType::Enum(arguments.to_vec())),\n        _ if arguments.is_empty() => Ok(SchemaType::Primitive(head.clone())),\n        _ => Ok(SchemaType::Extension {\n            head: head.clone(),\n            arguments: arguments.to_vec(),\n        }),\n    }?;\n    Ok(match properties {\n        Some(properties) => SchemaType::WithProperties {\n            schema: Box::new(normalized),\n            properties,\n        },\n        None => normalized,\n    })\n}\n\nfn normalize_function""",
)

# ---------------------------------------------------------------------------
# Rust native Schema ABI flattens internal properties into the canonical AST.
# ---------------------------------------------------------------------------
RUST_PROTOCOL = "core/rust/src/core/protocol.rs"
replace_once(
    RUST_PROTOCOL,
    """        Union(_) => \"union\",\n        Vector(_) => \"vector\",\n        Tuple(_) => \"tuple\",\n        Map(_) => \"map\",\n""",
    """        Union(_) => \"union\",\n        Vector(_) => \"vector\",\n        Set(_) => \"set\",\n        Tuple(_) => \"tuple\",\n        Map(_) => \"map\",\n        WithProperties { schema, .. } => schema_kind(schema),\n""",
)
replace_once(
    RUST_PROTOCOL,
    """        Vector(value) => schema_ast_map(vec![\n            (\"kind\", Form::Keyword(\"vector\".into())),\n            (\"item\", schema_ast_form(value)),\n        ]),\n        Tuple(values) =>""",
    """        Vector(value) => schema_ast_map(vec![\n            (\"kind\", Form::Keyword(\"vector\".into())),\n            (\"item\", schema_ast_form(value)),\n        ]),\n        Set(value) => schema_ast_map(vec![\n            (\"kind\", Form::Keyword(\"set\".into())),\n            (\"item\", schema_ast_form(value)),\n        ]),\n        Tuple(values) =>""",
)
replace_once(
    RUST_PROTOCOL,
    """                        .map(|field| {\n                            schema_ast_map(vec![\n                                (\"name\", field.name.clone()),\n                                (\"type\", schema_ast_form(&field.value_type)),\n                            ])\n                        })\n""",
    """                        .map(|field| {\n                            let mut entries = vec![(\"name\", field.name.clone())];\n                            if let Some(properties) = &field.properties {\n                                entries.push((\"properties\", properties.clone()));\n                            }\n                            entries.push((\"type\", schema_ast_form(&field.value_type)));\n                            schema_ast_map(entries)\n                        })\n""",
)
replace_once(
    RUST_PROTOCOL,
    """        Enum(values) => schema_ast_map(vec![\n            (\"kind\", Form::Keyword(\"enum\".into())),\n            (\"values\", Form::Vector(values.clone())),\n        ]),\n        Extension { head, arguments } =>""",
    """        Enum(values) => schema_ast_map(vec![\n            (\"kind\", Form::Keyword(\"enum\".into())),\n            (\"values\", Form::Vector(values.clone())),\n        ]),\n        WithProperties { schema, properties } => {\n            let Form::Map(mut entries) = schema_ast_form(schema) else {\n                unreachable!(\"canonical schema AST must be a map\");\n            };\n            entries.push((Form::Keyword(\"properties\".into()), properties.clone()));\n            Form::Map(entries)\n        }\n        Extension { head, arguments } =>""",
)

# Extend the Rust parity probe with every promoted generic property shape.
RUST_TEST = "core/rust/tests/std_typed_schema.rs"
replace_once(
    RUST_TEST,
    """                        (quote [:vector [:maybe :int]]) \\\n                        (quote [:tuple :keyword :int :str]) \\\n                        (quote [:map [:name :str] [:tags [:vector :keyword]]]) \\\n""",
    """                        (quote [:vector [:maybe :int]]) \\\n                        (quote [:str {:min-count 1 :max-count 8 :pattern \"^a\"}]) \\\n                        (quote [:keyword {:qualified true}]) \\\n                        (quote [:vector {:min-count 1 :max-count 3 :distinct true} :int]) \\\n                        (quote [:set {:min-count 1 :max-count 3} :keyword]) \\\n                        (quote [:tuple :keyword :int :str]) \\\n                        (quote [:map [:name :str] [:tags [:vector :keyword]]]) \\\n                        (quote [:map {:closed true} [:id :int] [:nickname {:optional true} :str]]) \\\n""",
)
replace_once(
    RUST_TEST,
    """                    [(Schema/kind (schema (quote [:or :int :str]))) \\\n                     (Schema/kind (schema (quote [:fn [:int] :int]))) \\\n                     (Schema/kind \\\n                      (schema \\\n                       (quote [:function [:fn [:int] :int] \\\n                                         [:fn [:str] :str]])))]])\"\n""",
    """                    (= (typed/normalize \\\n                        (quote [:map {:closed true} \\\n                                     [:id :int] \\\n                                     [:nickname {:optional true} :str]])) \\\n                       {:kind :map \\\n                        :properties {:closed true} \\\n                        :fields \\\n                        [{:name :id \\\n                          :type {:kind :primitive :name :int}} \\\n                         {:name :nickname \\\n                          :properties {:optional true} \\\n                          :type {:kind :primitive :name :str}}]}) \\\n                    [(Schema/kind (schema (quote [:or :int :str]))) \\\n                     (Schema/kind (schema (quote [:fn [:int] :int]))) \\\n                     (Schema/kind (schema (quote [:set :int]))) \\\n                     (Schema/kind (schema (quote [:str {:min-count 1}]))) \\\n                     (Schema/kind \\\n                      (schema \\\n                       (quote [:function [:fn [:int] :int] \\\n                                         [:fn [:str] :str]])))]])\"\n""",
)
replace_once(RUST_TEST, '"[true true [:union :fn :function]]"', '"[true true true [:union :fn :set :primitive :function]]"')

# ---------------------------------------------------------------------------
# Truffle canonical schema model
# ---------------------------------------------------------------------------
JAVA_SCHEMA = "core/java/src/main/java/hara/truffle/HalcSchema.java"
replace_once(
    JAVA_SCHEMA,
    """          Union,\n          VectorType,\n          Tuple,\n          MapType,\n          FunctionType,\n""",
    """          Union,\n          VectorType,\n          SetType,\n          Tuple,\n          MapType,\n          Properties,\n          FunctionType,\n""",
)
replace_once(
    JAVA_SCHEMA,
    """  public record VectorType(Type item) implements Type {}\n\n  public record Tuple""",
    """  public record VectorType(Type item) implements Type {}\n\n  public record SetType(Type item) implements Type {}\n\n  public record Tuple""",
)
replace_once(
    JAVA_SCHEMA,
    """  public record Field(Object name, Type type) {}\n\n  public record MapType""",
    """  public record Field(Object name, Object properties, Type type) {}\n\n  public record MapType""",
)
replace_once(
    JAVA_SCHEMA,
    """  public record MapType(List<Field> fields) implements Type {\n    public MapType {\n      fields = List.copyOf(fields);\n    }\n  }\n\n  public record Function""",
    """  public record MapType(List<Field> fields) implements Type {\n    public MapType {\n      fields = List.copyOf(fields);\n    }\n  }\n\n  public record Properties(Type schema, Object properties) implements Type {}\n\n  public record Function""",
)
replace_once(
    JAVA_SCHEMA,
    """  public static Object shorthand(Type schema) {\n    if (schema instanceof Primitive primitive) {""",
    """  public static Object shorthand(Type schema) {\n    if (schema instanceof Properties decorated) {\n      ILinearType<?> surface = vector(shorthand(decorated.schema()));\n      if (surface == null || surface.count() == 0) return shorthand(decorated.schema());\n      ArrayList<Object> values = new ArrayList<>();\n      values.add(surface.nth(0));\n      values.add(decorated.properties());\n      for (int index = 1; index < surface.count(); index++) values.add(surface.nth(index));\n      return vectorOf(values.toArray());\n    }\n    if (schema instanceof Primitive primitive) {""",
)
replace_once(
    JAVA_SCHEMA,
    """    if (schema instanceof VectorType vector) {\n      return vectorOf(Keyword.create(\"vector\"), shorthand(vector.item()));\n    }\n    if (schema instanceof Tuple tuple) {""",
    """    if (schema instanceof VectorType vector) {\n      return vectorOf(Keyword.create(\"vector\"), shorthand(vector.item()));\n    }\n    if (schema instanceof SetType set) {\n      return vectorOf(Keyword.create(\"set\"), shorthand(set.item()));\n    }\n    if (schema instanceof Tuple tuple) {""",
)
replace_once(
    JAVA_SCHEMA,
    """      map.fields().forEach(\n          field -> values.add(vectorOf(field.name(), shorthand(field.type()))));\n""",
    """      map.fields().forEach(\n          field -> {\n            if (field.properties() == null)\n              values.add(vectorOf(field.name(), shorthand(field.type())));\n            else\n              values.add(vectorOf(field.name(), field.properties(), shorthand(field.type())));\n          });\n""",
)

sub_once(
    JAVA_SCHEMA,
    r"    List<Object> arguments = values\(vector, 1\);\n    String headName = keywordName\(head\);\n    return switch \(headName\) \{.*?\n    \};\n  \}\n\n  private static Entry<Object, Object> longhandEntry",
    """    List<Object> arguments = values(vector, 1);\n    String headName = keywordName(head);\n    Object properties = null;\n    if (supportsProperties(headName) && !arguments.isEmpty() && schemaMap(arguments.get(0)) != null) {\n      properties = arguments.remove(0);\n    }\n    Type normalized = switch (headName) {\n      case \"or\" -> normalizeUnion(arguments);\n      case \"maybe\" -> {\n        requireCount(headName, arguments, 1);\n        yield normalizeUnion(List.of(arguments.get(0), Keyword.create(\"nil\")));\n      }\n      case \"vector\" -> {\n        requireCount(headName, arguments, 1);\n        yield new VectorType(normalize(arguments.get(0)));\n      }\n      case \"set\" -> {\n        requireCount(headName, arguments, 1);\n        yield new SetType(normalize(arguments.get(0)));\n      }\n      case \"tuple\" -> new Tuple(normalizeAll(arguments));\n      case \"map\" -> normalizeMap(arguments);\n      case \"fn\" -> new FunctionType(List.of(normalizeFunction(vector)));\n      case \"function\" -> {\n        if (arguments.isEmpty()) {\n          throw invalid(\":function schema requires at least one :fn schema\");\n        }\n        List<Function> arities = new ArrayList<>();\n        for (Object argument : arguments) {\n          ILinearType<?> function = vector(argument);\n          if (function == null) {\n            throw invalid(\":function members must be :fn schemas\");\n          }\n          arities.add(normalizeFunction(function));\n        }\n        yield new FunctionType(arities);\n      }\n      case \"enum\" -> new EnumType(arguments);\n      default -> arguments.isEmpty()\n          ? new Primitive(headName)\n          : new Extension(headName, arguments);\n    };\n    return properties == null ? normalized : new Properties(normalized, properties);\n  }\n\n  private static boolean supportsProperties(String head) {\n    return List.of(\"str\", \"keyword\", \"vector\", \"set\", \"map\").contains(head);\n  }\n\n  private static Entry<Object, Object> longhandEntry""",
)

replace_once(
    JAVA_SCHEMA,
    """      Entry<Object, Object> name = longhandEntry(field, \"name\");\n      Entry<Object, Object> type = longhandEntry(field, \"type\");\n      if (name == null) throw invalid(\"map schema field requires :name\");\n      if (type == null) throw invalid(\"map schema field requires :type\");\n      fields.add(new Field(name.getValue(), normalize(type.getValue())));\n""",
    """      Entry<Object, Object> name = longhandEntry(field, \"name\");\n      Entry<Object, Object> type = longhandEntry(field, \"type\");\n      Object properties = longhandValue(field, \"properties\");\n      if (name == null) throw invalid(\"map schema field requires :name\");\n      if (type == null) throw invalid(\"map schema field requires :type\");\n      if (properties != null && schemaMap(properties) == null)\n        throw invalid(\"map schema field :properties must be a map\");\n      fields.add(new Field(name.getValue(), properties, normalize(type.getValue())));\n""",
)

sub_once(
    JAVA_SCHEMA,
    r"  private static Type normalizeLonghand\(IMapType<Object, Object> schema, String kind\) \{\n    List<Object> children = longhandValues\(schema, \"children\", List\.of\(\)\);\n    return switch \(kind\) \{.*?\n    \};\n  \}",
    """  private static Type normalizeLonghand(IMapType<Object, Object> schema, String kind) {\n    List<Object> children = longhandValues(schema, \"children\", List.of());\n    Type normalized = switch (kind) {\n      case \"primitive\" -> {\n        Object name = longhandValue(schema, \"name\");\n        if (name == null && !children.isEmpty()) name = children.get(0);\n        if (!(name instanceof Keyword keyword)) {\n          throw invalid(\"primitive schema requires one keyword name\");\n        }\n        yield new Primitive(keywordName(keyword));\n      }\n      case \"reference\" -> {\n        Object name = longhandValue(schema, \"name\");\n        if (name == null && !children.isEmpty()) name = children.get(0);\n        yield normalizeReference(name);\n      }\n      case \"union\", \"or\" -> normalizeUnion(longhandValues(schema, \"types\", children));\n      case \"vector\" -> {\n        Object item = longhandValue(schema, \"item\");\n        if (item == null && !children.isEmpty()) item = children.get(0);\n        if (item == null) throw invalid(\"vector schema requires :item\");\n        yield new VectorType(normalize(item));\n      }\n      case \"set\" -> {\n        Object item = longhandValue(schema, \"item\");\n        if (item == null && !children.isEmpty()) item = children.get(0);\n        if (item == null) throw invalid(\"set schema requires :item\");\n        yield new SetType(normalize(item));\n      }\n      case \"tuple\" -> new Tuple(normalizeAll(longhandValues(schema, \"items\", children)));\n      case \"map\" -> normalizeLonghandMap(schema, children);\n      case \"fn\" -> new FunctionType(List.of(normalizeLonghandFunction(schema)));\n      case \"function\" -> normalizeLonghandFunctions(longhandValues(schema, \"arities\", children));\n      case \"enum\" -> new EnumType(longhandValues(schema, \"values\", children));\n      case \"extension\" -> {\n        Object headValue = longhandValue(schema, \"head\");\n        if (headValue == null) headValue = longhandValue(schema, \"name\");\n        if (!(headValue instanceof Keyword head)) {\n          throw invalid(\"extension schema :head must be a keyword\");\n        }\n        yield new Extension(keywordName(head), longhandValues(schema, \"arguments\", children));\n      }\n      case \"unknown\" -> {\n        Object surface = longhandValue(schema, \"surface\");\n        if (surface == null && !children.isEmpty()) surface = children.get(0);\n        yield new Unknown(surface == null ? schema : surface);\n      }\n      default -> throw invalid(\"unsupported longhand schema kind: \" + kind);\n    };\n    Object properties = longhandValue(schema, \"properties\");\n    if (properties == null) return normalized;\n    if (schemaMap(properties) == null) throw invalid(\"schema :properties must be a map\");\n    return new Properties(normalized, properties);\n  }""",
)

replace_once(
    JAVA_SCHEMA,
    """  private static Type resolve(Type type, Map<String, Type> definitions) {\n    HashSet<String> visited = new HashSet<>();\n    while (type instanceof Reference reference && visited.add(reference.name())) {\n      Type next = definitions.get(reference.name());\n      if (next == null) break;\n      type = next;\n    }\n    return type;\n  }\n""",
    """  private static Type resolve(Type type, Map<String, Type> definitions) {\n    HashSet<String> visited = new HashSet<>();\n    while (true) {\n      if (type instanceof Properties decorated) {\n        type = decorated.schema();\n        continue;\n      }\n      if (!(type instanceof Reference reference) || !visited.add(reference.name())) return type;\n      Type next = definitions.get(reference.name());\n      if (next == null) return type;\n      type = next;\n    }\n  }\n""",
)
replace_once(
    JAVA_SCHEMA,
    """        fields.add(new Field(entry.getKey(), inferExpression(entry.getValue(), environment)));\n      }\n      return new MapType(fields);\n    }\n    if (form instanceof hara.lang.data.types.ISetType<?>)\n      return new Extension(\"set\", List.of());\n""",
    """        fields.add(new Field(entry.getKey(), null, inferExpression(entry.getValue(), environment)));\n      }\n      return new MapType(fields);\n    }\n    if (form instanceof hara.lang.data.types.ISetType<?> set) {\n      List<Type> members = new ArrayList<>();\n      for (Object value : set) pushJoined(members, inferExpression(value, environment));\n      return new SetType(join(members));\n    }\n""",
)
sub_once(
    JAVA_SCHEMA,
    r"  private static Type normalizeMap\(List<Object> arguments\) \{.*?\n  \}\n\n  private static Function normalizeFunction",
    """  private static Type normalizeMap(List<Object> arguments) {\n    List<Field> fields = new ArrayList<>();\n    for (Object argument : arguments) {\n      ILinearType<?> pair = vector(argument);\n      if (pair == null || (pair.count() != 2 && pair.count() != 3)) {\n        throw invalid(\":map schema fields must be [name type] or [name properties type]\");\n      }\n      if (pair.count() == 2) {\n        fields.add(new Field(pair.nth(0), null, normalize(pair.nth(1))));\n      } else {\n        Object properties = pair.nth(1);\n        if (schemaMap(properties) == null)\n          throw invalid(\":map schema field properties must be a map\");\n        fields.add(new Field(pair.nth(0), properties, normalize(pair.nth(2))));\n      }\n    }\n    return new MapType(fields);\n  }\n\n  private static Function normalizeFunction""",
)

# ---------------------------------------------------------------------------
# Truffle Schema native ABI: same canonical AST, no Properties wrapper leakage.
# ---------------------------------------------------------------------------
JAVA_CONTEXT = "core/java/src/main/java/hara/truffle/HaraContext.java"
replace_once(
    JAVA_CONTEXT,
    """  private static String schemaKind(HalcSchema.Type ast) {\n    if (ast instanceof HalcSchema.Primitive) return \"primitive\";\n    if (ast instanceof HalcSchema.Reference) return \"reference\";\n    if (ast instanceof HalcSchema.Union) return \"union\";\n    if (ast instanceof HalcSchema.VectorType) return \"vector\";\n    if (ast instanceof HalcSchema.Tuple) return \"tuple\";\n    if (ast instanceof HalcSchema.MapType) return \"map\";\n""",
    """  private static String schemaKind(HalcSchema.Type ast) {\n    if (ast instanceof HalcSchema.Properties decorated) return schemaKind(decorated.schema());\n    if (ast instanceof HalcSchema.Primitive) return \"primitive\";\n    if (ast instanceof HalcSchema.Reference) return \"reference\";\n    if (ast instanceof HalcSchema.Union) return \"union\";\n    if (ast instanceof HalcSchema.VectorType) return \"vector\";\n    if (ast instanceof HalcSchema.SetType) return \"set\";\n    if (ast instanceof HalcSchema.Tuple) return \"tuple\";\n    if (ast instanceof HalcSchema.MapType) return \"map\";\n""",
)
replace_once(
    JAVA_CONTEXT,
    """  private static Object schemaAst(HalcSchema.Type ast) {\n    if (ast instanceof HalcSchema.Primitive primitive) {""",
    """  private static Object schemaAst(HalcSchema.Type ast) {\n    if (ast instanceof HalcSchema.Properties decorated) {\n      Object base = schemaAst(decorated.schema());\n      if (!(base instanceof hara.lang.data.types.IMapType<?, ?> map))\n        throw new HaraException(\"canonical schema AST must be a map\");\n      ArrayList<Object> entries = new ArrayList<>();\n      for (Object item : map) {\n        Map.Entry<?, ?> entry = (Map.Entry<?, ?>) item;\n        entries.add(entry.getKey());\n        entries.add(entry.getValue());\n      }\n      entries.add(Keyword.create(\"properties\"));\n      entries.add(decorated.properties());\n      return schemaAstMap(entries.toArray());\n    }\n    if (ast instanceof HalcSchema.Primitive primitive) {""",
)
replace_once(
    JAVA_CONTEXT,
    """    if (ast instanceof HalcSchema.VectorType vector) {\n      return schemaAstMap(\n          Keyword.create(\"kind\"), Keyword.create(\"vector\"),\n          Keyword.create(\"item\"), schemaAst(vector.item()));\n    }\n    if (ast instanceof HalcSchema.Tuple tuple) {""",
    """    if (ast instanceof HalcSchema.VectorType vector) {\n      return schemaAstMap(\n          Keyword.create(\"kind\"), Keyword.create(\"vector\"),\n          Keyword.create(\"item\"), schemaAst(vector.item()));\n    }\n    if (ast instanceof HalcSchema.SetType set) {\n      return schemaAstMap(\n          Keyword.create(\"kind\"), Keyword.create(\"set\"),\n          Keyword.create(\"item\"), schemaAst(set.item()));\n    }\n    if (ast instanceof HalcSchema.Tuple tuple) {""",
)
replace_once(
    JAVA_CONTEXT,
    """      map.fields().forEach(\n          field ->\n              fields.add(\n                  schemaAstMap(\n                      Keyword.create(\"name\"), field.name(),\n                      Keyword.create(\"type\"), schemaAst(field.type()))));\n""",
    """      map.fields().forEach(\n          field -> {\n            if (field.properties() == null) {\n              fields.add(\n                  schemaAstMap(\n                      Keyword.create(\"name\"), field.name(),\n                      Keyword.create(\"type\"), schemaAst(field.type())));\n            } else {\n              fields.add(\n                  schemaAstMap(\n                      Keyword.create(\"name\"), field.name(),\n                      Keyword.create(\"properties\"), field.properties(),\n                      Keyword.create(\"type\"), schemaAst(field.type())));\n            }\n          });\n""",
)

# Java parity probe gets the same promoted surfaces and canonical map assertion.
JAVA_TEST = "core/java/src/test/java/hara/truffle/StdTypedSchemaTest.java"
replace_once(
    JAVA_TEST,
    '                      + "       (quote [:vector [:maybe :int]]) "\n                      + "       (quote [:tuple :keyword :int :str]) "',
    '                      + "       (quote [:vector [:maybe :int]]) "\n'
    '                      + "       (quote [:str {:min-count 1 :max-count 8 :pattern \\\"^a\\\"}]) "\n'
    '                      + "       (quote [:keyword {:qualified true}]) "\n'
    '                      + "       (quote [:vector {:min-count 1 :max-count 3 :distinct true} :int]) "\n'
    '                      + "       (quote [:set {:min-count 1 :max-count 3} :keyword]) "\n'
    '                      + "       (quote [:tuple :keyword :int :str]) "',
)
replace_once(
    JAVA_TEST,
    '                      + "       (quote [:map [:name :str] [:tags [:vector :keyword]]]) "\n                      + "       (quote [:fn [:str & :any] :str]) "',
    '                      + "       (quote [:map [:name :str] [:tags [:vector :keyword]]]) "\n'
    '                      + "       (quote [:map {:closed true} [:id :int] [:nickname {:optional true} :str]]) "\n'
    '                      + "       (quote [:fn [:str & :any] :str]) "',
)
replace_once(
    JAVA_TEST,
    '          "[[] true [:union :fn :function]]",',
    '          "[[] true true [:union :fn :set :primitive :function]]",',
)
replace_once(
    JAVA_TEST,
    '                      + "    [(Schema/kind "\n                      + "      (std.foundation/schema (quote [:or :int :str]))) "',
    '                      + "    (= (std.typed.schema/normalize "\n'
    '                      + "        (quote [:map {:closed true} "\n'
    '                      + "                     [:id :int] "\n'
    '                      + "                     [:nickname {:optional true} :str]])) "\n'
    '                      + "       {:kind :map "\n'
    '                      + "        :properties {:closed true} "\n'
    '                      + "        :fields "\n'
    '                      + "        [{:name :id :type {:kind :primitive :name :int}} "\n'
    '                      + "         {:name :nickname :properties {:optional true} "\n'
    '                      + "          :type {:kind :primitive :name :str}}]}) "\n'
    '                      + "    [(Schema/kind "\n'
    '                      + "      (std.foundation/schema (quote [:or :int :str]))) "',
)
replace_once(
    JAVA_TEST,
    '                      + "     (Schema/kind "\n                      + "      (std.foundation/schema (quote [:fn [:int] :int]))) "\n                      + "     (Schema/kind "',
    '                      + "     (Schema/kind "\n'
    '                      + "      (std.foundation/schema (quote [:fn [:int] :int]))) "\n'
    '                      + "     (Schema/kind "\n'
    '                      + "      (std.foundation/schema (quote [:set :int]))) "\n'
    '                      + "     (Schema/kind "\n'
    '                      + "      (std.foundation/schema (quote [:str {:min-count 1}]))) "\n'
    '                      + "     (Schema/kind "',
)

print("applied #832 native schema parity candidate")
