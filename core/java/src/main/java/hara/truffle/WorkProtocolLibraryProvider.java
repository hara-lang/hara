package hara.truffle;

import hara.lang.data.Symbol;
import hara.lang.protocol.IWork;
import hara.lang.protocol.IWorkHost;
import hara.lang.protocol.IWorkRef;
import hara.lang.protocol.IWorkRun;
import java.util.List;
import java.util.Map;

/** Installs the native work lifecycle protocol identities and scope helpers. */
public final class WorkProtocolLibraryProvider implements HaraLibraryProvider {
  @Override
  public String namespace() {
    return "work.native.protocol";
  }

  @Override
  public int order() {
    return 1000;
  }

  @Override
  public boolean eager() {
    return true;
  }

  @Override
  public void install(HaraContext context) {
    HaraProtocol component = requireProtocol(context, "IComponent");
    HaraProtocol closed = requireProtocol(context, "IClosed");

    installWork(context.defineProtocol("IWork", Map.of("work-spec", 1)));
    HaraProtocol workRef = context.defineProtocol("IWorkRef", Map.of("work-id", 1));
    installWorkRef(workRef);
    installWorkHost(
        context.defineProtocol(
            "IWorkHost",
            Map.of("work-submit", 4, "work-resolve", 2),
            List.of(component)));
    installWorkRun(
        context.defineProtocol(
            "IWorkRun",
            Map.of(
                "work-status", 1,
                "work-result", 1,
                "work-events", 2,
                "work-cancel", 2),
            List.of(workRef, closed)));

    context.defineLibraryValue(namespace(), "default-host", HaraWorkHost.instance(), null);
    context.defineLibraryFunction(
        namespace(),
        "current-context",
        arguments -> {
          requireArity("current-context", arguments, 0);
          return HaraWorkHost.currentWorkContext();
        },
        null);
    context.defineLibraryFunction(
        namespace(),
        "context-id",
        arguments -> workContext("context-id", arguments, 0).workId(),
        null);
    context.defineLibraryFunction(
        namespace(),
        "context-cancelled?",
        arguments -> workContext("context-cancelled?", arguments, 0).cancelled(),
        null);
    context.defineLibraryFunction(
        namespace(),
        "context-cancel-reason",
        arguments -> workContext("context-cancel-reason", arguments, 0).cancelReason(),
        null);
    context.defineLibraryFunction(
        namespace(),
        "context-deadline-nanos",
        arguments -> workContext("context-deadline-nanos", arguments, 0).deadlineNanos(),
        null);
    context.defineLibraryFunction(
        namespace(),
        "context-check!",
        arguments -> {
          HaraWorkHost.WorkContext workContext = workContext("context-check!", arguments, 0);
          workContext.checkCancelled();
          return workContext;
        },
        null);
    context.defineLibraryFunction(
        namespace(),
        "context-submit",
        arguments -> {
          if (arguments.length == 3) {
            return requireCurrent("context-submit")
                .submitChild(arguments[0], arguments[1], arguments[2]);
          }
          if (arguments.length == 4) {
            return asWorkContext("context-submit", arguments[0])
                .submitChild(arguments[1], arguments[2], arguments[3]);
          }
          throw new HaraException("context-submit expects 3 or 4 arguments");
        },
        null);
    context.defineLibraryFunction(
        namespace(),
        "context-on-close!",
        arguments -> {
          if (arguments.length == 1) {
            return requireCurrent("context-on-close!").onClose(arguments[0]);
          }
          if (arguments.length == 2) {
            return asWorkContext("context-on-close!", arguments[0]).onClose(arguments[1]);
          }
          throw new HaraException("context-on-close! expects 1 or 2 arguments");
        },
        null);
  }

  static void installWork(HaraProtocol protocol) {
    protocol.extend(
        IWork.class,
        "work-spec",
        (receiver, arguments) -> ((IWork) receiver).workSpec());
  }

  static void installWorkRef(HaraProtocol protocol) {
    protocol.extend(
        IWorkRef.class,
        "work-id",
        (receiver, arguments) -> ((IWorkRef) receiver).workId());
  }

  static void installWorkHost(HaraProtocol protocol) {
    protocol.extend(
        IWorkHost.class,
        "work-submit",
        (receiver, arguments) ->
            ((IWorkHost) receiver).workSubmit(arguments[0], arguments[1], arguments[2]));
    protocol.extend(
        IWorkHost.class,
        "work-resolve",
        (receiver, arguments) -> ((IWorkHost) receiver).workResolve(arguments[0]));
  }

  static void installWorkRun(HaraProtocol protocol) {
    protocol.extend(
        IWorkRun.class,
        "work-status",
        (receiver, arguments) -> ((IWorkRun) receiver).workStatus());
    protocol.extend(
        IWorkRun.class,
        "work-result",
        (receiver, arguments) -> ((IWorkRun) receiver).workResult());
    protocol.extend(
        IWorkRun.class,
        "work-events",
        (receiver, arguments) -> ((IWorkRun) receiver).workEvents(arguments[0]));
    protocol.extend(
        IWorkRun.class,
        "work-cancel",
        (receiver, arguments) -> ((IWorkRun) receiver).workCancel(arguments[0]));
  }

  private static HaraWorkHost.WorkContext workContext(
      String name, Object[] arguments, int implicitArity) {
    if (arguments.length == implicitArity) return requireCurrent(name);
    if (arguments.length == implicitArity + 1) return asWorkContext(name, arguments[0]);
    throw new HaraException(
        name + " expects " + implicitArity + " or " + (implicitArity + 1) + " arguments");
  }

  private static HaraWorkHost.WorkContext requireCurrent(String name) {
    HaraWorkHost.WorkContext workContext = HaraWorkHost.currentWorkContext();
    if (workContext == null) {
      throw new HaraException(name + " requires an active native work context");
    }
    return workContext;
  }

  private static HaraWorkHost.WorkContext asWorkContext(String name, Object value) {
    if (value instanceof HaraWorkHost.WorkContext workContext) return workContext;
    throw new HaraException(name + " requires a native work context");
  }

  private static void requireArity(String name, Object[] arguments, int arity) {
    if (arguments.length != arity) {
      throw new HaraException(name + " expects " + arity + " arguments");
    }
  }

  private static HaraProtocol requireProtocol(HaraContext context, String name) {
    HaraVar variable = context.resolve(Symbol.create("std.foundation", name));
    if (variable == null || !(variable.get() instanceof HaraProtocol protocol)) {
      throw new HaraException("Native work protocol parent is unavailable: " + name);
    }
    return protocol;
  }
}
