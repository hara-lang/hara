from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CODEC = ROOT / "core/java/src/main/java/hara/truffle/bytecode/HbcCodec.java"
TEST = ROOT / "core/java/src/test/java/hara/truffle/bytecode/HbcCodecTest.java"


def replace_once(text, old, new, label):
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one marker, found {count}")
    return text.replace(old, new, 1)


text = CODEC.read_text()
text = replace_once(
    text,
    """      case HalcSchema.VectorType vector -> {\n        out.u8(3);\n        writeSchemaType(out, vector.item());\n      }\n      case HalcSchema.Tuple tuple -> {""",
    """      case HalcSchema.VectorType vector -> {\n        out.u8(3);\n        writeSchemaType(out, vector.item());\n      }\n      case HalcSchema.SetType set -> {\n        out.u8(10);\n        writeSchemaType(out, set.item());\n      }\n      case HalcSchema.Tuple tuple -> {""",
    "HbcCodec set writer",
)
text = replace_once(
    text,
    """      case HalcSchema.MapType map -> {\n        out.u8(5);\n        out.many(\n            map.fields(),\n            field -> {\n              writeSchemaSurface(out, field.name());\n              writeSchemaType(out, field.type());\n            });\n      }\n      case HalcSchema.FunctionType function -> {""",
    """      case HalcSchema.MapType map -> {\n        // Preserve tag 5 for property-free maps so existing HBC artifacts remain stable.\n        boolean propertyAware = map.fields().stream().anyMatch(field -> field.properties() != null);\n        out.u8(propertyAware ? 12 : 5);\n        out.many(\n            map.fields(),\n            field -> {\n              writeSchemaSurface(out, field.name());\n              if (propertyAware) {\n                out.bool(field.properties() != null);\n                if (field.properties() != null) writeSchemaSurface(out, field.properties());\n              }\n              writeSchemaType(out, field.type());\n            });\n      }\n      case HalcSchema.Properties properties -> {\n        out.u8(11);\n        writeSchemaType(out, properties.schema());\n        writeSchemaSurface(out, properties.properties());\n      }\n      case HalcSchema.FunctionType function -> {""",
    "HbcCodec property writer",
)
text = replace_once(
    text,
    """      case 5 ->\n          new HalcSchema.MapType(\n              in.many(\n                  reader ->\n                      new HalcSchema.Field(\n                          readSchemaSurface(reader), readSchemaType(reader))));\n      case 6 ->""",
    """      case 5 ->\n          new HalcSchema.MapType(\n              in.many(\n                  reader ->\n                      new HalcSchema.Field(\n                          readSchemaSurface(reader), null, readSchemaType(reader))));\n      case 6 ->""",
    "HbcCodec legacy map reader",
)
text = replace_once(
    text,
    """      case 9 -> new HalcSchema.Unknown(readSchemaSurface(in));\n      default -> throw malformed(\"bytecode artifact contains unknown schema type\");""",
    """      case 9 -> new HalcSchema.Unknown(readSchemaSurface(in));\n      case 10 -> new HalcSchema.SetType(readSchemaType(in));\n      case 11 -> new HalcSchema.Properties(readSchemaType(in), readSchemaSurface(in));\n      case 12 ->\n          new HalcSchema.MapType(\n              in.many(\n                  reader -> {\n                    Object name = readSchemaSurface(reader);\n                    Object properties = reader.bool() ? readSchemaSurface(reader) : null;\n                    return new HalcSchema.Field(name, properties, readSchemaType(reader));\n                  }));\n      default -> throw malformed(\"bytecode artifact contains unknown schema type\");""",
    "HbcCodec new schema readers",
)
CODEC.write_text(text)

test = TEST.read_text()
test = replace_once(
    test,
    """                        new HalcSchema.Field(\n                            hara.lang.data.Keyword.create(\"id\"),\n                            new HalcSchema.Primitive(\"int\"))))),\n            Map.of(""",
    """                        new HalcSchema.Field(\n                            hara.lang.data.Keyword.create(\"id\"),\n                            null,\n                            new HalcSchema.Primitive(\"int\")))),\n                \"demo/Labels\",\n                new HalcSchema.SetType(new HalcSchema.Primitive(\"keyword\")),\n                \"demo/Handle\",\n                new HalcSchema.Properties(\n                    new HalcSchema.Primitive(\"str\"),\n                    HalcSchema.readSurface(\"{:min-count 1 :max-count 32}\")),\n                \"demo/Profile\",\n                new HalcSchema.Properties(\n                    new HalcSchema.MapType(\n                        List.of(\n                            new HalcSchema.Field(\n                                hara.lang.data.Keyword.create(\"nickname\"),\n                                HalcSchema.readSurface(\"{:optional true}\"),\n                                new HalcSchema.Primitive(\"str\")))),\n                    HalcSchema.readSurface(\"{:closed true}\"))),\n            Map.of(""",
    "HbcCodecTest property schemas",
)
TEST.write_text(test)

print("applied #832 Truffle HBC schema artifact parity")
