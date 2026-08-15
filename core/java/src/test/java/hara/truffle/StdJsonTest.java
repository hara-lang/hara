package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import org.graalvm.polyglot.Context;
import org.junit.Test;

public class StdJsonTest {
  @Test
  public void strictJsonReadsWritesAndPrettyPrints() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertEquals(
          "[nil true -2 \"x\" [3] {\"a\" 4}]",
          context
              .eval(HaraLanguage.ID, "(std.native.Json/read \"[null,true,-2,\\\"x\\\",[3],{\\\"a\\\":4}]\")")
              .toString());
      assertEquals(
          "{\"a\":1,\"b\":[true,null]}",
          context.eval(HaraLanguage.ID, "(std.native.Json/write {\"a\" 1 \"b\" [true nil]})").asString());
      assertEquals(
          "{\"a\":1}",
          context.eval(HaraLanguage.ID, "(Json/write {\"a\" 1})").asString());
      assertEquals(
          "{\n  \"a\": 1\n}",
          context.eval(HaraLanguage.ID, "(std.native.Json/pretty {\"a\" 1} {})").asString());
      assertThrows(
          RuntimeException.class,
          () -> context.eval(HaraLanguage.ID, "(std.native.Json/pretty {\"a\" 1} nil)"));
    }
  }

  @Test
  public void strictJsonRejectsUnsupportedFormsAndPrettyUsesReadablePrinter() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertThrows(
          RuntimeException.class,
          () -> context.eval(HaraLanguage.ID, "(std.native.Json/read \"1.5\")"));
      assertThrows(
          RuntimeException.class,
          () -> context.eval(HaraLanguage.ID, "(std.native.Json/write {:a 1})"));
      assertEquals(
          "{:a [1 2]}",
          context.eval(HaraLanguage.ID, "(pretty/pprint-str {:a [1 2]})").asString());
    }
  }
  @Test
  public void nativeResultsAndErrorsRoundTripThroughJsonEnvelopes() {
    Object data =
        hara.lang.data.Map.Standard.from(null, "value", 42L);
    Object context =
        hara.lang.data.Map.Standard.from(null, "source", "rpc");
    HaraResult success = HaraResult.success(data, context);
    String encoded = StdJson.write(success);
    assertTrue(encoded.startsWith("{\"$hara\":\"result\",\"status\":\"success\""));
    Object value = StdJson.read(encoded);
    assertTrue(value instanceof HaraResult);
    HaraResult decoded = (HaraResult) value;
    assertTrue(decoded.isSuccess());
    assertEquals("rpc", decoded.context().lookup("source"));

    hara.lang.base.Ex.Info error =
        new hara.lang.base.Ex.Info(
            "boom",
            hara.lang.data.Map.Standard.from(null, "code", "demo/boom"));
    decoded =
        (HaraResult) StdJson.read(StdJson.write(HaraResult.error(error, context)));
    assertTrue(decoded.isError());
    assertEquals("boom", decoded.errorValue().getMessage());
  }

  @Test
  public void resultJsonStripsDisplayRejectsNativeContextAndValidatesExactEnvelopes() {
    Object context =
        hara.lang.data.Map.Standard.from(
            null,
            hara.lang.data.Keyword.create("display"),
            new Object(),
            "source",
            "json");
    String encoded = StdJson.write(HaraResult.success(1L, context));
    assertFalse(encoded.contains("display"));
    assertTrue(encoded.contains("\"source\":\"json\""));

    HaraResult nonportable =
        HaraResult.success(
            1L,
            hara.lang.data.Map.Standard.from(null, "native", new Object()));
    assertThrows(IllegalArgumentException.class, () -> StdJson.write(nonportable));
    assertThrows(
        IllegalArgumentException.class,
        () ->
            StdJson.read(
                "{\"$hara\":\"result\",\"status\":\"success\",\"data\":1,\"error\":{},\"context\":{}}"));
    Object generic =
        StdJson.read(
            "{\"$hara\":\"result\",\"status\":\"success\",\"data\":1,\"error\":null,\"context\":{},\"extra\":true}");
    assertTrue(generic instanceof hara.lang.data.types.IMapType<?, ?>);
  }

}
