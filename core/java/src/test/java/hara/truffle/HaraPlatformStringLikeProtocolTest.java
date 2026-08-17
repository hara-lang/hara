package hara.truffle;

import static org.junit.Assert.assertEquals;

import org.graalvm.polyglot.Context;
import org.junit.Test;

public final class HaraPlatformStringLikeProtocolTest {
  @Test
  public void istringlikeIsInstalledBeforeAnyHalResourceIsLoaded() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertEquals(
          "[true \"hello/world\" :hello/world \"custom\" \"restored\"]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(defstruct WrappedName [value]) "
                      + "(extend-type WrappedName std.protocol.istringlike/IStringLike "
                      + "  (to-string [wrapped] (:value wrapped)) "
                      + "  (from-string [wrapped text] (WrappedName text))) "
                      + "[(satisfies? std.protocol.istringlike/IStringLike :hello) "
                      + " (std.protocol.istringlike/to-string :hello/world) "
                      + " (std.protocol.istringlike/from-string :sample \"hello/world\") "
                      + " (std.protocol.istringlike/to-string (WrappedName \"custom\")) "
                      + " (:value (std.protocol.istringlike/from-string "
                      + "          (WrappedName \"\") \"restored\"))]")
              .toString());
    }
  }
}
