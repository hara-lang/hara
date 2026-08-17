package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import java.util.UUID;
import org.graalvm.polyglot.Context;
import org.graalvm.polyglot.PolyglotException;
import org.junit.Test;

public class HaraWorkHostTest {
  @Test
  public void submissionReturnsBeforeTheNativeResultSettles() {
    String id = "immediate-" + UUID.randomUUID();
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      enterNamespace(context, "work.host.immediate");
      String state =
          context
              .eval(
                  HaraLanguage.ID,
                  "(do "
                      + "(def live-run "
                      + "  (work/work-submit work.native.protocol/default-host "
                      + "    :payload 7 "
                      + "    {:id \""
                      + id
                      + "\" "
                      + "     :work/execute "
                      + "     (fn [work input options run-id] "
                      + "       (promise/delay 1000 (fn [] [work input run-id])))})) "
                      + "(work/work-status live-run))")
              .toString();

      assertTrue(state.equals(":queued") || state.equals(":running"));
      assertEquals(
          "[:payload 7 \"" + id + "\"]",
          context
              .eval(HaraLanguage.ID, "(deref (work/work-result live-run))")
              .toString());
      assertEquals(
          ":completed",
          context
              .eval(HaraLanguage.ID, "(work/work-status live-run)")
              .toString());
    }
  }

  @Test
  public void resolvesTheSameCompletedRunFromAnIndependentSession() {
    String id = "cross-session-" + UUID.randomUUID();
    try (Context first = Context.newBuilder(HaraLanguage.ID).build();
        Context second = Context.newBuilder(HaraLanguage.ID).build()) {
      enterNamespace(first, "work.host.first");
      assertEquals(
          "42",
          first
              .eval(
                  HaraLanguage.ID,
                  "(do "
                      + "(def cross-run "
                      + "  (work/work-submit work.native.protocol/default-host "
                      + "    :payload nil "
                      + "    {:id \""
                      + id
                      + "\" "
                      + "     :work/execute (fn [work input options run-id] 42)})) "
                      + "(deref (work/work-result cross-run)))")
              .toString());

      enterNamespace(second, "work.host.second");
      assertEquals(
          "[\"" + id + "\" 42 :completed]",
          second
              .eval(
                  HaraLanguage.ID,
                  "(let [run (work/work-resolve work.native.protocol/default-host \""
                      + id
                      + "\")] "
                      + "  [(work/work-id run) "
                      + "   (deref (work/work-result run)) "
                      + "   (work/work-status run)])")
              .toString());
    }
  }

  @Test
  public void retainsStructuredFailureAndRejectsTheResultPromise() {
    String id = "failed-" + UUID.randomUUID();
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      enterNamespace(context, "work.host.failed");
      context.eval(
          HaraLanguage.ID,
          "(def failed-run "
              + "  (work/work-submit work.native.protocol/default-host "
              + "    :payload nil "
              + "    {:id \""
              + id
              + "\" "
              + "     :work/execute "
              + "     (fn [work input options run-id] "
              + "       (throw (ex-info \"work failed\" {:code :work/test})))}))");

      PolyglotException failure =
          assertThrows(
              PolyglotException.class,
              () -> context.eval(HaraLanguage.ID, "(deref (work/work-result failed-run))"));
      assertTrue(failure.getMessage().contains("work failed"));
      assertEquals(
          ":failed",
          context.eval(HaraLanguage.ID, "(work/work-status failed-run)").toString());
    }
  }

  @Test
  public void terminalStateAndResultCannotBeOverwrittenByLateCancellation() {
    String id = "terminal-" + UUID.randomUUID();
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      enterNamespace(context, "work.host.terminal");
      assertEquals(
          "[42 false :completed 42]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(do "
                      + "(def terminal-run "
                      + "  (work/work-submit work.native.protocol/default-host "
                      + "    :payload nil "
                      + "    {:id \""
                      + id
                      + "\" "
                      + "     :work/execute (fn [work input options run-id] 42)})) "
                      + "(let [value (deref (work/work-result terminal-run)) "
                      + "      cancelled (deref (work/work-cancel terminal-run :late))] "
                      + "  [value cancelled "
                      + "   (work/work-status terminal-run) "
                      + "   (deref (work/work-result terminal-run))]))")
              .toString());
    }
  }

  private static void enterNamespace(Context context, String name) {
    context.eval(
        HaraLanguage.ID,
        "(ns " + name + " (:require [std.work.protocol :as work]))");
  }
}
