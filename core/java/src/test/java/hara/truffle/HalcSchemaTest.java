package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import hara.lang.base.G;
import hara.lang.data.Keyword;
import hara.lang.data.Symbol;
import hara.lang.data.types.ILinearType;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import org.junit.Test;

public class HalcSchemaTest {
  @Test
  public void matchesSharedStdTypedParityCorpus() throws IOException {
    for (Object parityCase : parityCases()) {
      ILinearType<?> items = vector(parityCase);
      assertNotNull("parity case must be a vector", items);
      Keyword operation = (Keyword) items.nth(0);
      String id = G.display(items.nth(1));
      switch (operation.getName()) {
        case "normalize" ->
            assertEquals(
                id,
                stringAt(items, 3),
                canonical(HalcSchema.normalize(parseOne(stringAt(items, 2)))));
        case "error" -> {
          HaraException error =
              assertThrows(
                  id,
                  HaraException.class,
                  () -> HalcSchema.normalize(parseOne(stringAt(items, 2))));
          assertEquals(id, ((Keyword) items.nth(3)).getName(), errorCode(error));
        }
        case "assignable" ->
            assertEquals(
                id,
                items.nth(4),
                HalcSchema.assignable(
                    HalcSchema.normalize(parseOne(stringAt(items, 2))),
                    HalcSchema.normalize(parseOne(stringAt(items, 3)))));
        case "compatible" ->
            assertEquals(
                id,
                items.nth(4),
                HalcSchema.compatible(
                    HalcSchema.normalize(parseOne(stringAt(items, 2))),
                    HalcSchema.normalize(parseOne(stringAt(items, 3)))));
        case "infer" ->
            assertEquals(
                id,
                stringAt(items, 3),
                canonical(HalcSchema.infer(parseOne(stringAt(items, 2)), Map.of(), Map.of())));
        case "infer-call" -> {
          String name = stringAt(items, 3);
          HalcSchema.Type contract = HalcSchema.normalize(parseOne(stringAt(items, 4)));
          assertEquals(
              id,
              stringAt(items, 5),
              canonical(
                  HalcSchema.infer(
                      parseOne(stringAt(items, 2)), Map.of(), Map.of(name, contract))));
        }
        default -> throw new AssertionError("unsupported parity operation: " + operation.getName());
      }
    }
  }

  @Test
  public void normalizesNestedNamedFunctionSchemas() {
    Object schema = parseOne("[:fn [#'demo/Customer & :int] [:maybe :str]]");
    assertEquals(
        new HalcSchema.FunctionType(
            List.of(
                new HalcSchema.Function(
                    List.of(new HalcSchema.Reference("demo/Customer")),
                    new HalcSchema.Primitive("int"),
                    new HalcSchema.Union(
                        List.of(
                            new HalcSchema.Primitive("str"),
                            new HalcSchema.Primitive("nil")))))),
        HalcSchema.normalize(schema));
  }

  @Test
  public void resolvesRecursiveReferencesWithoutExpandingCycles() {
    HalcSchema.Type node =
        HalcSchema.normalize(parseOne("[:map [:next [:maybe #'demo/Node]]]") );
    HalcSchema.Type resolved =
        HalcSchema.resolve(
            new HalcSchema.Reference("demo/Node"), Map.of("demo/Node", node));
    assertEquals(
        "map[:next=union[reference(demo/Node),primitive(:nil)]]", canonical(resolved));
  }

  @Test
  public void separatesCompatibilityFromDirectionalAssignment() {
    HalcSchema.Type number = HalcSchema.normalize(parseOne(":num"));
    HalcSchema.Type integer = HalcSchema.normalize(parseOne(":int"));
    assertTrue(HalcSchema.compatible(number, integer));
    assertTrue(HalcSchema.assignable(number, integer));
    assertFalse(HalcSchema.assignable(integer, number));
  }

  @Test
  public void infersBodyResultsSeparatelyFromDeclaredContracts() {
    Object[] forms =
        HaraLanguage.readAll(
            "(ns demo)\n"
                + "(def Unary [:fn [:int] :num])\n"
                + "(defn ^{:schema #'demo/Unary} choose [value] "
                + "  (let [next (+ value 1)] (if true next 0)))\n"
                + "(defn labels [] {:name \"Ada\" :active true})\n"
                + "(defn select ([value] value) ([left right] right))",
            "typed.hal");
    Map<String, HalcSchema.Type> declarations =
        Map.of("demo/choose", new HalcSchema.Reference("demo/Unary"));
    Map<String, HalcSchema.Type> definitions =
        Map.of("demo/Unary", HalcSchema.normalize(parseOne("[:fn [:int] :num]")));

    Map<String, HalcSchema.Type> inferred =
        HalcSchema.inferFunctionTypes("demo", forms, declarations, definitions);
    HalcSchema.Function choose =
        ((HalcSchema.FunctionType) inferred.get("demo/choose")).arities().get(0);
    assertEquals(List.of(new HalcSchema.Primitive("int")), choose.fixed());
    assertEquals(new HalcSchema.Primitive("int"), choose.output());
    HalcSchema.Function labels =
        ((HalcSchema.FunctionType) inferred.get("demo/labels")).arities().get(0);
    assertTrue(labels.output() instanceof HalcSchema.MapType);
    assertEquals(
        2, ((HalcSchema.FunctionType) inferred.get("demo/select")).arities().size());
  }

  private static Object parseOne(String source) {
    Object[] forms = HaraLanguage.readAll(source, "typed-schema.hal");
    if (forms.length != 1) throw new AssertionError("expected one form: " + source);
    return forms[0];
  }

  private static List<Object> parityCases() throws IOException {
    String source;
    try (InputStream stream =
        HalcSchemaTest.class.getResourceAsStream("/std/typed/parity_corpus.hal")) {
      assertNotNull("shared std.typed parity corpus is on the test classpath", stream);
      source = new String(stream.readAllBytes(), StandardCharsets.UTF_8);
    }
    for (Object form : HaraLanguage.readAll(source, "std/typed/parity_corpus.hal")) {
      if (!(form instanceof hara.lang.data.List<?> definition) || definition.count() < 3) continue;
      if (!(definition.nth(0) instanceof Symbol operator)
          || !"def".equals(operator.getName())) continue;
      if (!(definition.nth(1) instanceof Symbol name)
          || !"+cases+".equals(name.getName())) continue;
      ILinearType<?> cases = vector(definition.nth(2));
      if (cases == null) continue;
      List<Object> output = new ArrayList<>();
      for (int index = 0; index < cases.count(); index++) output.add(cases.nth(index));
      return output;
    }
    throw new AssertionError("shared parity corpus does not define +cases+");
  }

  private static String stringAt(ILinearType<?> items, int index) {
    Object value = items.nth(index);
    if (!(value instanceof String source)) {
      throw new AssertionError("expected string at " + index + ", got " + G.display(value));
    }
    return source;
  }

  private static ILinearType<?> vector(Object value) {
    return value instanceof ILinearType<?> linear && "[".equals(linear.startString())
        ? linear
        : null;
  }

  private static String errorCode(Throwable error) {
    String message = error.getMessage();
    int separator = message.indexOf(':');
    return separator < 0 ? message : message.substring(0, separator);
  }

  private static String canonical(HalcSchema.Type schema) {
    if (schema instanceof HalcSchema.Primitive primitive) {
      return "primitive(:" + primitive.name() + ")";
    }
    if (schema instanceof HalcSchema.Reference reference) {
      return "reference(" + reference.name() + ")";
    }
    if (schema instanceof HalcSchema.Union union) {
      List<String> members = new ArrayList<>();
      for (HalcSchema.Type member : union.types()) members.add(canonical(member));
      return "union[" + String.join(",", members) + "]";
    }
    if (schema instanceof HalcSchema.VectorType vector) {
      return "vector(" + canonical(vector.item()) + ")";
    }
    if (schema instanceof HalcSchema.Tuple tuple) {
      List<String> members = new ArrayList<>();
      for (HalcSchema.Type member : tuple.items()) members.add(canonical(member));
      return "tuple[" + String.join(",", members) + "]";
    }
    if (schema instanceof HalcSchema.MapType map) {
      List<String> fields = new ArrayList<>();
      for (HalcSchema.Field field : map.fields()) {
        fields.add(G.display(field.name()) + "=" + canonical(field.type()));
      }
      return "map[" + String.join(",", fields) + "]";
    }
    if (schema instanceof HalcSchema.FunctionType functions) {
      List<String> arities = new ArrayList<>();
      for (HalcSchema.Function function : functions.arities()) {
        List<String> fixed = new ArrayList<>();
        for (HalcSchema.Type input : function.fixed()) fixed.add(canonical(input));
        arities.add(
            "fn(fixed=["
                + String.join(",", fixed)
                + "],rest="
                + (function.rest() == null ? "none" : canonical(function.rest()))
                + ",output="
                + canonical(function.output())
                + ")");
      }
      return arities.size() == 1
          ? arities.get(0)
          : "function[" + String.join(",", arities) + "]";
    }
    if (schema instanceof HalcSchema.EnumType enumeration) {
      List<String> values = new ArrayList<>();
      for (Object value : enumeration.values()) values.add(G.display(value));
      return "enum[" + String.join(",", values) + "]";
    }
    if (schema instanceof HalcSchema.Unknown) return "unknown";
    if (schema instanceof HalcSchema.Extension extension) {
      return "extension(" + extension.head() + ")";
    }
    throw new AssertionError("unsupported schema type: " + schema);
  }
}
