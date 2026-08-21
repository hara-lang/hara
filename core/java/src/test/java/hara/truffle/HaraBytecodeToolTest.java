package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import org.junit.Test;

public class HaraBytecodeToolTest {
  @Test
  public void executesTheRustProducedAlphaConformanceCorpus() throws Exception {
    ByteArrayOutputStream outputBytes = new ByteArrayOutputStream();
    ByteArrayOutputStream errorBytes = new ByteArrayOutputStream();
    int status;
    try (PrintStream output = new PrintStream(outputBytes, true, StandardCharsets.UTF_8);
        PrintStream error = new PrintStream(errorBytes, true, StandardCharsets.UTF_8)) {
      status =
          HaraBytecodeTool.run(
              new String[] {
                "conformance",
                Path.of(System.getProperty("basedir"), "../rust/assets/bytecode-conformance.hcc")
                    .normalize()
                    .toString()
              },
              output,
              error);
    }
    assertEquals(errorBytes.toString(StandardCharsets.UTF_8), 0, status);
    assertTrue(
        outputBytes
            .toString(StandardCharsets.UTF_8)
            .matches("HBC0 conformance passed: [1-9][0-9]+ cases\\R"));
    assertEquals("", errorBytes.toString(StandardCharsets.UTF_8));
  }

  @Test
  public void disassemblesTheRustProducedAlphaBundle() throws Exception {
    ByteArrayOutputStream outputBytes = new ByteArrayOutputStream();
    ByteArrayOutputStream errorBytes = new ByteArrayOutputStream();
    int status;
    try (PrintStream output = new PrintStream(outputBytes, true, StandardCharsets.UTF_8);
        PrintStream error = new PrintStream(errorBytes, true, StandardCharsets.UTF_8)) {
      status =
          HaraBytecodeTool.run(
              new String[] {
                "disassemble",
                Path.of(System.getProperty("basedir"), "../rust/assets/std.foundation.hbx")
                    .normalize()
                    .toString()
              },
              output,
              error);
    }
    assertEquals(errorBytes.toString(StandardCharsets.UTF_8), 0, status);
    assertTrue(outputBytes.toString(StandardCharsets.UTF_8).contains("module std.foundation\n"));
    assertTrue(
        outputBytes.toString(StandardCharsets.UTF_8).contains("module std.foundation.string\n"));
    assertEquals("", errorBytes.toString(StandardCharsets.UTF_8));
  }
}
