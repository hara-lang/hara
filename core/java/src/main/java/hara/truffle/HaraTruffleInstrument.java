package hara.truffle;

import com.oracle.truffle.api.frame.VirtualFrame;
import com.oracle.truffle.api.instrumentation.EventBinding;
import com.oracle.truffle.api.instrumentation.EventContext;
import com.oracle.truffle.api.instrumentation.ExecutionEventListener;
import com.oracle.truffle.api.instrumentation.Instrumenter;
import com.oracle.truffle.api.instrumentation.SourceSectionFilter;
import com.oracle.truffle.api.instrumentation.StandardTags;
import com.oracle.truffle.api.instrumentation.TruffleInstrument;
import com.oracle.truffle.api.nodes.Node;
import java.util.LinkedHashMap;
import java.util.Map;

/** Engine-scoped Truffle producer activated only while a trusted target is attached. */
@TruffleInstrument.Registration(
    id = HaraTruffleInstrument.ID,
    name = "Hara Runtime Instrumentation",
    version = "0.1",
    services = HaraTruffleInstrument.Service.class)
public final class HaraTruffleInstrument extends TruffleInstrument {
  public static final String ID = "hara-runtime-instrumentation";

  /** Host-only binding controller. No instance is installed into a Hara namespace. */
  public interface Service {
    void setEnabled(boolean enabled);

    boolean isEnabled();
  }

  private Controller controller;

  @Override
  protected void onCreate(Env environment) {
    controller = new Controller(environment.getInstrumenter());
    environment.registerService(controller);
  }

  @Override
  protected void onDispose(Env environment) {
    if (controller != null) controller.close();
  }

  private static final class Controller implements Service, AutoCloseable {
    private static final SourceSectionFilter FILTER =
        SourceSectionFilter.newBuilder()
            .tagIs(
                StandardTags.ExpressionTag.class,
                HaraInstrumentationTags.ExecutionRootTag.class)
            .build();

    private final Instrumenter instrumenter;
    private EventBinding<ExecutionEventListener> binding;

    Controller(Instrumenter instrumenter) {
      this.instrumenter = instrumenter;
    }

    @Override
    public synchronized void setEnabled(boolean enabled) {
      if (enabled) {
        if (binding == null) {
          binding =
              instrumenter.attachExecutionEventListener(FILTER, new Listener());
        }
      } else if (binding != null) {
        binding.dispose();
        binding = null;
      }
    }

    @Override
    public synchronized boolean isEnabled() {
      return binding != null;
    }

    @Override
    public synchronized void close() {
      setEnabled(false);
    }
  }

  private static final class Listener implements ExecutionEventListener {
    @Override
    public void onEnter(EventContext eventContext, VirtualFrame frame) {
      HaraContext context = context(eventContext);
      if (context == null) return;
      context.publishInstrumentation(
          InstrumentationModel.EventKind.SEMANTIC_BOUNDARY,
          eventContext.getInstrumentedSourceSection(),
          Map.of());
    }

    @Override
    public void onReturnValue(
        EventContext eventContext, VirtualFrame frame, Object result) {
      if (!eventContext.hasTag(HaraInstrumentationTags.ExecutionRootTag.class)) return;
      HaraContext context = context(eventContext);
      if (context == null) return;
      context.publishInstrumentation(
          InstrumentationModel.EventKind.EXECUTION_TERMINAL,
          eventContext.getInstrumentedSourceSection(),
          Map.of("status", "returned"));
    }

    @Override
    public void onReturnExceptional(
        EventContext eventContext, VirtualFrame frame, Throwable exception) {
      if (!eventContext.hasTag(HaraInstrumentationTags.ExecutionRootTag.class)) return;
      HaraContext context = context(eventContext);
      if (context == null) return;
      Map<String, String> data = failureData(exception);
      context.publishInstrumentation(
          InstrumentationModel.EventKind.EXCEPTION_RAISE,
          eventContext.getInstrumentedSourceSection(),
          data);
      LinkedHashMap<String, String> terminal = new LinkedHashMap<>(data);
      terminal.put("status", "failed");
      context.publishInstrumentation(
          InstrumentationModel.EventKind.EXECUTION_TERMINAL,
          eventContext.getInstrumentedSourceSection(),
          terminal);
    }

    private static HaraContext context(EventContext eventContext) {
      Node node = eventContext.getInstrumentedNode();
      if (node == null) return null;
      return HaraLanguage.currentContext(node);
    }

    private static Map<String, String> failureData(Throwable exception) {
      LinkedHashMap<String, String> data = new LinkedHashMap<>();
      data.put("error-class", exception.getClass().getName());
      String message = exception.getMessage();
      if (message != null && !message.isBlank()) data.put("error-message", message);
      return data;
    }
  }
}
