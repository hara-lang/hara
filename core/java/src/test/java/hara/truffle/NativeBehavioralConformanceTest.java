package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import java.lang.reflect.Field;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.Value;
import org.junit.Test;

/** Runs the source-owned native behavioral corpus and guards exact live-surface closure. */
public class NativeBehavioralConformanceTest {
  private static final Path CORPUS =
      Path.of("lib/test-fixtures/std/foundation/native_method_conformance.hal");
  private static final Path RUST_LOCAL_COPY =
      Path.of("rust/hal-test-fixtures/std/foundation/native_method_conformance.hal");

  @Test
  public void sharedCorpusClosesOverTheLiveManifestAndRejectsDrift() throws Exception {
    String corpus = Files.readString(CORPUS);
    Set<String> classified;
    try (Context context = Context.newBuilder(HaraLanguage.ID).allowAllAccess(true).build()) {
      assertEquals(
          "true",
          context.eval(HaraLanguage.ID, corpus + "\n(native-corpus-valid?)").toString());
      classified =
          methodSet(
              context.eval(
                  HaraLanguage.ID, corpus + "\n(native-method-keys)"));
      System.out.println(
          "native behavioral classifications "
              + context
                  .eval(
                      HaraLanguage.ID,
                      corpus + "\n(native-classification-summary)")
                  .toString());
    }

    Set<String> live = liveMethods();
    assertClosed(live, classified);
    assertFalse(
        "The Rust-local fixture must not diverge from the source-owned corpus",
        Files.exists(RUST_LOCAL_COPY));

    Set<String> removed = new LinkedHashSet<>(classified);
    String first = removed.iterator().next();
    removed.remove(first);
    assertThrows(AssertionError.class, () -> assertClosed(live, removed));

    Set<String> added = new LinkedHashSet<>(classified);
    added.add("Unclassified/addition");
    assertThrows(AssertionError.class, () -> assertClosed(live, added));

    Set<String> renamed = new LinkedHashSet<>(classified);
    renamed.remove(first);
    renamed.add(first + "-renamed");
    assertThrows(AssertionError.class, () -> assertClosed(live, renamed));
  }

  @Test
  public void truffleRunsEveryClassificationAndPortableBoundaryProbe() throws Exception {
    String corpus = Files.readString(CORPUS);
    try (Context context = Context.newBuilder(HaraLanguage.ID).allowAllAccess(true).build()) {
      Set<String> methods =
          methodSet(context.eval(HaraLanguage.ID, corpus + "\n(native-method-keys)"));
      StringBuilder source = new StringBuilder(corpus).append("\n[");
      for (String method : methods) {
        source.append("(native-method-result '").append(method).append(" nil) ");
      }
      source.append("]");

      String result = context.eval(HaraLanguage.ID, source.toString()).toString();
      assertTrue(result, !result.contains(":pass false"));
      assertEquals(methods.size(), result.split(":pass true", -1).length - 1);

      assertEquals(
          "[true true true true true true true true true true true true]",
          context
              .eval(HaraLanguage.ID, corpus + "\n(native-boundary-report)")
              .toString());
    }
  }

  @Test
  public void identityFastPathsRemainExplicitInTheJvmImplementation() throws Exception {
    String source =
        Files.readString(Path.of("java/src/main/java/hara/truffle/HaraContext.java"));
    assertTrue(
        "Base/vec must return an existing persistent vector unchanged",
        source.contains("instanceof hara.lang.data.Vector<?>")
            && source.contains("return value;"));
    assertTrue(
        "Base/set must recognize every persistent set before materializing",
        source.contains("instanceof hara.lang.data.types.ISetType<?>"));
  }

  private static Set<String> methodSet(Value value) {
    assertTrue("native method keys must be a vector", value.hasArrayElements());
    Set<String> methods = new LinkedHashSet<>();
    for (long index = 0; index < value.getArraySize(); index++) {
      String method = value.getArrayElement(index).toString();
      assertTrue("duplicate native corpus method " + method, methods.add(method));
    }
    assertFalse("the native corpus must not be empty", methods.isEmpty());
    return methods;
  }

  private static Set<String> liveMethods() throws Exception {
    Field field = HaraContext.class.getDeclaredField("NATIVE_TYPES");
    field.setAccessible(true);
    @SuppressWarnings("unchecked")
    Map<String, List<String>> runtimeTypes =
        (Map<String, List<String>>) field.get(null);
    Set<String> methods = new LinkedHashSet<>();
    for (Map.Entry<String, List<String>> type : runtimeTypes.entrySet()) {
      for (String method : type.getValue()) {
        assertTrue(
            "Duplicate live native method: " + type.getKey() + "/" + method,
            methods.add(type.getKey() + "/" + method));
      }
    }
    return methods;
  }

  private static void assertClosed(Set<String> live, Set<String> classified) {
    Set<String> missing = new LinkedHashSet<>(live);
    missing.removeAll(classified);
    Set<String> extra = new LinkedHashSet<>(classified);
    extra.removeAll(live);
    assertTrue(
        "Native behavioral closure mismatch; missing=" + missing + ", extra=" + extra,
        missing.isEmpty() && extra.isEmpty());
  }
}
