package hara.truffle;

import java.nio.charset.StandardCharsets;
import java.util.List;

/** Conformance-only provider. In-process isolation is explicitly non-secure. */
final class InProcessSandboxProvider implements SandboxProvider {
  static final InProcessSandboxProvider INSTANCE = new InProcessSandboxProvider();

  private InProcessSandboxProvider() {}

  @Override
  public String name() {
    return "in-process";
  }

  @Override
  public boolean secure() {
    return false;
  }

  @Override
  public SandboxInstance open(SandboxModel.SandboxSpec spec) {
    return new Instance(spec, SessionKernel.Session.privateSandbox(spec.entryNamespace()));
  }

  private static final class Instance implements SandboxInstance {
    private final SandboxModel.SandboxSpec spec;
    private final SessionKernel.Session session;
    private SandboxModel.SandboxState state = SandboxModel.SandboxState.OPEN;

    private Instance(SandboxModel.SandboxSpec spec, SessionKernel.Session session) {
      this.spec = spec;
      this.session = session;
    }

    private void requireOpen() {
      if (state == SandboxModel.SandboxState.CLOSED) {
        throw new SandboxModel.SandboxException(
            SandboxModel.ErrorCode.CLOSED, "sandbox is closed");
      }
      if (state == SandboxModel.SandboxState.RUNNING) {
        throw new SandboxModel.SandboxException(SandboxModel.ErrorCode.BUSY, "sandbox is busy");
      }
    }

    @Override
    public Object eval(String source) {
      requireOpen();
      if (source.getBytes(StandardCharsets.UTF_8).length > spec.limits().sourceBytes()) {
        throw new SandboxModel.SandboxException(
            SandboxModel.ErrorCode.LIMIT_EXCEEDED, "sandbox source limit exceeded");
      }
      state = SandboxModel.SandboxState.RUNNING;
      try {
        Object result = session.evalTransfer(source);
        if (String.valueOf(result).getBytes(StandardCharsets.UTF_8).length
            > spec.limits().resultBytes()) {
          state = SandboxModel.SandboxState.FAILED;
          throw new SandboxModel.SandboxException(
              SandboxModel.ErrorCode.LIMIT_EXCEEDED, "sandbox result limit exceeded");
        }
        state = SandboxModel.SandboxState.OPEN;
        return result;
      } catch (SandboxModel.SandboxException error) {
        throw error;
      } catch (RuntimeException error) {
        state = SandboxModel.SandboxState.FAILED;
        throw new SandboxModel.SandboxException(
            SandboxModel.ErrorCode.EVALUATION_FAILED, error.getMessage());
      }
    }

    @Override
    public Object call(String callable, List<String> argumentForms) {
      if (callable == null || !callable.matches("[A-Za-z0-9._/?!*+-]+")) {
        throw new SandboxModel.SandboxException(
            SandboxModel.ErrorCode.INVALID_SPEC, "invalid sandbox callable");
      }
      return eval("(" + callable + argumentForms.stream().map(value -> " " + value).reduce("", String::concat) + ")");
    }

    @Override
    public boolean cancel() {
      requireOpen();
      state = SandboxModel.SandboxState.CANCELLED;
      return false;
    }

    @Override
    public SandboxModel.SandboxState state() {
      return state;
    }

    @Override
    public void close() {
      if (state != SandboxModel.SandboxState.CLOSED) {
        session.close();
        state = SandboxModel.SandboxState.CLOSED;
      }
    }
  }
}
