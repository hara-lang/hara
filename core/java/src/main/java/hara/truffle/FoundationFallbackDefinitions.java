package hara.truffle;

import hara.lang.data.List;
import hara.lang.data.Symbol;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.util.LinkedHashSet;
import java.util.Set;

/**
 * Names whose complete Truffle semantics require the portable {@code std.foundation} startup.
 *
 * <p>The source is parsed once, on the first Truffle source compilation, but none of its
 * definitions are executed. This keeps demand exact for optimized Java exports that are replaced
 * or completed by a HAL definition. A small set of Java primitives also depends on protocol or
 * reader initialization performed while Foundation executes despite having no same-name HAL
 * definition.
 */
final class FoundationFallbackDefinitions {
  private static final String RESOURCE = "std/foundation.hal";
  private static final Set<String> NAMES = load();
  private static final Set<String> INITIALIZATION_DEPENDENCIES =
      Set.of(
          // Reader registration turns decimal text into the exact numeric value rather than
          // leaving the parsed representation as an interop map.
          "read-string",
          // These optimized collection operations dispatch through protocol implementations
          // installed while Foundation starts, including compact Tuple receivers.
          "assoc",
          "nth");

  private FoundationFallbackDefinitions() {}

  static boolean defines(String name) {
    return NAMES.contains(name);
  }

  static boolean requiresInitialization(String name) {
    return NAMES.contains(name) || INITIALIZATION_DEPENDENCIES.contains(name);
  }

  static boolean isInitializationDependency(String name) {
    return INITIALIZATION_DEPENDENCIES.contains(name);
  }

  static Set<String> names() {
    return NAMES;
  }

  private static Set<String> load() {
    ClassLoader loader = HaraContext.class.getClassLoader();
    try (InputStream input = loader.getResourceAsStream(RESOURCE)) {
      if (input == null) return Set.of();
      String source = new String(input.readAllBytes(), StandardCharsets.UTF_8);
      Object[] forms = HaraLanguage.readAll(source, RESOURCE);
      LinkedHashSet<String> names = new LinkedHashSet<>();
      for (Object form : forms) collect(form, names);
      return Set.copyOf(names);
    } catch (IOException error) {
      throw new HaraException("Unable to index " + RESOURCE + ": " + error.getMessage());
    }
  }

  private static void collect(Object form, Set<String> names) {
    if (!(form instanceof List<?> list)
        || list.count() == 0
        || !(list.nth(0) instanceof Symbol operator)
        || operator.getNamespace() != null) {
      return;
    }
    String operation = operator.getName();
    if ("do".equals(operation)) {
      for (int index = 1; index < list.count(); index++) collect(list.nth(index), names);
      return;
    }
    if ("declare".equals(operation)) {
      for (int index = 1; index < list.count(); index++) addSymbol(list.nth(index), names);
      return;
    }
    if (!Set.of(
            "def",
            "defn",
            "defn-",
            "defmacro",
            "defstruct",
            "defmutable",
            "defprotocol",
            "defmulti")
        .contains(operation)) {
      return;
    }
    if (list.count() < 2 || !(list.nth(1) instanceof Symbol name)) return;
    names.add(name.getName());
    if ("defstruct".equals(operation) || "defmutable".equals(operation)) {
      names.add("->" + name.getName());
      names.add("map->" + name.getName());
    }
    if ("defprotocol".equals(operation)) {
      for (int index = 2; index < list.count(); index++) {
        Object method = list.nth(index);
        if (method instanceof List<?> declaration && declaration.count() > 0) {
          addSymbol(declaration.nth(0), names);
        }
      }
    }
  }

  private static void addSymbol(Object value, Set<String> names) {
    if (value instanceof Symbol symbol && symbol.getNamespace() == null) {
      names.add(symbol.getName());
    }
  }
}
