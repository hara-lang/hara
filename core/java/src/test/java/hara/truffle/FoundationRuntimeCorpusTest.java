package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import hara.kernel.base.Parser;
import hara.lang.data.Keyword;
import hara.lang.data.types.ILinearType;
import hara.lang.data.types.IMapType;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashSet;
import java.util.Set;
import org.graalvm.polyglot.Context;
import org.junit.Test;

/** Runs the canonical Foundation forms shared with the Rust evaluator and bytecode VM. */
public class FoundationRuntimeCorpusTest {
  private static final Path CORPUS =
      specsRegistry()
          .resolve(
              "00-unsorted/platform-language/draft/conformance/parity/foundation-runtime.edn");

  @Test
  @SuppressWarnings("rawtypes")
  public void canonicalFoundationFormsMatchPinnedResults() throws Exception {
    IMapType manifest =
        (IMapType) Parser.LispReader.readString(Files.readString(CORPUS), null);
    Object rawCases = manifest.lookup(Keyword.create("cases"));
    assertTrue("Foundation runtime corpus must contain :cases", rawCases instanceof ILinearType);
    ILinearType cases = (ILinearType) rawCases;
    assertTrue("Foundation runtime corpus unexpectedly shrank", cases.count() >= 12);
    Set<Object> ids = new HashSet<>();
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      for (Object rawCase : cases) {
        IMapType testCase = (IMapType) rawCase;
        Object id = testCase.lookup(Keyword.create("id"));
        String source = (String) testCase.lookup(Keyword.create("source"));
        String expected = (String) testCase.lookup(Keyword.create("expect"));
        assertTrue("Duplicate Foundation runtime case " + id, ids.add(id));
        assertEquals(
            "Foundation runtime case " + id,
            expected,
            context.eval(HaraLanguage.ID, source).toString());
      }
    }
  }

  private static Path specsRegistry() {
    String override = System.getenv("HARA_SPECS_REGISTRY");
    return override == null || override.isBlank()
        ? Path.of("../hara-specs-registry")
        : Path.of(override);
  }
}
