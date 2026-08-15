package hara.truffle;

import hara.kernel.builtin.BuiltinStruct;
import hara.lang.base.Ex;
import hara.lang.data.Keyword;
import hara.lang.data.types.ILinearType;
import hara.lang.data.types.IMapType;
import hara.lang.protocol.IMetadata;
import hara.verify.json.JsonValue;
import hara.verify.json.StrictJson;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/** Compact and indented encoders for strict JSON values and native Result envelopes. */
final class StdJson {
  private StdJson() {}

  static Object read(String source) {
    return decode(StrictJson.parse(source));
  }

  static String write(Object value) {
    Object projected = project(value);
    requireStrictValue(projected);
    StringBuilder out = new StringBuilder();
    append(out, JsonValue.fromHara(projected), 0, false);
    return out.toString();
  }

  static String writePretty(Object value) {
    Object projected = project(value);
    requireStrictValue(projected);
    StringBuilder out = new StringBuilder();
    append(out, JsonValue.fromHara(projected), 0, true);
    return out.toString();
  }

  private static Object project(Object value) {
    Object raw = nullable(value);
    if (raw instanceof HaraResult result) {
      LinkedHashMap<String, Object> envelope = new LinkedHashMap<>();
      envelope.put("$hara", "result");
      envelope.put("status", result.status().getName());
      envelope.put("data", project(result.data()));
      envelope.put("error", project(result.errorValue()));
      envelope.put("context", project(result.transportContext()));
      return envelope;
    }
    if (raw instanceof Ex.Info error) {
      LinkedHashMap<String, Object> envelope = new LinkedHashMap<>();
      envelope.put("$hara", "error");
      envelope.put("message", error.getMessage());
      envelope.put("data", project(error.getData()));
      envelope.put("cause", project(error.getCause()));
      return envelope;
    }
    if (raw instanceof ILinearType<?> values) {
      ArrayList<Object> projected = new ArrayList<>();
      for (Object item : values) projected.add(project(item));
      return projected;
    }
    if (raw instanceof List<?> values) {
      ArrayList<Object> projected = new ArrayList<>();
      for (Object item : values) projected.add(project(item));
      return projected;
    }
    if (raw instanceof IMapType<?, ?> values) {
      LinkedHashMap<String, Object> projected = new LinkedHashMap<>();
      for (var entry : values) {
        if (!(entry.getKey() instanceof String key)) {
          throw new IllegalArgumentException("JSON object keys must be strings.");
        }
        projected.put(key, project(entry.getValue()));
      }
      return projected;
    }
    if (raw instanceof Map<?, ?> values) {
      LinkedHashMap<String, Object> projected = new LinkedHashMap<>();
      for (var entry : values.entrySet()) {
        if (!(entry.getKey() instanceof String key)) {
          throw new IllegalArgumentException("JSON object keys must be strings.");
        }
        projected.put(key, project(entry.getValue()));
      }
      return projected;
    }
    return raw;
  }

  private static Object decode(Object value) {
    Object raw = nullable(value);
    if (raw instanceof ILinearType<?> values) {
      ArrayList<Object> decoded = new ArrayList<>();
      for (Object item : values) decoded.add(decode(item));
      return BuiltinStruct.vector(decoded);
    }
    if (!(raw instanceof IMapType<?, ?> values)) return raw;

    LinkedHashMap<String, Object> decoded = new LinkedHashMap<>();
    for (var entry : values) {
      if (!(entry.getKey() instanceof String key)) return raw;
      decoded.put(key, decode(entry.getValue()));
    }
    Object tag = decoded.get("$hara");
    if ("result".equals(tag)
        && exact(decoded, "$hara", "status", "data", "error", "context")) {
      return decodeResult(decoded);
    }
    if ("error".equals(tag)
        && exact(decoded, "$hara", "message", "data", "cause")) {
      return decodeError(decoded);
    }
    return orderedMap(decoded);
  }

  private static HaraResult decodeResult(Map<String, Object> envelope) {
    Object status = envelope.get("status");
    Object data = nullable(envelope.get("data"));
    Object error = nullable(envelope.get("error"));
    Object context = envelope.get("context");
    if (!(context instanceof IMapType<?, ?>)) {
      throw malformed("Hara Result context");
    }
    if ("success".equals(status)) {
      if (error != null) throw malformed("success Result contains an error");
      return HaraResult.success(data, context);
    }
    if ("error".equals(status)) {
      if (data != null) throw malformed("error Result contains success data");
      if (!(error instanceof Ex.Info)) throw malformed("error Result lacks a native Error");
      return HaraResult.error(error, context);
    }
    throw malformed("Hara Result status");
  }

  private static Ex.Info decodeError(Map<String, Object> envelope) {
    Object message = envelope.get("message");
    Object data = envelope.get("data");
    Object cause = nullable(envelope.get("cause"));
    if (!(message instanceof String)) throw malformed("Hara Error message");
    if (!(data instanceof IMetadata metadata)) throw malformed("Hara Error data");
    if (cause != null && !(cause instanceof Throwable)) throw malformed("Hara Error cause");
    return new Ex.Info((String) message, metadata, (Throwable) cause);
  }

  private static boolean exact(Map<String, Object> value, String... fields) {
    if (value.size() != fields.length) return false;
    for (String field : fields) if (!value.containsKey(field)) return false;
    return true;
  }

  private static Object orderedMap(Map<String, Object> value) {
    ArrayList<Object> entries = new ArrayList<>();
    value.forEach(
        (key, item) -> {
          entries.add(key);
          entries.add(item);
        });
    return BuiltinStruct.orderedMap(entries);
  }

  private static Object nullable(Object value) {
    Object raw = HaraBox.unwrap(value);
    return raw == HaraNull.SINGLETON ? null : raw;
  }

  private static IllegalArgumentException malformed(String field) {
    return new IllegalArgumentException("json/read: malformed " + field + ".");
  }

  private static void append(StringBuilder out, JsonValue value, int depth, boolean pretty) {
    if (value instanceof JsonValue.Null) out.append("null");
    else if (value instanceof JsonValue.Bool bool) out.append(bool.value());
    else if (value instanceof JsonValue.Integer integer) out.append(integer.value());
    else if (value instanceof JsonValue.String string) appendString(out, string.value());
    else if (value instanceof JsonValue.Array array) {
      out.append('[');
      for (int index = 0; index < array.values().size(); index++) {
        if (index > 0) out.append(',');
        if (pretty) newline(out, depth + 1);
        append(out, array.values().get(index), depth + 1, pretty);
      }
      if (pretty && !array.values().isEmpty()) newline(out, depth);
      out.append(']');
    } else if (value instanceof JsonValue.Object object) {
      out.append('{');
      int index = 0;
      for (var entry : object.values().entrySet()) {
        if (index++ > 0) out.append(',');
        if (pretty) newline(out, depth + 1);
        appendString(out, entry.getKey());
        out.append(pretty ? ": " : ":");
        append(out, entry.getValue(), depth + 1, pretty);
      }
      if (pretty && !object.values().isEmpty()) newline(out, depth);
      out.append('}');
    } else throw new IllegalArgumentException("Unsupported strict JSON value");
  }

  private static void newline(StringBuilder out, int depth) {
    out.append('\n');
    out.append("  ".repeat(depth));
  }

  private static void appendString(StringBuilder out, String value) {
    out.append('"');
    for (int index = 0; index < value.length(); index++) {
      char c = value.charAt(index);
      switch (c) {
        case '"' -> out.append("\\\"");
        case '\\' -> out.append("\\\\");
        case '\b' -> out.append("\\b");
        case '\f' -> out.append("\\f");
        case '\n' -> out.append("\\n");
        case '\r' -> out.append("\\r");
        case '\t' -> out.append("\\t");
        default -> {
          if (c < 0x20) out.append(String.format("\\u%04x", (int) c));
          else out.append(c);
        }
      }
    }
    out.append('"');
  }

  private static void requireStrictValue(Object value) {
    if (value == null || value instanceof Boolean || value instanceof String) return;
    if (value instanceof Byte || value instanceof Short || value instanceof Integer || value instanceof Long) return;
    if (value instanceof java.math.BigInteger integer) {
      try {
        integer.longValueExact();
        return;
      } catch (ArithmeticException error) {
        throw new IllegalArgumentException("JSON integers must fit in the signed 64-bit range.");
      }
    }
    if (value instanceof ILinearType<?> vector) {
      for (Object item : vector) requireStrictValue(item);
      return;
    }
    if (value instanceof List<?> vector) {
      for (Object item : vector) requireStrictValue(item);
      return;
    }
    if (value instanceof IMapType<?, ?> map) {
      for (var entry : map) {
        if (!(entry.getKey() instanceof String)) {
          throw new IllegalArgumentException("JSON object keys must be strings.");
        }
        requireStrictValue(entry.getValue());
      }
      return;
    }
    if (value instanceof Map<?, ?> map) {
      for (var entry : map.entrySet()) {
        if (!(entry.getKey() instanceof String)) {
          throw new IllegalArgumentException("JSON object keys must be strings.");
        }
        requireStrictValue(entry.getValue());
      }
      return;
    }
    throw new IllegalArgumentException(
        "JSON values must be nil, booleans, signed 64-bit integers, strings, vectors, string-key maps, or native Result/Error envelopes containing those values.");
  }
}
