package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import com.oracle.truffle.api.interop.InteropLibrary;
import com.oracle.truffle.api.interop.UnsupportedMessageException;
import java.math.BigInteger;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.PolyglotException;
import org.graalvm.polyglot.Value;
import org.junit.Test;

public class HaraNumericInteropTest {
  private static final BigInteger LARGE_INTEGER = new BigInteger("123456789012345678901234567890");

  @Test
  public void exportsBigIntegersAsExactPolyglotNumbers() throws Exception {
    InteropLibrary interop = InteropLibrary.getUncached();
    Object exported = HaraBox.export(LARGE_INTEGER);

    assertTrue(interop.isNumber(exported));
    assertTrue(interop.fitsInBigInteger(exported));
    assertFalse(interop.fitsInLong(exported));
    assertFalse(interop.fitsInDouble(exported));
    assertEquals(LARGE_INTEGER, interop.asBigInteger(exported));
    assertThrows(UnsupportedMessageException.class, () -> interop.asLong(exported));
  }

  @Test
  public void exportsArbitraryBigDecimalsAsBoxedHostObjects() throws Exception {
    InteropLibrary interop = InteropLibrary.getUncached();
    Object exported = HaraBox.export(new java.math.BigDecimal("1.2300"));

    assertFalse(interop.isNumber(exported));
    assertFalse(interop.hasMembers(exported));
  }

  @Test
  public void exposesArbitraryIntegersAndExactDecimalsAtTheLanguageBoundary() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      Value integer = context.eval(HaraLanguage.ID, Long.toString(Long.MAX_VALUE));
      assertTrue(integer.isNumber());
      assertEquals(Long.MAX_VALUE, integer.asLong());

      PolyglotException large =
          assertThrows(
              PolyglotException.class,
              () -> context.eval(HaraLanguage.ID, LARGE_INTEGER.toString()));
      assertTrue(large.getMessage().contains("Invalid number"));

      Value floating = context.eval(HaraLanguage.ID, "1.2300");
      assertTrue(floating.isNumber());
      assertTrue(floating.fitsInDouble());
      assertEquals(1.23, floating.asDouble(), 0.0);

      PolyglotException suffix =
          assertThrows(PolyglotException.class, () -> context.eval(HaraLanguage.ID, "1.2300M"));
      assertTrue(suffix.getMessage().contains("legacy numeric suffixes N and M"));
    }
  }
}
