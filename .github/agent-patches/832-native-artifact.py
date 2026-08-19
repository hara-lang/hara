from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PATH = ROOT / "core/rust/src/vm/artifact.rs"
text = PATH.read_text()


def replace_once(old, new):
    global text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"artifact patch marker expected once, found {count}: {old[:80]!r}")
    text = text.replace(old, new, 1)


replace_once(
    """        SchemaType::Vector(item) => {\n            out.byte(3);\n            write_schema_type(out, item)?;\n        }\n        SchemaType::Tuple(items) => {""",
    """        SchemaType::Vector(item) => {\n            out.byte(3);\n            write_schema_type(out, item)?;\n        }\n        SchemaType::Set(item) => {\n            out.byte(10);\n            write_schema_type(out, item)?;\n        }\n        SchemaType::Tuple(items) => {""",
)

replace_once(
    """        SchemaType::Map(fields) => {\n            out.byte(5);\n            out.len(fields.len())?;\n            for field in fields {\n                write_schema_form(out, &field.name)?;\n                write_schema_type(out, &field.value_type)?;\n            }\n        }\n        SchemaType::Function(arities) => {""",
    """        SchemaType::Map(fields) => {\n            // Keep tag 5 byte-for-byte compatible with existing artifacts.\n            // Property-aware fields use tag 12 so older schema maps remain readable.\n            let property_aware = fields.iter().any(|field| field.properties.is_some());\n            out.byte(if property_aware { 12 } else { 5 });\n            out.len(fields.len())?;\n            for field in fields {\n                write_schema_form(out, &field.name)?;\n                if property_aware {\n                    match &field.properties {\n                        Some(properties) => {\n                            out.byte(1);\n                            write_schema_form(out, properties)?;\n                        }\n                        None => out.byte(0),\n                    }\n                }\n                write_schema_type(out, &field.value_type)?;\n            }\n        }\n        SchemaType::WithProperties { schema, properties } => {\n            out.byte(11);\n            write_schema_type(out, schema)?;\n            write_schema_form(out, properties)?;\n        }\n        SchemaType::Function(arities) => {""",
)

replace_once(
    """        3 => SchemaType::Vector(Box::new(read_schema_type(reader)?)),\n        4 => SchemaType::Tuple(reader.many(read_schema_type)?),\n        5 => SchemaType::Map(reader.many(|reader| {\n            Ok(SchemaField {\n                name: read_schema_form(reader)?,\n                value_type: read_schema_type(reader)?,\n            })\n        })?),\n        6 => SchemaType::Function(reader.many(|reader| {""",
    """        3 => SchemaType::Vector(Box::new(read_schema_type(reader)?)),\n        4 => SchemaType::Tuple(reader.many(read_schema_type)?),\n        5 => SchemaType::Map(reader.many(|reader| {\n            Ok(SchemaField {\n                name: read_schema_form(reader)?,\n                properties: None,\n                value_type: read_schema_type(reader)?,\n            })\n        })?),\n        6 => SchemaType::Function(reader.many(|reader| {""",
)

replace_once(
    """        9 => SchemaType::Unknown(read_schema_form(reader)?),\n        _ => return Err(\"bytecode artifact contains unknown schema type\".into()),""",
    """        9 => SchemaType::Unknown(read_schema_form(reader)?),\n        10 => SchemaType::Set(Box::new(read_schema_type(reader)?)),\n        11 => SchemaType::WithProperties {\n            schema: Box::new(read_schema_type(reader)?),\n            properties: read_schema_form(reader)?,\n        },\n        12 => SchemaType::Map(reader.many(|reader| {\n            let name = read_schema_form(reader)?;\n            let properties = if reader.boolean()? {\n                Some(read_schema_form(reader)?)\n            } else {\n                None\n            };\n            Ok(SchemaField {\n                name,\n                properties,\n                value_type: read_schema_type(reader)?,\n            })\n        })?),\n        _ => return Err(\"bytecode artifact contains unknown schema type\".into()),""",
)

PATH.write_text(text)

TEST_PATH = ROOT / "core/rust/src/vm/artifact/tests.rs"
test = TEST_PATH.read_text()
old = """        SchemaType::Map(vec![SchemaField {\n            name: crate::kernel::parse(\":id\").unwrap(),\n            value_type: SchemaType::Primitive(\"int\".into()),\n        }]),\n    );\n    program.function_types.insert("""
new = """        SchemaType::Map(vec![SchemaField {\n            name: crate::kernel::parse(\":id\").unwrap(),\n            properties: None,\n            value_type: SchemaType::Primitive(\"int\".into()),\n        }]),\n    );\n    program.schema_types.insert(\n        \"demo/Labels\".into(),\n        SchemaType::Set(Box::new(SchemaType::Primitive(\"keyword\".into()))),\n    );\n    program.schema_types.insert(\n        \"demo/Handle\".into(),\n        SchemaType::WithProperties {\n            schema: Box::new(SchemaType::Primitive(\"str\".into())),\n            properties: crate::kernel::parse(\"{:min-count 1 :max-count 32}\").unwrap(),\n        },\n    );\n    program.schema_types.insert(\n        \"demo/Profile\".into(),\n        SchemaType::WithProperties {\n            schema: Box::new(SchemaType::Map(vec![SchemaField {\n                name: crate::kernel::parse(\":nickname\").unwrap(),\n                properties: Some(crate::kernel::parse(\"{:optional true}\").unwrap()),\n                value_type: SchemaType::Primitive(\"str\".into()),\n            }])),\n            properties: crate::kernel::parse(\"{:closed true}\").unwrap(),\n        },\n    );\n    program.function_types.insert("""
count = test.count(old)
if count != 1:
    raise SystemExit(f"artifact test patch marker expected once, found {count}")
TEST_PATH.write_text(test.replace(old, new, 1))

RUST_PROBE_PATH = ROOT / "core/rust/tests/std_typed_schema.rs"
probe = RUST_PROBE_PATH.read_text()
old_marker = ':pattern "^a"'
new_marker = r':pattern \"^a\"'
count = probe.count(old_marker)
if count != 1:
    raise SystemExit(f"Rust property probe quoting marker expected once, found {count}")
RUST_PROBE_PATH.write_text(probe.replace(old_marker, new_marker, 1))

print("applied #832 bytecode schema artifact parity")
