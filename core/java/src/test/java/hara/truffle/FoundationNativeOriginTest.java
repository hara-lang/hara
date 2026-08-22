package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;

import hara.lang.data.Symbol;
import org.graalvm.polyglot.Context;
import org.junit.Test;

public class FoundationNativeOriginTest {
  @Test
  public void freshContextsKeepStringOwnershipAndOriginsSeparate() {
    assertOrigins("std.foundation.string", "length", "std.native.String", "length");
  }

  @Test
  public void freshContextsKeepBytesOwnershipAndOriginsSeparate() {
    assertOrigins("std.foundation.bytes", "count", "std.native.Bytes", "count");
  }

  @Test
  public void freshContextsKeepPromiseOwnershipAndOriginsSeparate() {
    assertOrigins("std.foundation.promise", "run", "std.native.Promise", "run");
  }

  @Test
  public void freshContextsKeepCoroutineOwnershipAndOriginsSeparate() {
    assertOrigins("std.foundation.coroutine", "create", "std.native.Coroutine", "create");
  }

  private static void assertOrigins(
      String foundationNamespace,
      String foundationSymbol,
      String nativeNamespace,
      String nativeSymbol) {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      context.eval(HaraLanguage.ID, "nil");
      context.enter();
      try {
        HaraContext hara = HaraLanguage.currentContext();
        HaraVar foundation = hara.resolve(Symbol.create(foundationNamespace, foundationSymbol));
        HaraVar nativeVar = hara.resolve(Symbol.create(nativeNamespace, nativeSymbol));
        assertNotNull(foundation);
        assertNotNull(nativeVar);
        assertEquals(HaraVar.Origin.HAL_FALLBACK, foundation.origin());
        assertEquals(HaraVar.Origin.RUNTIME_PRIMITIVE, nativeVar.origin());
      } finally {
        context.leave();
      }
    }
  }

}
