package hara.truffle;

import java.util.List;

/** Trusted host SPI for a sandbox backend. */
interface SandboxProvider {
  String name();

  boolean secure();

  SandboxInstance open(SandboxModel.SandboxSpec spec);

  interface SandboxInstance extends AutoCloseable {
    Object eval(String source);

    Object call(String callable, List<Object> arguments);

    boolean cancel();

    SandboxModel.SandboxState state();

    SandboxModel.SandboxError error();

    @Override
    void close();
  }
}
