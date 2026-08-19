package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;

import hara.kernel.base.Parser;
import hara.lang.data.Keyword;
import hara.lang.data.Symbol;
import hara.lang.data.types.ILinearType;
import hara.lang.data.types.IMapType;
import java.lang.reflect.Field;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import org.graalvm.polyglot.Context;
import org.junit.Test;

/** Keeps the closed std.native inventory and its direct method surface aligned with the spec. */
public class NativeMethodParityTest {
  private static final Path CONTRACT =
      specsRegistry()
          .resolve("00-unsorted/platform-language/draft/conformance/native.edn");
  private static final Path FIXTURE =
      Path.of("lib/test-fixtures/std/foundation/native_method_conformance.hal");

  @Test
  public void nativeInventoryIsClosedAndClassified() throws Exception {
    IMapType contract = readMap(CONTRACT);
    IMapType inventory = map(contract, "inventory");
    Map<String, NativeTypeSpec> types = types(contract);

    assertEquals(Boolean.TRUE, inventory.lookup(keyword("closed")));
    assertEquals(((Number) inventory.lookup(keyword("type-count"))).intValue(), types.size());
    assertEquals(
        ((Number) inventory.lookup(keyword("method-count"))).intValue(),
        types.values().stream().mapToInt(type -> type.methods.size()).sum());

    Field field = HaraContext.class.getDeclaredField("NATIVE_TYPES");
    field.setAccessible(true);
    @SuppressWarnings("unchecked")
    Map<String, List<String>> runtimeTypes = (Map<String, List<String>>) field.get(null);
    Map<String, List<String>> specifiedTypes = new LinkedHashMap<>();
    types.forEach((name, type) -> specifiedTypes.put(name, type.methods));
    assertEquals("Truffle native inventory differs from native.edn", specifiedTypes, runtimeTypes);

    for (NativeTypeSpec type : types.values()) {
      assertTrue(
          "Unsupported availability classification for " + type.name,
          Set.of("implemented", "capability-gated").contains(type.availability));
      Set<String> classified = new LinkedHashSet<>(type.halWrappers);
      for (String primitive : type.foundationPrimitives) {
        assertTrue("Duplicate method classification: " + type.name + "/" + primitive,
            classified.add(primitive));
      }
      for (String nativeOnly : type.nativeOnly) {
        assertTrue("Duplicate method classification: " + type.name + "/" + nativeOnly,
            classified.add(nativeOnly));
      }
      assertEquals(
          "Every native method must have exactly one Foundation exposure: " + type.name,
          new LinkedHashSet<>(type.methods),
          classified);
      if (!type.halWrappers.isEmpty()) {
        assertNotNull("HAL wrappers require a source: " + type.name, type.wrapperSource);
        String source = Files.readString(Path.of(type.wrapperSource));
        for (String method : type.halWrappers) {
          assertTrue(
              "Missing HAL wrapper call " + type.name + "/" + method,
              source.contains(type.name + "/" + method));
        }
      }
    }
  }

  @Test
  public void languageBuiltinAccountingMatchesTheSharedContract() throws Exception {
    IMapType builtins = map(readMap(CONTRACT), "language-builtins");
    Field field = HaraContext.class.getDeclaredField("LANGUAGE_BUILTINS");
    field.setAccessible(true);
    @SuppressWarnings("unchecked")
    Map<String, List<String>> runtime = (Map<String, List<String>>) field.get(null);
    Map<String, List<String>> specified = new LinkedHashMap<>();
    for (String category : List.of("evaluation", "definitions", "namespaces", "interop")) {
      specified.put(category, symbols(builtins.lookup(keyword(category)), category));
    }
    assertEquals("Truffle builtin accounting differs from native.edn", specified, runtime);
    assertTrue("Builtins must not be a native type", !types(readMap(CONTRACT)).containsKey("Builtins"));
  }

  @Test
  public void removedFoundationPathwaysCannotBeReintroducedSilently() throws Exception {
    Path javaRoot = Files.isDirectory(Path.of("java/src/main")) ? Path.of("java") : Path.of(".");
    Path sourceRoot = javaRoot.resolve("src/main/java/hara/truffle");
    String context = Files.readString(sourceRoot.resolve("HaraContext.java"));
    String collection =
        Files.readString(sourceRoot.resolve("StdFoundationCollection.java"));
    for (String removed :
        List.of(
            "std.native.Base/pair?",
            "Base/unreduced",
            "std.native.Builtins",
            "reduceKeyValues(",
            "mergeMaps(",
            "selectKeys(",
            "reduceIterator(")) {
      assertTrue("Removed Foundation pathway returned: " + removed,
          !context.contains(removed) && !collection.contains(removed));
    }
    assertTrue(
        "Source-owned sequence library must not be restored",
        !Files.exists(sourceRoot.resolve("StdFoundationSequence.java")));
    for (String sourceOwned : List.of("reduce", "reduce-kv", "merge", "select-keys")) {
      assertTrue(
          "HAL-owned function was restored as a Java export: " + sourceOwned,
          !collection.contains("name = \"" + sourceOwned + "\""));
    }
    Path foundationRoot = Path.of("lib/src/std/foundation.hal");
    List<Path> foundationSources = new ArrayList<>(List.of(foundationRoot));
    try (var paths = Files.walk(Path.of("lib/src/std/foundation"))) {
      paths.filter(path -> path.toString().endsWith(".hal")).forEach(foundationSources::add);
    }
    for (Path sourcePath : foundationSources) {
      String source = Files.readString(sourcePath);
      assertTrue(
          "Foundation source must use has?, never contains?: " + sourcePath,
          !source.contains("contains?"));
      assertTrue(
          "decimal? is not part of the language contract: " + sourcePath,
          !source.contains("decimal?"));
    }
    assertTrue(
        "Base must not export the non-spec decimal? predicate",
        !context.contains("\"decimal?\""));
    assertTrue(
        "vec identity fast path must return the original value",
        context.contains("instanceof hara.lang.data.Vector<?>") && context.contains("return value;"));
    assertTrue(
        "set identity fast path must recognize every persistent set",
        context.contains("instanceof hara.lang.data.types.ISetType<?>"));
  }

  @Test
  public void everySpecifiedNativeMethodIsDirectlyCallable() throws Exception {
    Map<String, NativeTypeSpec> types = types(readMap(CONTRACT));
    StringBuilder source = new StringBuilder(Files.readString(FIXTURE)).append("\n[");
    for (NativeTypeSpec type : types.values()) {
      for (String method : type.methods) {
        String symbol = type.name + "/" + method;
        source
            .append("(native-method-result '")
            .append(symbol)
            .append(" (fn [] (")
            .append(symbol)
            .append(" nil nil nil nil nil nil nil nil nil))) ");
      }
    }
    source.append("]");

    try (Context context = Context.newBuilder(HaraLanguage.ID).allowAllAccess(true).build()) {
      String result = context.eval(HaraLanguage.ID, source.toString()).toString();
      assertTrue(result, !result.contains(":pass false"));
      assertEquals(
          types.values().stream().mapToInt(type -> type.methods.size()).sum(),
          result.split(":pass true", -1).length - 1);
      assertEquals(
          "[\"native failure\" true]",
          context
              .eval(
                  HaraLanguage.ID,
                  "[(Error/message "
                      + "(Error/new \"native failure\" {})) "
                      + "(string? (Error/class "
                      + "(Error/new \"native failure\" {})))]")
              .toString());
    }
  }

  @Test
  public void nativeTypeObjectsAndAliasesAreUniversalIncludingBlankNamespaces() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).allowAllAccess(true).build()) {
      assertEquals(
          "[true 1]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns blank.native (:config {:blank true})) "
                      + "[(std.foundation/= Iter std.native.Iter) "
                      + " (Iter/iter-next (Iter/iter-map (fn [value] value) [1]))]")
              .toString());
    }
  }

  private static Map<String, NativeTypeSpec> types(IMapType contract) {
    ILinearType entries = linear(contract.lookup(keyword("types")), ":types");
    Map<String, NativeTypeSpec> output = new LinkedHashMap<>();
    for (int index = 0; index < entries.count(); index++) {
      IMapType entry = (IMapType) entries.nth(index);
      String name = ((Symbol) entry.lookup(keyword("name"))).getName();
      List<String> methods = symbols(entry.lookup(keyword("methods")), name + " :methods");
      assertEquals(
          "Duplicate methods declared for " + name,
          new LinkedHashSet<>(methods).size(),
          methods.size());
      String availability = ((Keyword) entry.lookup(keyword("availability"))).getName();
      IMapType classification = map(entry, "method-classification");
      List<String> halWrappers = classified(classification.lookup(keyword("hal-wrapper")), methods);
      List<String> primitives =
          classified(classification.lookup(keyword("foundation-primitive")), methods);
      List<String> nativeOnly =
          classified(classification.lookup(keyword("native-only")), methods);
      String wrapperSource = (String) entry.lookup(keyword("wrapper-source"));
      assertTrue("Duplicate native type: " + name, !output.containsKey(name));
      output.put(
          name,
          new NativeTypeSpec(
              name, methods, availability, halWrappers, primitives, nativeOnly, wrapperSource));
    }
    return output;
  }

  private static List<String> classified(Object value, List<String> all) {
    if (value == null) return List.of();
    if (value instanceof Keyword marker && "all".equals(marker.getName())) {
      return all;
    }
    return symbols(value, "method classification");
  }

  private static List<String> symbols(Object value, String label) {
    ILinearType values = linear(value, label);
    List<String> output = new ArrayList<>();
    for (int index = 0; index < values.count(); index++) {
      output.add(((Symbol) values.nth(index)).getName());
    }
    return List.copyOf(output);
  }

  private static IMapType readMap(Path path) throws Exception {
    Object value = Parser.LispReader.readString(Files.readString(path), null);
    assertTrue("Expected EDN map: " + path, value instanceof IMapType);
    return (IMapType) value;
  }

  private static IMapType map(IMapType parent, String name) {
    Object value = parent.lookup(keyword(name));
    assertTrue("Expected map at :" + name, value instanceof IMapType);
    return (IMapType) value;
  }

  private static ILinearType linear(Object value, String label) {
    assertTrue("Expected vector at " + label, value instanceof ILinearType);
    return (ILinearType) value;
  }

  private static Keyword keyword(String name) {
    return Keyword.create(name);
  }

  private static Path specsRegistry() {
    String override = System.getenv("HARA_SPECS_REGISTRY");
    return override == null || override.isBlank()
        ? Path.of("../hara-specs-registry")
        : Path.of(override);
  }

  private record NativeTypeSpec(
      String name,
      List<String> methods,
      String availability,
      List<String> halWrappers,
      List<String> foundationPrimitives,
      List<String> nativeOnly,
      String wrapperSource) {}
}
