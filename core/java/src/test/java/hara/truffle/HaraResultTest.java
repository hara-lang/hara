package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertSame;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import hara.lang.base.Ex;
import hara.lang.data.Keyword;
import hara.lang.data.types.IMapType;
import hara.lang.protocol.Constant;
import org.graalvm.polyglot.Context;
import org.junit.Test;

public class HaraResultTest {
  @Test
  public void equalityAndHashIgnoreContext() {
    IMapType<Object, Object> leftContext =
        hara.lang.data.Map.Standard.from(
            null, Keyword.create("source"), Keyword.create("left"));
    IMapType<Object, Object> rightContext =
        hara.lang.data.Map.Standard.from(
            null, Keyword.create("source"), Keyword.create("right"));
    HaraResult left = HaraResult.success(42L, leftContext);
    HaraResult right = HaraResult.success(42L, rightContext);

    assertTrue(left.equality(right));
    assertEquals(left.hashCalc(Constant.HashType.RAPID), right.hashCalc(Constant.HashType.RAPID));
    assertEquals(42L, left.deref());
    assertEquals(Keyword.create("success"), left.status());
  }

  @Test
  public void contextMergeUsesSuppliedKeysWithoutChangingOutcome() {
    HaraResult result =
        HaraResult.success(
            7L,
            hara.lang.data.Map.Standard.from(
                null,
                Keyword.create("source"),
                Keyword.create("left"),
                Keyword.create("kept"),
                Boolean.TRUE));
    HaraResult updated =
        result.withContext(
            hara.lang.data.Map.Standard.from(
                null,
                Keyword.create("source"),
                Keyword.create("right"),
                Keyword.create("added"),
                1L));

    assertTrue(result.equality(updated));
    assertEquals(Keyword.create("right"), updated.context().lookup(Keyword.create("source")));
    assertEquals(Boolean.TRUE, updated.context().lookup(Keyword.create("kept")));
    assertEquals(1L, updated.context().lookup(Keyword.create("added")));
  }

  @Test
  public void errorDerefThrowsThePreservedNativeError() {
    Ex.Info error =
        new Ex.Info(
            "boom",
            hara.lang.data.Map.Standard.from(
                null, Keyword.create("code"), Keyword.create("boom")));
    HaraResult result = HaraResult.error(error);

    assertSame(error, result.errorValue());
    Ex.Info thrown = assertThrows(Ex.Info.class, () -> result.deref());
    assertSame(error, thrown);
    assertTrue(result.display().startsWith("#hara/Result[:error"));
  }

  @Test
  public void languageExportsExposeTheNativeContract() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertTrue(
          context
              .eval(
                  HaraLanguage.ID,
                  "(do "
                      + "(def r (std.native.Result/success 42 {:source :left})) "
                      + "(def e (std.native.Result/error (ex-info \"boom\" {:code :boom}) {:source :test})) "
                      + "(and "
                      + "(= :hara/Result (type r)) "
                      + "(std.native.Result/result? r) "
                      + "(std.native.Result/success? r) "
                      + "(not (std.native.Result/error? r)) "
                      + "(= :success (std.native.Result/status r)) "
                      + "(= 42 (std.native.Result/data r)) "
                      + "(= nil (std.native.Result/error-value r)) "
                      + "(= 42 (deref r)) "
                      + "(= r (std.native.Result/success 42 {:source :right})) "
                      + "(= :right (get (std.native.Result/context "
                      + "(std.native.Result/with-context r {:source :right})) :source)) "
                      + "(std.native.Result/error? e) "
                      + "(= :hara/Error (type (std.native.Result/error-value e)))))")
              .asBoolean());
    }
  }
}
