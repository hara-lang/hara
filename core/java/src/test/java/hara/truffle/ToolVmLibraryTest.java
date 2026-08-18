package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

import hara.lang.data.Keyword;
import hara.lang.data.types.IMapType;
import hara.truffle.bytecode.HbcCodec;
import hara.truffle.bytecode.HbcProgram;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.ServiceLoader;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.StreamSupport;
import org.graalvm.polyglot.Context;
import org.junit.Test;

public class ToolVmLibraryTest {
  @Test
  public void providerIsDiscoverableAndPublicFacadeLoads() {
    Set<String> namespaces =
        StreamSupport.stream(ServiceLoader.load(HaraLibraryProvider.class).spliterator(), false)
            .map(HaraLibraryProvider::namespace)
            .collect(Collectors.toSet());
    assertTrue(namespaces.contains("tool.vm.provider"));

    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertEquals(
          ":truffle",
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns tool.vm.provider-probe (:require [tool.vm :as vm])) "
                      + "(:provider/id (vm/current-provider))")
              .toString());
    }
  }

  @Test
  public void halcValidationAndInspectionUseCanonicalCodec() {
    String source = "(ns sample.vm) (def value 42)";
    Object[] forms = HaraLanguage.readAll(source, "sample/vm.hal");
    byte[] artifact =
        HalcArtifact.encode(
            "sample.vm",
            "sample/vm.hal",
            source.getBytes(StandardCharsets.UTF_8),
            forms);

    assertEquals(
        Boolean.TRUE,
        ToolVmLibrary.validate(null, new Object[] {Keyword.create("halc"), artifact}));
    IMapType<Keyword, Object> inspection =
        (IMapType<Keyword, Object>) ToolVmLibrary.inspect(null, new Object[] {Keyword.create("halc"), artifact});
    assertEquals(Keyword.create("halc"), inspection.lookup(Keyword.create("artifact/format")));
    assertEquals("sample.vm", inspection.lookup(Keyword.create("module/namespace")));
    assertEquals(2L, inspection.lookup(Keyword.create("forms/count")));
  }

  @Test
  public void hbcValidationInspectionAndDisassemblyUseCanonicalCodec() {
    HbcProgram.Function function =
        new HbcProgram.Function(
            "entry",
            false,
            0,
            false,
            0,
            0,
            2,
            List.of(
                new HbcProgram.Instruction(HbcProgram.Opcode.CONSTANT, 0, 0, 0),
                new HbcProgram.Instruction(HbcProgram.Opcode.CONSTANT, 1, 0, 0),
                new HbcProgram.Instruction(
                    HbcProgram.Opcode.PRIMITIVE, HbcProgram.Primitive.ADD.id(), 2, 0),
                new HbcProgram.Instruction(HbcProgram.Opcode.RETURN, 0, 0, 0)),
            java.util.Arrays.asList(null, null, null, null),
            List.of());
    HbcProgram program =
        new HbcProgram(List.of(19L, 23L), List.of(), List.of(function), 0);
    byte[] artifact = HbcCodec.encode(program);

    assertEquals(
        Boolean.TRUE,
        ToolVmLibrary.validate(null, new Object[] {Keyword.create("hbc"), artifact}));
    IMapType<Keyword, Object> inspection =
        (IMapType<Keyword, Object>) ToolVmLibrary.inspect(null, new Object[] {Keyword.create("hbc"), artifact});
    assertEquals(Keyword.create("hbc"), inspection.lookup(Keyword.create("artifact/format")));
    assertEquals(1L, inspection.lookup(Keyword.create("functions/count")));
    assertTrue(ToolVmLibrary.disassemble(null, new Object[] {artifact}).toString().startsWith("HBC0 entry="));
  }
}
