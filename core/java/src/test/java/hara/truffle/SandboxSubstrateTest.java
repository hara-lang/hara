package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertThrows;

import java.util.List;
import org.junit.Test;

public class SandboxSubstrateTest {
  @Test
  public void inProcessLifecycleIsPrivateAndExplicitlyNonSecure() {
    try (SessionKernel kernel = new SessionKernel(false, false)) {
      SandboxProvider provider = InProcessSandboxProvider.INSTANCE;
      assertFalse(provider.secure());
      kernel.registerSandboxProvider(provider);
      int sessionsBefore = kernel.size();

      SandboxModel.SandboxId sandbox = kernel.openSandbox(SandboxModel.SandboxSpec.inProcess());
      assertEquals(sessionsBefore, kernel.size());
      assertEquals(41L, kernel.sandboxEval(sandbox, "(def answer 41) answer"));
      assertEquals(42L, kernel.sandboxCall(sandbox, "std.foundation/+", List.of(41L, 1L)));
      String inertSource = "(do (def injected 99) :executed)";
      assertEquals(
          inertSource,
          kernel.sandboxCall(sandbox, "std.foundation/identity", List.of(inertSource)));
      assertEquals(SandboxModel.SandboxState.OPEN, kernel.sandboxStatus(sandbox).state());
      assertThrows(
          SandboxModel.SandboxException.class, () -> kernel.sandboxEval(sandbox, "injected"));
      assertFalse(kernel.cancelSandbox(sandbox));
      assertEquals(SandboxModel.SandboxState.CANCELLED, kernel.sandboxStatus(sandbox).state());

      kernel.closeSandbox(sandbox);
      SandboxModel.SandboxException error =
          assertThrows(SandboxModel.SandboxException.class, () -> kernel.sandboxStatus(sandbox));
      assertEquals(SandboxModel.ErrorCode.NOT_FOUND, error.code());
    }
  }

  @Test
  public void specValidationAndRuntimeIsolationAreEnforced() {
    SandboxModel.SandboxException invalid =
        assertThrows(
            SandboxModel.SandboxException.class,
            () -> new SandboxModel.SandboxLimits(1, 1, 1, 2));
    assertEquals(SandboxModel.ErrorCode.INVALID_SPEC, invalid.code());

    try (SessionKernel kernel = new SessionKernel(false, false)) {
      kernel.registerSandboxProvider(InProcessSandboxProvider.INSTANCE);
      kernel.root().eval("(def parent-secret 42)");
      SandboxModel.SandboxId sandbox = kernel.openSandbox(SandboxModel.SandboxSpec.inProcess());

      SandboxModel.SandboxException error =
          assertThrows(
              SandboxModel.SandboxException.class,
              () -> kernel.sandboxEval(sandbox, "parent-secret"));
      assertEquals(SandboxModel.ErrorCode.EVALUATION_FAILED, error.code());
    }
  }
}
