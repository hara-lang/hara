package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

import org.graalvm.polyglot.Context;
import org.junit.Test;

public class FoundationFallbackDemandTest {
  @Test
  public void builtinAndClosedLexicalSourceDoesNotMaterializeFoundation() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertEquals(42L, context.eval(HaraLanguage.ID, "(+ 19 23)").asLong());
      assertTrue(
          context
              .eval(HaraLanguage.ID, "(= nil (resolve 'map))")
              .asBoolean());

      assertEquals(
          42L,
          context
              .eval(
                  HaraLanguage.ID,
                  "(do (defn local-successor [x] (+ x 1)) (local-successor 41))")
              .asLong());
      assertTrue(
          context
              .eval(HaraLanguage.ID, "(= nil (resolve 'map))")
              .asBoolean());
    }
  }

  @Test
  public void firstFallbackFunctionReferenceMaterializesFoundation() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertTrue(
          context
              .eval(HaraLanguage.ID, "(= nil (resolve 'map))")
              .asBoolean());

      assertTrue(
          context
              .eval(
                  HaraLanguage.ID,
                  "(= [2 3] (map (fn [value] (+ value 1)) [1 2]))")
              .asBoolean());
      assertFalse(
          context
              .eval(HaraLanguage.ID, "(= nil (resolve 'map))")
              .asBoolean());
    }
  }

  @Test
  public void firstFallbackMacroReferenceMaterializesFoundation() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertTrue(
          context
              .eval(HaraLanguage.ID, "(= nil (resolve 'if-not))")
              .asBoolean());
      assertEquals(42L, context.eval(HaraLanguage.ID, "(if-not false 42)").asLong());
      assertFalse(
          context
              .eval(HaraLanguage.ID, "(= nil (resolve 'if-not))")
              .asBoolean());
    }
  }

  @Test
  public void selectiveNamespacePolicySurvivesLaterFallbackUse() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      context.eval(
          HaraLanguage.ID,
          "(ns startup-selective (:config {:expose [map count]}))");

      assertTrue(
          context
              .eval(
                  HaraLanguage.ID,
                  "(and (= 2 (count (map (fn [value] value) [1 2]))) "
                      + "     (= nil (resolve 'inc)))")
              .asBoolean());
    }
  }
}
