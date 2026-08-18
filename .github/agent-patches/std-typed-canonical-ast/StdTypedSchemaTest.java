package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

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

  @Test
  public void nativeSchemaAstIsThePortableNormalForm() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      String actual =
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns typed-schema-ast-truffle-probe) "
                      + "(require 'std.typed.schema {:reload true}) "
                      + "(defn schema-ast-pair [surface] "
                      + "  (let [ast (Schema/ast (std.foundation/schema surface))] "
                      + "    [ast (std.typed.schema/normalize ast)])) "
                      + "(let [union (quote [:or :int :str :int]) "
                      + "      vector-schema (quote [:vector [:maybe :int]]) "
                      + "      map-schema (quote [:map [:name :str] "
                      + "                               [:tags [:vector :keyword]]]) "
                      + "      fn-schema (quote [:fn [:str & :any] :str]) "
                      + "      function-schema "
                      + "      (quote [:function [:fn [:int] :int] "
                      + "                        [:fn [:str & :any] :str]]) "
                      + "      extension (quote [:test/tagged 42])] "
                      + "  (pr-str "
                      + "   {:union (schema-ast-pair union) "
                      + "    :vector (schema-ast-pair vector-schema) "
                      + "    :map (schema-ast-pair map-schema) "
                      + "    :fn (schema-ast-pair fn-schema) "
                      + "    :function (schema-ast-pair function-schema) "
                      + "    :extension [(std.typed.schema/normalize extension) "
                      + "                (Schema/ast "
                      + "                 (std.foundation/schema extension))]}))")
              .asString();
      assertTrue(actual, false);
    }
  }
}
