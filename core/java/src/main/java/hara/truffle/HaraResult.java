package hara.truffle;

import hara.lang.base.Eq;
import hara.lang.base.Ex;
import hara.lang.base.G;
import hara.lang.data.Keyword;
import hara.lang.data.types.IMapType;
import hara.lang.protocol.Constant;
import hara.lang.protocol.IDeref;
import hara.lang.protocol.IDisplay;
import hara.lang.protocol.IEquality;
import hara.lang.protocol.IExInfo;
import hara.lang.protocol.IHash;
import java.util.Map.Entry;
import java.util.Objects;

/** A completed native Hara outcome. Context is diagnostic and is not part of identity. */
public final class HaraResult implements IDeref<Object>, IDisplay, IEquality, IHash {
  public enum Status {
    SUCCESS,
    ERROR
  }

  private static final IMapType<Object, Object> EMPTY_CONTEXT =
      hara.lang.data.Map.Standard.EMPTY;

  private final Status status;
  private final Object data;
  private final Ex.Info error;
  private final IMapType<Object, Object> context;

  private HaraResult(
      Status status, Object data, Ex.Info error, IMapType<Object, Object> context) {
    this.status = status;
    this.data = data;
    this.error = error;
    this.context = context;
  }

  public static HaraResult success(Object data) {
    return success(data, EMPTY_CONTEXT);
  }

  public static HaraResult success(Object data, Object context) {
    return new HaraResult(Status.SUCCESS, HaraBox.unwrap(data), null, contextMap(context));
  }

  public static HaraResult error(Object error) {
    return error(error, EMPTY_CONTEXT);
  }

  public static HaraResult error(Object error, Object context) {
    return new HaraResult(Status.ERROR, null, normalizeError(error), contextMap(context));
  }

  public Keyword status() {
    return Keyword.create(status == Status.SUCCESS ? "success" : "error");
  }

  public Object data() {
    return data;
  }

  public Ex.Info errorValue() {
    return error;
  }

  public IMapType<Object, Object> context() {
    return context;
  }

  public boolean isSuccess() {
    return status == Status.SUCCESS;
  }

  public boolean isError() {
    return status == Status.ERROR;
  }

  @SuppressWarnings({"rawtypes", "unchecked"})
  public HaraResult withContext(Object additionalContext) {
    IMapType<Object, Object> additional = contextMap(additionalContext);
    IMapType merged = context;
    for (Object entryValue : additional) {
      Entry entry = (Entry) entryValue;
      merged = (IMapType) merged.assoc(entry.getKey(), entry.getValue());
    }
    return new HaraResult(status, data, error, (IMapType<Object, Object>) merged);
  }

  @Override
  public Object deref() {
    if (isSuccess()) return data;
    throw Ex.Sneaky(error);
  }

  @Override
  public boolean equality(Object other) {
    if (!(HaraBox.unwrap(other) instanceof HaraResult result)) return false;
    return status == result.status
        && Eq.eq(data, result.data)
        && errorEquals(error, result.error);
  }

  @Override
  public long hashCalc(Constant.HashType hashType) {
    long hash = "::RESULT".hashCode();
    hash = hash * 31 + (isSuccess() ? 1 : 2);
    hash = hash * 31 + G.hashCalc(hashType, data);
    hash = hash * 31 + errorHash(error, hashType);
    return hash;
  }

  @Override
  public String display() {
    return "#hara/Result["
        + status().display()
        + " "
        + G.display(data)
        + " "
        + displayError(error)
        + " "
        + G.display(context)
        + "]";
  }

  @Override
  public boolean equals(Object other) {
    return equality(other);
  }

  @Override
  public int hashCode() {
    return Long.hashCode(hashCalc(Constant.HashType.RAPID));
  }

  @Override
  public String toString() {
    return display();
  }

  @SuppressWarnings("unchecked")
  private static IMapType<Object, Object> contextMap(Object value) {
    Object raw = HaraBox.unwrap(value);
    if (!(raw instanceof IMapType<?, ?>)) {
      throw new HaraException("Result context must be a map");
    }
    return (IMapType<Object, Object>) raw;
  }

  private static Ex.Info normalizeError(Object value) {
    Object raw = HaraBox.unwrap(value);
    if (raw instanceof Ex.Info info) return info;
    if (raw instanceof Throwable throwable && raw instanceof IExInfo info) {
      return new Ex.Info(errorMessage(throwable), info.getData(), throwable.getCause());
    }
    if (raw instanceof Throwable throwable) {
      return new Ex.Info(
          errorMessage(throwable),
          hara.lang.data.Map.Standard.from(
              null,
              Keyword.create("error", "class"),
              throwable.getClass().getName(),
              Keyword.create("error", "message"),
              errorMessage(throwable)),
          throwable.getCause());
    }
    return new Ex.Info(
        G.display(raw),
        hara.lang.data.Map.Standard.from(
            null, Keyword.create("error", "value"), raw));
  }

  private static String errorMessage(Throwable error) {
    return error.getMessage() == null ? error.getClass().getName() : error.getMessage();
  }

  private static Object errorData(Throwable error) {
    return error instanceof IExInfo info ? info.getData() : null;
  }

  private static boolean errorEquals(Throwable left, Throwable right) {
    if (left == right) return true;
    if (left == null || right == null) return false;
    return left.getClass().equals(right.getClass())
        && Objects.equals(left.getMessage(), right.getMessage())
        && Eq.eq(errorData(left), errorData(right))
        && errorEquals(left.getCause(), right.getCause());
  }

  private static long errorHash(Throwable error, Constant.HashType hashType) {
    if (error == null) return 0;
    long hash = "::RESULT_ERROR".hashCode();
    hash = hash * 31 + "hara/Error".hashCode();
    hash = hash * 31 + Objects.hashCode(error.getMessage());
    hash = hash * 31 + G.hashCalc(hashType, errorData(error));
    hash = hash * 31 + errorHash(error.getCause(), hashType);
    return hash;
  }

  private static String displayError(Throwable error) {
    if (error == null) return "nil";
    return "#error["
        + G.display(errorMessage(error))
        + " "
        + G.display(errorData(error))
        + "]";
  }
}
