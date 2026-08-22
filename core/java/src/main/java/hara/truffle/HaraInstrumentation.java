package hara.truffle;

import com.oracle.truffle.api.frame.VirtualFrame;
import com.oracle.truffle.api.instrumentation.EventContext;
import com.oracle.truffle.api.instrumentation.EventBinding;
import com.oracle.truffle.api.instrumentation.ExecutionEventListener;
import com.oracle.truffle.api.instrumentation.SourceSectionFilter;
import com.oracle.truffle.api.instrumentation.StandardTags;
import com.oracle.truffle.api.instrumentation.TruffleInstrument;
import com.oracle.truffle.api.source.SourceSection;
import hara.truffle.InstrumentationModel.EventKind;
import java.util.Map;

@TruffleInstrument.Registration(
    id = "hara-execution",
    name = "Hara execution instrumentation",
    version = "0.1")
public final class HaraInstrumentation extends TruffleInstrument {
  private Env environment;
  private EventBinding<ExecutionEventListener> binding;

  @Override
  protected void onCreate(Env env) {
    environment = env;
    env.registerService(this);
  }

  synchronized void activate() {
    if (binding != null && binding.isAttached()) return;
    binding =
        environment
            .getInstrumenter()
            .attachExecutionEventListener(
                SourceSectionFilter.newBuilder()
                    .tagIs(StandardTags.ExpressionTag.class)
                    .build(),
                new ExecutionEventListener() {
                  @Override
                  public void onEnter(EventContext context, VirtualFrame frame) {
                    publish(EventKind.SEMANTIC_BOUNDARY, context, null);
                  }

                  @Override
                  public void onReturnValue(
                      EventContext context, VirtualFrame frame, Object result) {}

                  @Override
                  public void onReturnExceptional(
                      EventContext context, VirtualFrame frame, Throwable exception) {}
                });
  }

  synchronized void deactivate() {
    if (binding == null) return;
    binding.dispose();
    binding = null;
  }

  @Override
  protected synchronized void onDispose(Env env) {
    deactivate();
    environment = null;
  }

  private static void publish(EventKind event, EventContext eventContext, Throwable exception) {
    try {
      HaraContext context =
          HaraLanguage.currentContext(eventContext.getInstrumentedNode());
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
