package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import hara.lang.data.Keyword;
import org.graalvm.polyglot.Context;
import org.junit.Test;

public class SyntaxEnvironmentResultTest {
  @Test
  public void commentSuppressesAnalysisAndEvaluation() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertEquals(
          "[nil true]",
          context
              .eval(
                  HaraLanguage.ID,
                  "[(comment missing-symbol (throw (ex-info \"boom\" {})) (def leaked 1)) "
                      + "(special-symbol? 'comment)]")
              .toString());
    }
  }

  @Test
  public void portableRuntimeAndNativeResultContractsAreAvailable() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertEquals(
          "[user 42 42 42 true :std.native.Result :success nil]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(do (require 'std.lib.runtime) "
                      + "[(std.lib.runtime/current) "
                      + "(std.lib.runtime/eval-in 'user '[(+ 19 23)]) "
                      + "(std.lib.runtime/eval '(+ 19 23)) "
                      + "(std.lib.runtime/load-string \"(+ 19 23)\") "
                      + "(map? (std.lib.runtime/snapshot)) "
                      + "(type (Result/create :success 1)) "
                      + "(Result/status (Result/create :success 1)) "
                      + "(Result/context (Result/create :success 1))])")
              .toString());
    }
  }

  @Test
  public void resultRoundTripsThroughHtaAsANativeResult() {
    HaraResult result =
        HaraResult.success(
            42L,
            hara.lang.data.Map.Standard.from(
                null, Keyword.create("source"), Keyword.create("hta")));

    Object decoded = HtaValueCodec.decodeCanonical(HtaValueCodec.encode(result));

    assertTrue(decoded instanceof HaraResult);
    assertTrue(result.equality(decoded));
    assertEquals(
        Keyword.create("hta"), ((HaraResult) decoded).context().lookup(Keyword.create("source")));
  }
}
