package hara.truffle;

import java.util.Objects;

/** Immutable data contracts for Kernel-managed sandboxes. */
final class SandboxModel {
  static final String SPEC_PROTOCOL = "hara.sandbox-spec/0-alpha";

  private SandboxModel() {}

  record SandboxId(long value) implements Comparable<SandboxId> {
    SandboxId {
      if (value <= 0) throw new IllegalArgumentException("INVALID_SANDBOX_ID");
    }

    @Override
    public int compareTo(SandboxId other) {
      return Long.compare(value, other.value);
    }

    @Override
    public String toString() {
      return Long.toString(value);
    }
  }

  enum SandboxState {
    OPEN,
    RUNNING,
    CANCELLED,
    FAILED,
    CLOSED
  }

  record SandboxLimits(
      int sourceBytes, int resultBytes, long evaluationMillis, int activeEvaluations) {
    SandboxLimits {
      if (sourceBytes <= 0
          || resultBytes <= 0
          || evaluationMillis <= 0
          || activeEvaluations != 1) {
        throw new SandboxException(ErrorCode.INVALID_SPEC, "invalid sandbox limits");
      }
    }

    static SandboxLimits defaults() {
      return new SandboxLimits(64 * 1024, 1024 * 1024, 5_000, 1);
    }
  }

  record SandboxSpec(
      String protocol,
      String provider,
      String runtime,
      String entryNamespace,
      SandboxLimits limits) {
    SandboxSpec {
      Objects.requireNonNull(protocol, "protocol");
      Objects.requireNonNull(provider, "provider");
      Objects.requireNonNull(runtime, "runtime");
      Objects.requireNonNull(entryNamespace, "entryNamespace");
      Objects.requireNonNull(limits, "limits");
      if (!SPEC_PROTOCOL.equals(protocol)) {
        throw new SandboxException(ErrorCode.INVALID_SPEC, "unsupported sandbox protocol");
      }
      if (provider.isEmpty() || runtime.isEmpty()) {
        throw new SandboxException(ErrorCode.INVALID_SPEC, "provider and runtime are required");
      }
      try {
        SessionModel.SessionId.parse(entryNamespace);
      } catch (IllegalArgumentException error) {
        throw new SandboxException(ErrorCode.INVALID_SPEC, "invalid entry namespace");
      }
    }

    static SandboxSpec inProcess() {
      return new SandboxSpec(
          SPEC_PROTOCOL,
          "in-process",
          "hara.standard/0-alpha",
          "user",
          SandboxLimits.defaults());
    }
  }

  record SandboxStatus(SandboxId id, String provider, SandboxState state) {}

  enum ErrorCode {
    INVALID_SPEC,
    PROVIDER_NOT_FOUND,
    NOT_FOUND,
    CLOSED,
    BUSY,
    LIMIT_EXCEEDED,
    EVALUATION_FAILED,
    UNSUPPORTED
  }

  static final class SandboxException extends RuntimeException {
    private final ErrorCode code;

    SandboxException(ErrorCode code, String message) {
      super(message);
      this.code = code;
    }

    ErrorCode code() {
      return code;
    }
  }
}
