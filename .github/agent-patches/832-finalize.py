#!/usr/bin/env python3
"""Finalize the generated #832 native candidate before validation.

The promotion scripts intentionally derive their edits from the current source tree. This
small finalizer corrects two representation details that must hold at the portable
boundary: map-entry annotations are stored as the value of :properties (not as a nested
{:properties ...} map), and JVM HBC verification checks the preserved schema surfaces
rather than relying on Java object identity for decoded Forms.
"""

from pathlib import Path


def replace_once(path: Path, before: str, after: str) -> None:
    text = path.read_text()
    if after in text:
        return
    if before not in text:
        raise SystemExit(f"expected marker not found in {path}")
    path.write_text(text.replace(before, after, 1))


rust_schema = Path("core/rust/src/halc/std_typed_schema.rs")
replace_once(
    rust_schema,
    """                    let schema = parse_surface(&schema_form)?;
                    let properties = (!options.is_empty()).then(|| Form::Map(options));
                    entries.insert(name.clone(), MapEntry { schema, properties });
""",
    """                    let schema = parse_surface(&schema_form)?;
                    let explicit_properties = options.remove(&Form::keyword("properties"));
                    let properties = match explicit_properties {
                        Some(properties) if options.is_empty() => Some(properties),
                        Some(_) => {
                            return Err(
                                "map entry cannot mix :properties with direct options"
                                    .to_string(),
                            )
                        }
                        None => (!options.is_empty()).then(|| Form::Map(options)),
                    };
                    entries.insert(name.clone(), MapEntry { schema, properties });
""",
)

java_hbc_test = Path("core/java/src/test/java/hara/truffle/bytecode/HbcCodecTest.java")
replace_once(
    java_hbc_test,
    """        assertEquals(program, decoded);
        assertArrayEquals(first, HbcCodec.encode(decoded));
""",
    """        assertArrayEquals(first, HbcCodec.encode(decoded));

        assertInstanceOf(HalcSchema.SetType.class, decoded.schemaTypes().get("demo/Labels"));
        HalcSchema.WithProperties profile = assertInstanceOf(
                HalcSchema.WithProperties.class,
                decoded.schemaTypes().get("demo/Profile"));
        assertNotNull(profile.properties());
        HalcSchema.MapType profileMap = assertInstanceOf(
                HalcSchema.MapType.class,
                profile.schema());
        assertNotNull(profileMap.entries().get(Form.keyword("name")).properties());
""",
)

print("finalized #832 native schema candidate")
