package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import hara.truffle.InstrumentationException.Code;
import hara.truffle.InstrumentationModel.Capability;
import hara.truffle.InstrumentationModel.EventDelivery;
import hara.truffle.InstrumentationModel.EventEnvelope;
import hara.truffle.InstrumentationModel.EventKind;
import hara.truffle.InstrumentationModel.EventPhase;
import hara.truffle.InstrumentationModel.InstrumentFilter;
import hara.truffle.InstrumentationModel.InstrumentMode;
import hara.truffle.InstrumentationModel.InstrumentRegistration;
import hara.truffle.InstrumentationModel.ProjectionRequest;
import hara.truffle.InstrumentationModel.RuntimeBackend;
import hara.truffle.InstrumentationModel.TargetDescriptor;
import hara.truffle.InstrumentationModel.TargetHandle;
import hara.truffle.InstrumentationModel.TargetKind;
import hara.truffle.NativeInstrumentation.NativeInstrumentHandle;
import hara.truffle.NativeInstrumentation.NativeTargetHandle;
import hara.truffle.NativeInstrumentation.NativeControlLease;
import hara.truffle.bytecode.HbcProgram;
import hara.truffle.bytecode.HbcProgram.Function;
import hara.truffle.bytecode.HbcProgram.Instruction;
import hara.truffle.bytecode.HbcProgram.Opcode;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.Set;
import org.junit.Test;

public class SessionInstrumentationTest {
  @Test
  public void sessionOwnsScopedHostServiceAndDirectStopCleansEverything() {
    SessionModel.SessionId sessionId = SessionModel.SessionId.parse("instrumented");
    try (SessionKernel kernel = new SessionKernel(false, false)) {
      SessionKernel.Session session = kernel.create(sessionId);
      InstrumentationHub hub = kernel.instrumentationHub();
      TargetHandle target =
          hub.registerTarget(interpreterTarget("execution", sessionId.value()));
      NativeInstrumentation service = kernel.instrumentation(sessionId);
      NativeTargetHandle nativeTarget =
          service.bindTargetIdentity(target.targetId(), target.generation());
      NativeInstrumentHandle trace =
          service.register(passive("trace", sessionId.value()));
      service.attach(trace, nativeTarget);

      assertEquals(
          1,
          hub.publish(
              target,
              EventKind.EXECUTION_TERMINAL,
              EventPhase.LIVE,
              null,
              Map.of("status", "returned")));
      assertEquals(1, service.drainEvents(trace).events().size());
      session.stop();
      assertEquals(0, hub.instrumentCount());
      assertEquals(0, hub.targetCount());
      assertEquals(0, hub.attachmentCount());
      InstrumentationException closed =
          assertThrows(
              InstrumentationException.class,
              () -> service.drainEvents(trace));
      assertEquals(Code.SESSION_CLOSED, closed.code());
    }
  }

  @Test
  public void sessionIdReuseDoesNotReviveOldServiceOrHandles() {
    SessionModel.SessionId sessionId = SessionModel.SessionId.parse("reused");
    try (SessionKernel kernel = new SessionKernel(false, false)) {
      kernel.create(sessionId);
      InstrumentationHub hub = kernel.instrumentationHub();
      TargetHandle firstTarget =
          hub.registerTarget(interpreterTarget("execution", sessionId.value()));
      NativeInstrumentation oldService = kernel.instrumentation(sessionId);
      NativeInstrumentHandle oldTrace =
          oldService.register(passive("trace", sessionId.value()));
      oldService.attach(
          oldTrace,
          oldService.bindTargetIdentity("execution", firstTarget.generation()));
      kernel.closeSession(sessionId);

      kernel.create(sessionId);
      TargetHandle secondTarget =
          hub.registerTarget(interpreterTarget("execution", sessionId.value()));
      NativeInstrumentation newService = kernel.instrumentation(sessionId);
      NativeTargetHandle newTarget =
          newService.bindTargetIdentity("execution", secondTarget.generation());
      assertEquals(1L, secondTarget.generation());
      InstrumentationException closed =
          assertThrows(
              InstrumentationException.class,
              () ->
                  oldService.bindTargetIdentity(
                      "execution", secondTarget.generation()));
      assertEquals(Code.SESSION_CLOSED, closed.code());
      InstrumentationException stale =
          assertThrows(
              InstrumentationException.class,
              () -> newService.attach(oldTrace, newTarget));
      assertEquals(Code.STALE_INSTRUMENT, stale.code());
    }
  }

  @Test
  public void crossSessionAndCrossRuntimeHandlesFailClosed() {
    SessionModel.SessionId alpha = SessionModel.SessionId.parse("alpha");
    SessionModel.SessionId beta = SessionModel.SessionId.parse("beta");
    try (SessionKernel first = new SessionKernel(false, false);
        SessionKernel second = new SessionKernel(false, false)) {
      first.create(alpha);
      first.create(beta);
      second.create(alpha);
      TargetHandle alphaTarget =
          first
              .instrumentationHub()
              .registerTarget(interpreterTarget("alpha-target", alpha.value()));
      TargetHandle betaTarget =
          first
              .instrumentationHub()
              .registerTarget(interpreterTarget("beta-target", beta.value()));
      TargetHandle foreignTarget =
          second
              .instrumentationHub()
              .registerTarget(interpreterTarget("foreign-target", alpha.value()));
      NativeInstrumentation alphaService = first.instrumentation(alpha);
      NativeInstrumentation betaService = first.instrumentation(beta);
      NativeInstrumentation foreignService = second.instrumentation(alpha);
      NativeInstrumentHandle alphaTrace =
          alphaService.register(passive("trace", alpha.value()));
      NativeTargetHandle betaNative =
          betaService.bindTargetIdentity(
              betaTarget.targetId(), betaTarget.generation());
      NativeTargetHandle foreignNative =
          foreignService.bindTargetIdentity(
              foreignTarget.targetId(), foreignTarget.generation());

      InstrumentationException crossSession =
          assertThrows(
              InstrumentationException.class,
              () -> betaService.attach(alphaTrace, betaNative));
      assertEquals(Code.CROSS_SESSION, crossSession.code());
      InstrumentationException crossRuntime =
          assertThrows(
              InstrumentationException.class,
              () -> foreignService.attach(alphaTrace, foreignNative));
      assertEquals(Code.CROSS_RUNTIME, crossRuntime.code());
      assertEquals(
          alphaTarget, first.instrumentationHub().bindTarget("alpha-target", 0));
    }
  }

  @Test
  public void kernelShutdownInvalidatesRootServiceAndHub() {
    SessionKernel kernel = new SessionKernel(false, false);
    NativeInstrumentation service = kernel.instrumentation(kernel.root().id());
    NativeInstrumentHandle trace = service.register(passive("root-trace", "ROOT"));
    InstrumentationHub hub = kernel.instrumentationHub();
    kernel.close();
    assertTrue(hub.isClosed());
    InstrumentationException closed =
        assertThrows(
            InstrumentationException.class,
            () -> service.drainEvents(trace));
    assertEquals(Code.RUNTIME_CLOSED, closed.code());
  }

  @Test
  public void truffleProducerIsLazyAndEmitsPassiveTopLevelEvents() {
    SessionModel.SessionId sessionId = SessionModel.SessionId.parse("truffle");
    try (SessionKernel kernel = new SessionKernel(false, false)) {
      SessionKernel.Session session = kernel.create(sessionId);
      NativeInstrumentation service = kernel.instrumentation(sessionId);
      NativeTargetHandle target =
          service.bindTargetIdentity(sessionId.value() + "/interpreter", 0);
      TargetDescriptor descriptor = service.targetDescriptor(target);
      assertEquals(new RuntimeBackend("java-truffle"), descriptor.backend());
      assertEquals(
          Set.of(
              Capability.EVENT_SEMANTIC_BOUNDARY,
              Capability.EVENT_EXCEPTION,
              Capability.EVENT_LIFECYCLE,
              Capability.INSPECT_SOURCE_LOCATION),
          descriptor.capabilities());
      NativeInstrumentHandle trace =
          service.register(
              passive(
                  "trace",
                  sessionId.value(),
                  Set.of(EventKind.SEMANTIC_BOUNDARY, EventKind.EXECUTION_TERMINAL),
                  Set.of(Capability.EVENT_SEMANTIC_BOUNDARY, Capability.EVENT_LIFECYCLE),
                  ProjectionRequest.none()));

      session.eval("42");
      assertFalse(session.truffleInstrumentationActive());
      assertTrue(service.drainEvents(trace).events().isEmpty());

      NativeInstrumentation.NativeAttachment attachment = service.attach(trace, target);
      assertTrue(session.truffleInstrumentationActive());
      session.eval("42");
      var events = service.drainEvents(trace).events();
      assertTrue(events.stream().anyMatch(event -> event.event() == EventKind.SEMANTIC_BOUNDARY));
      assertEquals(
          1,
          events.stream()
              .filter(event -> event.event() == EventKind.EXECUTION_TERMINAL)
              .count());

      service.detach(attachment);
      assertFalse(session.truffleInstrumentationActive());
    }
  }

  @Test
  public void hbcTargetUsesExplicitBackendProvenanceAndAdvertisesImplementedControls() {
    SessionModel.SessionId sessionId = SessionModel.SessionId.parse("hbc");
    try (SessionKernel kernel = new SessionKernel(false, false)) {
      kernel.create(sessionId);
      NativeInstrumentation service = kernel.instrumentation(sessionId);
      NativeTargetHandle target =
          service.bindTargetIdentity(sessionId.value() + "/hbc", 0);
      TargetDescriptor descriptor = service.targetDescriptor(target);

      assertEquals(new RuntimeBackend("java-hbc"), descriptor.backend());
      assertEquals(
          Set.of(
              Capability.EVENT_INSTRUCTION,
              Capability.EVENT_CALL,
              Capability.EVENT_EXCEPTION,
              Capability.EVENT_SUSPENSION,
              Capability.EVENT_LIFECYCLE,
              Capability.INSPECT_SOURCE_LOCATION,
              Capability.INSPECT_SOURCE_LOCATION,
              Capability.CONTROL_PAUSE,
              Capability.CONTROL_SINGLE_STEP,
              Capability.CONTROL_RESUME,
              Capability.CONTROL_SETTLE,
              Capability.CONTROL_TERMINATE),
          descriptor.capabilities());
    }
  }

  @Test
  public void topLevelFailureIsReportedOnceAndNestedRootsAreNotTerminal() {
    SessionModel.SessionId sessionId = SessionModel.SessionId.parse("failure");
    try (SessionKernel kernel = new SessionKernel(false, false)) {
      SessionKernel.Session session = kernel.create(sessionId);
      NativeInstrumentation service = kernel.instrumentation(sessionId);
      NativeTargetHandle target =
          service.bindTargetIdentity(sessionId.value() + "/interpreter", 0);
      NativeInstrumentHandle trace =
          service.register(
              passive(
                  "trace",
                  sessionId.value(),
                  Set.of(EventKind.EXCEPTION_RAISE, EventKind.EXECUTION_TERMINAL),
                  Set.of(Capability.EVENT_EXCEPTION, Capability.EVENT_LIFECYCLE),
                  ProjectionRequest.none()));
      service.attach(trace, target);

      session.eval("(do (defn inner [] 1) (inner))");
      var success = service.drainEvents(trace).events();
      assertEquals(
          1,
          success.stream()
              .filter(event -> event.event() == EventKind.EXECUTION_TERMINAL)
              .count());

      assertThrows(IllegalArgumentException.class, () -> session.eval("(throw \"boom\")"));
      var failure = service.drainEvents(trace).events();
      assertEquals(
          1,
          failure.stream().filter(event -> event.event() == EventKind.EXCEPTION_RAISE).count());
      assertEquals(
          1,
          failure.stream()
              .filter(event -> event.event() == EventKind.EXECUTION_TERMINAL)
              .count());
      EventEnvelope terminal =
          failure.stream()
              .filter(event -> event.event() == EventKind.EXECUTION_TERMINAL)
              .findFirst()
              .orElseThrow();
      assertEquals("failure", terminal.data().get("status"));
    }
  }

  @Test
  public void hbcProductionLoopRetainsStateAcrossSuspendStepResumeSettleAndTerminate() {
    SessionModel.SessionId sessionId = SessionModel.SessionId.parse("hbc-control");
    HbcProgram program = arithmeticHbcProgram();
    try (SessionKernel kernel = new SessionKernel(false, false)) {
      SessionKernel.Session session = kernel.create(sessionId);
      NativeInstrumentation service = kernel.instrumentation(sessionId);
      NativeTargetHandle target =
          service.bindTargetIdentity(sessionId.value() + "/hbc", 0);
      NativeInstrumentHandle controller =
          service.register(hbcController("controller", sessionId.value()));
      service.attach(controller, target);
      NativeControlLease lease = service.acquireControlLease(controller, target);

      service.issueDirective(lease, InstrumentationModel.InstrumentDirective.SUSPEND);
      assertThrows(IllegalArgumentException.class, () -> session.evalHbc(program));
      assertEquals(
          List.of(EventKind.MACHINE_SUSPEND),
          service.drainEvents(controller).events().stream().map(EventEnvelope::event).toList());

      service.issueDirective(lease, InstrumentationModel.InstrumentDirective.STEP_NEXT);
      assertThrows(IllegalArgumentException.class, () -> session.evalHbc(program));
      List<EventEnvelope> stepped = service.drainEvents(controller).events();
      assertEquals(
          List.of(
              EventKind.MACHINE_RESUME,
              EventKind.INSTRUCTION_EXECUTE,
              EventKind.MACHINE_SUSPEND),
          stepped.stream().map(EventEnvelope::event).toList());
      assertEquals(Integer.valueOf(0), stepped.get(1).location().instructionPointer());

      service.issueDirective(lease, InstrumentationModel.InstrumentDirective.SETTLE);
      assertEquals(42L, session.evalHbc(program).asLong());
      List<EventEnvelope> settled = service.drainEvents(controller).events();
      assertEquals(
          1,
          settled.stream()
              .filter(event -> event.event() == EventKind.MACHINE_RESUME)
              .count());
      assertEquals(
          1,
          settled.stream()
              .filter(event -> event.event() == EventKind.EXECUTION_TERMINAL)
              .count());
      assertEquals(
          "return",
          settled.stream()
              .filter(event -> event.event() == EventKind.EXECUTION_TERMINAL)
              .findFirst()
              .orElseThrow()
              .data()
              .get("status"));

      service.issueDirective(lease, InstrumentationModel.InstrumentDirective.SUSPEND);
      assertThrows(IllegalArgumentException.class, () -> session.evalHbc(program));
      service.drainEvents(controller);
      service.issueDirective(lease, InstrumentationModel.InstrumentDirective.TERMINATE);
      assertThrows(IllegalArgumentException.class, () -> session.evalHbc(program));
      List<EventEnvelope> terminated = service.drainEvents(controller).events();
      assertEquals(
          1,
          terminated.stream()
              .filter(event -> event.event() == EventKind.EXECUTION_TERMINAL)
              .count());
      assertEquals(
          "terminated",
          terminated.stream()
              .filter(event -> event.event() == EventKind.EXECUTION_TERMINAL)
              .findFirst()
              .orElseThrow()
              .data()
              .get("status"));
    }
  }

  @Test
  public void hbcProductionLoopEmitsCallsUnwindAndOneTerminalOutcome() {
    SessionModel.SessionId sessionId = SessionModel.SessionId.parse("hbc-boundaries");
    try (SessionKernel kernel = new SessionKernel(false, false)) {
      SessionKernel.Session session = kernel.create(sessionId);
      NativeInstrumentation service = kernel.instrumentation(sessionId);
      NativeTargetHandle target =
          service.bindTargetIdentity(sessionId.value() + "/hbc", 0);
      NativeInstrumentHandle trace =
          service.register(hbcController("trace", sessionId.value()));
      service.attach(trace, target);

      assertEquals(7L, session.evalHbc(hbcCallProgram()).asLong());
      List<EventEnvelope> calls = service.drainEvents(trace).events();
      assertEquals(
          1, calls.stream().filter(event -> event.event() == EventKind.CALL_ENTER).count());
      assertEquals(
          1, calls.stream().filter(event -> event.event() == EventKind.CALL_RETURN).count());
      assertEquals(
          1, calls.stream().filter(event -> event.event() == EventKind.EXECUTION_TERMINAL).count());

      assertThrows(IllegalArgumentException.class, () -> session.evalHbc(hbcThrowProgram()));
      List<EventEnvelope> failure = service.drainEvents(trace).events();
      assertEquals(
          1,
          failure.stream()
              .filter(event -> event.event() == EventKind.EXCEPTION_UNWIND)
              .count());
      assertEquals(
          1,
          failure.stream()
              .filter(event -> event.event() == EventKind.EXECUTION_TERMINAL)
              .count());
      assertEquals(
          "failure",
          failure.stream()
              .filter(event -> event.event() == EventKind.EXECUTION_TERMINAL)
              .findFirst()
              .orElseThrow()
              .data()
              .get("status"));
    }
  }

  @Test
  public void sourceLocationIsProjectedOnlyToRequestingInstruments() {
    SessionModel.SessionId sessionId = SessionModel.SessionId.parse("locations");
    try (SessionKernel kernel = new SessionKernel(false, false)) {
      SessionKernel.Session session = kernel.create(sessionId);
      NativeInstrumentation service = kernel.instrumentation(sessionId);
      NativeTargetHandle target =
          service.bindTargetIdentity(sessionId.value() + "/interpreter", 0);
      NativeInstrumentHandle withoutLocation =
          service.register(
              passive(
                  "without-location",
                  sessionId.value(),
                  Set.of(EventKind.SEMANTIC_BOUNDARY),
                  Set.of(Capability.EVENT_SEMANTIC_BOUNDARY),
                  ProjectionRequest.none()));
      NativeInstrumentHandle withLocation =
          service.register(
              passive(
                  "with-location",
                  sessionId.value(),
                  Set.of(EventKind.SEMANTIC_BOUNDARY),
                  Set.of(Capability.EVENT_SEMANTIC_BOUNDARY, Capability.INSPECT_SOURCE_LOCATION),
                  new ProjectionRequest(true, null, null, null, null, null, null)));
      service.attach(withoutLocation, target);
      service.attach(withLocation, target);

      session.eval("42", "location.hal", 1, 1);
      var without = service.drainEvents(withoutLocation).events();
      var with = service.drainEvents(withLocation).events();
      assertFalse(without.isEmpty());
      assertTrue(with.stream().allMatch(event -> event.location() != null));
      assertTrue(without.stream().allMatch(event -> event.location() == null));
    }
  }

  private static TargetDescriptor interpreterTarget(String id, String session) {
    return new TargetDescriptor(
        id,
        session,
        TargetKind.INTERPRETER,
        new RuntimeBackend("java"),
        Set.of(Capability.EVENT_LIFECYCLE));
  }

  private static InstrumentRegistration passive(String id, String session) {
    return passive(
        id,
        session,
        Set.of(EventKind.EXECUTION_TERMINAL),
        Set.of(Capability.EVENT_LIFECYCLE),
        ProjectionRequest.none());
  }

  private static InstrumentRegistration passive(
      String id,
      String session,
      Set<EventKind> events,
      Set<Capability> capabilities,
      ProjectionRequest projection) {
    return new InstrumentRegistration(
        id,
        session,
        InstrumentMode.PASSIVE,
        capabilities,
        events,
        new InstrumentFilter(session, Set.of(), Set.of(), Set.of()),
        projection,
        EventDelivery.queue(8));
  }

  private static InstrumentRegistration hbcController(String id, String session) {
    return new InstrumentRegistration(
        id,
        session,
        InstrumentMode.CONTROL,
        Set.of(
            Capability.EVENT_INSTRUCTION,
            Capability.EVENT_CALL,
            Capability.EVENT_EXCEPTION,
            Capability.EVENT_SUSPENSION,
            Capability.EVENT_LIFECYCLE,
            Capability.CONTROL_PAUSE,
            Capability.CONTROL_SINGLE_STEP,
            Capability.CONTROL_RESUME,
            Capability.CONTROL_SETTLE,
            Capability.CONTROL_TERMINATE),
        Set.of(
            EventKind.INSTRUCTION_EXECUTE,
            EventKind.CALL_ENTER,
            EventKind.CALL_RETURN,
            EventKind.EXCEPTION_UNWIND,
            EventKind.MACHINE_SUSPEND,
            EventKind.MACHINE_RESUME,
            EventKind.EXECUTION_TERMINAL),
        new InstrumentFilter(session, Set.of(), Set.of(TargetKind.HBC), Set.of(new RuntimeBackend("java-hbc"))),
        new ProjectionRequest(true, null, null, null, null, null, null),
        EventDelivery.queue(256));
  }

  private static HbcProgram arithmeticHbcProgram() {
    Function entry =
        new Function(
            "entry",
            false,
            0,
            false,
            0,
            0,
            2,
            List.of(
                new Instruction(Opcode.CONSTANT, 0, 0, 0),
                new Instruction(Opcode.CONSTANT, 1, 0, 0),
                new Instruction(Opcode.PRIMITIVE, HbcProgram.Primitive.ADD.id(), 2, 0),
                Instruction.of(Opcode.RETURN)),
            Arrays.asList(null, null, null, null),
            List.of());
    return new HbcProgram(List.of(41L, 1L), List.of(), List.of(entry), 0);
  }

  private static HbcProgram hbcCallProgram() {
    Function entry =
        new Function(
            "entry",
            false,
            0,
            false,
            0,
            0,
            1,
            List.of(
                new Instruction(Opcode.CALL_STATIC, 1, 0, 0),
                Instruction.of(Opcode.RETURN)),
            Arrays.asList(null, null),
            List.of());
    Function callee =
        new Function(
            "callee",
            false,
            0,
            false,
            0,
            0,
            1,
            List.of(new Instruction(Opcode.CONSTANT, 0, 0, 0), Instruction.of(Opcode.RETURN)),
            Arrays.asList(null, null),
            List.of());
    return new HbcProgram(List.of(7L), List.of(), List.of(entry, callee), 0);
  }

  private static HbcProgram hbcThrowProgram() {
    Function entry =
        new Function(
            "thrower",
            false,
            0,
            false,
            0,
            0,
            1,
            List.of(new Instruction(Opcode.CONSTANT, 0, 0, 0), Instruction.of(Opcode.THROW)),
            Arrays.asList(null, null),
            List.of());
    return new HbcProgram(List.of("boom"), List.of(), List.of(entry), 0);
  }
}
