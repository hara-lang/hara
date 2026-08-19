#!/usr/bin/env python3
"""Finalize the generated #832 native candidate before validation.

HBC schema property surfaces are portable forms, but their Java containers do not all
participate in HbcProgram record equality. Verify the portable contract directly:
decoding must preserve the promoted node kinds and annotations, and re-encoding must be
byte-for-byte canonical.
"""

from pathlib import Path


path = Path("core/java/src/test/java/hara/truffle/bytecode/HbcCodecTest.java")
text = path.read_text()
before = """    assertEquals(program, decoded);
    assertArrayEquals(first, HbcCodec.encode(decoded));
"""
after = """    assertArrayEquals(first, HbcCodec.encode(decoded));

    assertTrue(decoded.schemaTypes().get("demo/Labels") instanceof HalcSchema.SetType);
    assertTrue(decoded.schemaTypes().get("demo/Profile") instanceof HalcSchema.Properties);
    HalcSchema.Properties profile =
        (HalcSchema.Properties) decoded.schemaTypes().get("demo/Profile");
    assertTrue(profile.properties() != null);
    assertTrue(profile.schema() instanceof HalcSchema.MapType);
    HalcSchema.MapType profileMap = (HalcSchema.MapType) profile.schema();
    assertEquals(1, profileMap.fields().size());
    assertTrue(profileMap.fields().get(0).properties() != null);
"""
if after not in text:
    if before not in text:
        raise SystemExit(f"expected HBC round-trip marker not found in {path}")
    path.write_text(text.replace(before, after, 1))

print("finalized #832 native schema candidate")
