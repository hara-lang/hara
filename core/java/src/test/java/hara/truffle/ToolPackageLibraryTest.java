package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertThrows;

import hara.truffle.bytecode.HbcFormatException;
import java.io.InputStream;
import org.junit.Test;

public final class ToolPackageLibraryTest {
  @Test
  public void validatesAndInspectsTheRustProducedHbxFixture() throws Exception {
    byte[] bundle;
    try (InputStream input = getClass().getResourceAsStream("/std.foundation.hbx")) {
      if (input == null) throw new AssertionError("missing Rust HBX fixture");
      bundle = input.readAllBytes();
    }
    assertEquals(Boolean.TRUE, ToolPackageLibrary.validate(null, new Object[] {bundle}));
    ToolPackageLibrary.inspect(null, new Object[] {bundle});
    ToolPackageLibrary.unpack(null, new Object[] {bundle});

    byte[] malformed = bundle.clone();
    malformed[malformed.length - 1] ^= 1;
    assertThrows(
        HbcFormatException.class,
        () -> ToolPackageLibrary.validate(null, new Object[] {malformed}));
  }
}
