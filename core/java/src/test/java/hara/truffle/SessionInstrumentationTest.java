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
              Capability.CONTROL_PAUSE,
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
}
