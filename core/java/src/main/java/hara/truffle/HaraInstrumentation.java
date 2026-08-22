package hara.truffle;

import com.oracle.truffle.api.frame.VirtualFrame;
import com.oracle.truffle.api.instrumentation.EventContext;
import com.oracle.truffle.api.instrumentation.ExecutionEventListener;
import com.oracle.truffle.api.instrumentation.SourceSectionFilter;
import com.oracle.truffle.api.instrumentation.StandardTags;
import com.oracle.truffle.api.instrumentation.TruffleInstrument;
import com.oracle.truffle.api.source.SourceSection;
import hara.truffle.InstrumentationModel.EventKind;
import hara.truffle.node.HaraNodes;
import java.util.Map;

@TruffleInstrument.Registration(
    id = "hara-execution",
    name = "Hara execution instrumentation",
    version = "0.1")
public final class HaraInstrumentation extends TruffleInstrument {
  @Override
  protected void onCreate(Env env) {
    attach(
        env,
        SourceSectionFilter.newBuilder()
            .tagIs(StandardTags.ExpressionTag.class)
            .build(),
        EventKind.SEMANTIC_BOUNDARY,
        true);
    attach(
        env,
        SourceSectionFilter.newBuilder().tagIs(StandardTags.CallTag.class).build(),
        EventKind.CALL_ENTER,
        false);
    attach(
        env,
        SourceSectionFilter.newBuilder().tagIs(StandardTags.WriteVariableTag.class).build(),
        EventKind.VAR_SET,
        false);
  }

  private static void attach(
      Env env, SourceSectionFilter filter, EventKind event, boolean reportExceptions) {
    env.getInstrumenter()
        .attachExecutionEventListener(
            filter,
            new ExecutionEventListener() {
              @Override
              public void onEnter(EventContext context, VirtualFrame frame) {
                publish(actualEvent(event, context), context, null);
              }

              @Override
              public void onReturnValue(
                  EventContext context, VirtualFrame frame, Object result) {
                if (event == EventKind.CALL_ENTER) {
                  publish(EventKind.CALL_RETURN, context, null);
                }
              }

              @Override
              public void onReturnExceptional(
                  EventContext context, VirtualFrame frame, Throwable exception) {
                if (reportExceptions) {
                  publish(EventKind.EXCEPTION_RAISE, context, exception);
                }
              }

              @Override
              public void onYield(EventContext context, VirtualFrame frame, Object value) {
                if (reportExceptions) {
                  publish(EventKind.PROMISE_SUSPEND, context, null);
                }
              }

              @Override
              public void onResume(EventContext context, VirtualFrame frame) {
                if (reportExceptions) {
                  publish(EventKind.PROMISE_RESUME, context, null);
                }
              }
            });
  }

  private static EventKind actualEvent(EventKind event, EventContext context) {
    if (event == EventKind.VAR_SET
        && context.getInstrumentedNode() instanceof HaraNodes.SetField) {
      return EventKind.FIELD_SET;
    }
    return event;
  }

  private static void publish(EventKind event, EventContext eventContext, Throwable exception) {
    try {
      HaraContext context = HaraLanguage.currentContext();
      SourceSection source = eventContext.getInstrumentedSourceSection();
      context.publishInterpreterEvent(
          event,
          source,
          exception == null
              ? Map.of()
              : Map.of("type", exception.getClass().getName()));
    } catch (IllegalStateException ignored) {
      // Instrumentation callbacks outside an entered Hara context are irrelevant.
    }
  }
}
