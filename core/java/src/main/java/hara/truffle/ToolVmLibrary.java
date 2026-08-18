package hara.truffle;

import hara.lang.data.Keyword;
import hara.truffle.bytecode.HbcCodec;
import hara.truffle.bytecode.HbcDisassembler;
import hara.truffle.bytecode.HbcProgram;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;

/** Read-only HALC/HBC provider implementation for the Truffle runtime. */
public final class ToolVmLibrary {
  private static final Keyword HALC = Keyword.create("halc");
  private static final Keyword HBC = Keyword.create("hbc");

  private ToolVmLibrary() {}

  @HaraExport(
      name = "provider",
      doc = "Returns the exact read-only VM tooling capabilities of the Truffle runtime.",
      arglists = {"[]"})
  public static Object provider(HaraContext context, Object[] arguments) {
    expectArity("provider", arguments, 0);
    return orderedMap(
        "provider/id", keyword("truffle"),
        "provider/operations", keywords("validate", "inspect", "disassemble"),
        "provider/formats", orderedMap(
            "halc", keywords("validate", "inspect"),
            "hbc", keywords("validate", "inspect", "disassemble")),
        "provider/transforms", vector(),
        "provider/engines", orderedMap());
  }

  @HaraExport(
      name = "validate",
      doc = "Authenticates and validates canonical HALC or HBC bytes.",
      arglists = {"[format bytes]"})
  public static Object validate(HaraContext context, Object[] arguments) {
    expectArity("validate", arguments, 2);
    String format = format(arguments[0], "validate");
    byte[] bytes = bytes(arguments[1], "validate");
    switch (format) {
      case "halc" -> HalcArtifact.decode(bytes);
      case "hbc" -> HbcCodec.decode(bytes);
      default -> throw unsupported(format, "validate");
    }
    return Boolean.TRUE;
  }

  @HaraExport(
      name = "inspect",
      doc = "Returns ordinary Hara metadata derived from a validated HALC or HBC artifact.",
      arglists = {"[format bytes]"})
  public static Object inspect(HaraContext context, Object[] arguments) {
    expectArity("inspect", arguments, 2);
    String format = format(arguments[0], "inspect");
    byte[] bytes = bytes(arguments[1], "inspect");
    return switch (format) {
      case "halc" -> inspectHalc(bytes);
      case "hbc" -> inspectHbc(bytes);
      default -> throw unsupported(format, "inspect");
    };
  }

  @HaraExport(
      name = "disassemble",
      doc = "Returns deterministic HBC diagnostics; this is not source decompilation.",
      arglists = {"[bytes]"})
  public static Object disassemble(HaraContext context, Object[] arguments) {
    expectArity("disassemble", arguments, 1);
    byte[] bytes = bytes(arguments[0], "disassemble");
    return HbcDisassembler.disassemble(HbcCodec.decode(bytes));
  }

  private static Object inspectHalc(byte[] bytes) {
    HalcArtifact.Module module = HalcArtifact.decode(bytes);
    int payloadBytes = unsignedInt(bytes, 8, "HALC payload length");
    return orderedMap(
        "artifact/format", HALC,
        "artifact/version", 1L,
        "artifact/origin", keyword(module.origin == HalcArtifact.Origin.HALC ? "halc" : "legacy-hir"),
        "artifact/bytes", (long) bytes.length,
        "payload/bytes", (long) payloadBytes,
        "payload/checksum", Arrays.copyOfRange(bytes, 12, 44),
        "module/namespace", module.namespace,
        "module/resource", module.resource,
        "source/hash", module.sourceHash.clone(),
        "forms/count", (long) module.forms.length,
        "schemas/definitions", sortedStrings(module.schemas.definitions.keySet()),
        "schemas/functions", sortedStrings(module.schemas.functions.keySet()));
  }

  private static Object inspectHbc(byte[] bytes) {
    HbcProgram program = HbcCodec.decode(bytes);
    int payloadBytes = unsignedInt(bytes, 4, "HBC payload length");
    long instructions = program.functions().stream().mapToLong(function -> function.code().size()).sum();
    long handlers = program.functions().stream().mapToLong(function -> function.handlers().size()).sum();
    return orderedMap(
        "artifact/format", HBC,
        "artifact/version", 0L,
        "artifact/bytes", (long) bytes.length,
        "payload/bytes", (long) payloadBytes,
        "payload/checksum", Arrays.copyOfRange(bytes, 8 + payloadBytes, bytes.length),
        "module/namespace", program.namespace() == null ? HaraNull.SINGLETON : program.namespace(),
        "program/entry", (long) program.entry(),
        "constants/count", (long) program.constants().size(),
        "functions/count", (long) program.functions().size(),
        "instructions/count", instructions,
        "handlers/count", handlers);
  }

  private static int unsignedInt(byte[] bytes, int offset, String field) {
    if (offset < 0 || bytes.length < offset + Integer.BYTES) {
      throw new HaraException("Invalid " + field + ": truncated artifact");
    }
    int value = ByteBuffer.wrap(bytes, offset, Integer.BYTES).order(ByteOrder.BIG_ENDIAN).getInt();
    if (value < 0) throw new HaraException("Invalid " + field + ": length overflow");
    return value;
  }

  private static String format(Object value, String operation) {
    Object unwrapped = HaraBox.unwrap(value);
    if (unwrapped instanceof Keyword keyword
        && keyword.getNamespace() == null
        && (keyword.equals(HALC) || keyword.equals(HBC))) {
      return keyword.getName();
    }
    throw new HaraException(
        "tool.vm.provider/" + operation + " expects :halc or :hbc as its format");
  }

  private static byte[] bytes(Object value, String operation) {
    Object unwrapped = HaraBox.unwrap(value);
    if (unwrapped instanceof byte[] bytes) return bytes.clone();
    throw new HaraException("tool.vm.provider/" + operation + " expects Bytes");
  }

  private static HaraException unsupported(String format, String operation) {
    return new HaraException(
        "tool.vm.provider/" + operation + " does not support format :" + format);
  }

  private static void expectArity(String operation, Object[] arguments, int arity) {
    if (arguments.length != arity) {
      throw new HaraException(
          "tool.vm.provider/" + operation + " expects " + arity + " arguments");
    }
  }

  private static Keyword keyword(String value) {
    return Keyword.create(value);
  }

  private static Object vector(Object... values) {
    return hara.lang.data.Vector.Standard.from(null, values);
  }

  private static Object keywords(String... values) {
    Object[] keywords = new Object[values.length];
    for (int index = 0; index < values.length; index++) keywords[index] = keyword(values[index]);
    return vector(keywords);
  }

  private static Object sortedStrings(Iterable<String> values) {
    ArrayList<String> sorted = new ArrayList<>();
    for (String value : values) sorted.add(value);
    sorted.sort(Comparator.naturalOrder());
    return vector(sorted.toArray());
  }

  private static Object orderedMap(Object... entries) {
    if ((entries.length & 1) != 0) throw new IllegalArgumentException("ordered map requires pairs");
    Object[] values = new Object[entries.length];
    for (int index = 0; index < entries.length; index += 2) {
      values[index] = keyword((String) entries[index]);
      values[index + 1] = entries[index + 1];
    }
    return hara.lang.data.OrderedMap.Standard.from(null, values);
  }
}
