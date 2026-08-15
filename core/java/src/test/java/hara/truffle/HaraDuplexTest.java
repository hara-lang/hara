package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import org.graalvm.polyglot.Context;
import org.junit.Test;

public class HaraDuplexTest {
  @Test
  public void duplexHasStreamReceivePromiseSendAndIdempotentClose() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertEquals(
          "[true :std.native.Duplex true true :sent true]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(let [closed (atom 0) "
                      + "s (Stream/generate (fn [] (Coroutine/yield :received) :done)) "
                      + "d (Duplex/create s (fn [value] [:sent value]) (fn [] (swap! closed inc)))] "
                      + "(let [sent (first (deref (Duplex/send d :value)))] "
                      + "(IClose/close d) (Duplex/close d) "
                      + "[(Duplex/instance? d) (type d) (satisfies? IClose d) "
                      + "(= s (Duplex/receive d)) sent (= 1 (deref closed))]))")
              .toString());
    }
  }

  @Test
  public void closedDuplexRejectsSends() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      context.eval(
          HaraLanguage.ID,
          "(def closed-duplex (Duplex/create (Stream/generate (fn [] :done)) identity)) "
              + "(Duplex/close closed-duplex)");
      assertTrue(
          context
              .eval(HaraLanguage.ID, "(= :rejected (IPromise/state (Duplex/send closed-duplex :late)))")
              .asBoolean());
    }
  }

  @Test
  public void processDuplexStreamsStdoutAndWritesStdin() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).allowCreateProcess(true).build()) {
      assertEquals(
          "[\"reply:hello\" 0 true]",
          context.eval(
              HaraLanguage.ID,
              "(let [p (Process/spawn [\"sh\" \"-c\" \"read line; printf reply:$line\"]) "
                  + "d (Process/duplex p)] "
                  + "(deref (Duplex/send d (str/encode-utf8 \"hello\\n\"))) "
                  + "(Process/close-input p) "
                  + "[(str/decode-utf8 (deref (Stream/next (Duplex/receive d)))) "
                  + " (deref (Process/wait p)) "
                  + " (nil? (deref (Stream/next (Duplex/receive d))))])")
              .toString());
    }
  }

}
