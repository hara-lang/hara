import {
  LiveSessionError,
  requireBackendSession,
} from "./live-session-model.js";

export function createInterpreterLiveBackend(runtime) {
  if (!runtime || typeof runtime.startNamed !== "function") {
    throw new TypeError("interpreter live backend requires InterpreterObservationRuntime");
  }
  return Object.freeze({
    id: "interpreter",
    operations: Object.freeze([
      "snapshot",
      "step",
      "run",
      "resume",
      "resolve",
      "reject",
      "update",
      "reset",
      "cancel",
      "dispose",
    ]),
    replacementPolicies: Object.freeze([
      "restart",
      "replace-on-next-start",
    ]),
    sourceKinds: Object.freeze(["source"]),
    start({ sessionId, source }) {
      if (source.kind !== "source") {
        throw new LiveSessionError(
          "live-session/source-kind",
          "interpreter backend starts from source only",
          { backend: "interpreter", kind: source.kind },
        );
      }
      return runtime.startNamed(sessionId, source.sourceId, source.value);
    },
  });
}

export function createBytecodeLiveBackend(runtime) {
  if (!runtime || typeof runtime.compileNamed !== "function" ||
      typeof runtime.fromNamedArtifact !== "function") {
    throw new TypeError("HBC live backend requires BytecodeObservationRuntime");
  }
  return Object.freeze({
    id: "hbc",
    operations: Object.freeze([
      "snapshot",
      "step",
      "run",
      "pause",
      "resume",
      "resolve",
      "reject",
      "update",
      "reset",
      "cancel",
      "dispose",
    ]),
    replacementPolicies: Object.freeze([
      "restart",
      "replace-on-next-start",
    ]),
    sourceKinds: Object.freeze(["source", "artifact"]),
    start({ sessionId, source }) {
      const session = source.kind === "artifact"
        ? runtime.fromNamedArtifact(sessionId, source.sourceId, source.value)
        : runtime.compileNamed(sessionId, source.sourceId, source.value);
      return new BytecodeLiveSessionAdapter(session);
    },
  });
}

class BytecodeLiveSessionAdapter {
  constructor(session) {
    this.session = requireBackendSession(session, "hbc");
  }

  get status() { return this.session.status; }
  get sequence() { return this.session.sequence; }

  snapshot() { return this.session.snapshot(); }
  step() { return this.session.step(); }
  run(limit) { return this.session.run(limit); }
  pause() { return this.session.pause(); }
  resume(settlement) { return this.session.resume(settlement); }
  resolveSuspension(value) { return this.session.resolveSuspension(value); }
  rejectSuspension(error) { return this.session.rejectSuspension(error); }
  reset() { return this.session.reset(); }

  cancel() {
    return Object.freeze({ cancelled: this.session.dispose() });
  }

  dispose() { return this.session.dispose(); }
}
