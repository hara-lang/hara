package hara.truffle;

import static org.junit.Assert.assertEquals;

import org.graalvm.polyglot.Context;
import org.junit.Test;

public class StdTypedSchemaTest {
  @Test
  public void portableSchemaAcceptsCanonicalAndNativeForms() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertEquals(
          "[true true true false true false true]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns typed-schema-truffle-probe) "
                      + "(require 'std.typed.schema {:reload true}) "
                      + "(let [primitive (std.foundation/schema :int) "
                      + "      user (std.foundation/schema [:map [:name :str]])] "
                      + "  (pr-str "
                      + "    [(= (std.typed.schema/normalize :int) "
                      + "        (std.typed.schema/normalize [:int])) "
                      + "     (= (std.typed.schema/normalize :int) "
                      + "        (std.typed.schema/normalize primitive)) "
                      + "     (std.typed.schema/valid? [:int] 42) "
                      + "     (std.typed.schema/valid? [:int] \"42\") "
                      + "     (std.typed.schema/valid? user {:name \"Ada\"}) "
                      + "     (std.typed.schema/valid? user {:name 42}) "
                      + "     (std.typed.schema/compatible? primitive :int)]))")
              .asString());
    }
  }
}
