package hara.truffle;

import hara.lang.data.Symbol;
import hara.lang.protocol.IWork;
import hara.lang.protocol.IWorkHost;
import hara.lang.protocol.IWorkRef;
import hara.lang.protocol.IWorkRun;
import java.util.List;
import java.util.Map;

/** Installs the native work lifecycle protocol identities. */
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

  private static HaraProtocol requireProtocol(HaraContext context, String name) {
    HaraVar variable = context.resolve(Symbol.create("std.foundation", name));
    if (variable == null || !(variable.get() instanceof HaraProtocol protocol)) {
      throw new HaraException("Native work protocol parent is unavailable: " + name);
    }
    return protocol;
  }
}
