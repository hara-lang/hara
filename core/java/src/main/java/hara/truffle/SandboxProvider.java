package hara.truffle;

import java.util.List;

/** Trusted host SPI for a sandbox backend. */
interface SandboxProvider {
  String name();

  boolean secure();

  SandboxInstance open(SandboxModel.SandboxSpec spec);

  interface SandboxInstance extends AutoCloseable {
    Object eval(String source);

    Object call(String callable, List<String> argumentForms);

    boolean cancel();

    SandboxModel.SandboxState state();

    @Override
    void close();
  }
}
