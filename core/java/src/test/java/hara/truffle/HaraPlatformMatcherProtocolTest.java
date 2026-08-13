package hara.truffle;

import static org.junit.Assert.assertEquals;

import org.graalvm.polyglot.Context;
import org.junit.Test;

public final class HaraPlatformMatcherProtocolTest {
  @Test
  public void imatchIsInstalledBeforeAnyHalResourceIsLoaded() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertEquals(
          "[true true]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(require [std.protocol.imatch :as match]) "
                      + "(defstruct PlatformMatcher [expected]) "
                      + "(extend-type PlatformMatcher match/IMatch "
                      + "  (match-value [matcher actual] "
                      + "    (= (:expected matcher) actual))) "
                      + "[(= match/IMatch std.protocol.imatch/IMatch) "
                      + " (match/match-value (PlatformMatcher 42) 42)]")
              .toString());
    }
  }
}
