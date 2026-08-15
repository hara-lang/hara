package hara.truffle;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertSame;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import hara.lang.base.Ex;
import hara.lang.data.Keyword;
import hara.lang.data.types.IMapType;
import hara.lang.protocol.Constant;
import hara.lang.protocol.IDeref;
import hara.lang.protocol.IPromise;
import java.util.concurrent.atomic.AtomicBoolean;
import org.graalvm.polyglot.Context;
import org.junit.Test;

public class HaraResultTest {
  @Test
  public void equalityAndHashIgnoreContext() {
    IMapType<Object, Object> leftContext =
        hara.lang.data.Map.Standard.from(
            null, Keyword.create("source"), Keyword.create("left"));
    IMapType<Object, Object> rightContext =
        hara.lang.data.Map.Standard.from(
            null, Keyword.create("source"), Keyword.create("right"));
    HaraResult left = HaraResult.success(42L, leftContext);
    HaraResult right = HaraResult.success(42L, rightContext);

    assertTrue(left.equality(right));
    assertEquals(left.hashCalc(Constant.HashType.RAPID), right.hashCalc(Constant.HashType.RAPID));
    assertEquals(42L, left.deref());
    assertEquals(Keyword.create("success"), left.status());
  }

  @Test
  public void contextMergeUsesSuppliedKeysWithoutChangingOutcome() {
    HaraResult result =
        HaraResult.success(
            7L,
            hara.lang.data.Map.Standard.from(
                null,
                Keyword.create("source"),
                Keyword.create("left"),
                Keyword.create("kept"),
                Boolean.TRUE));
    HaraResult updated =
        result.withContext(
            hara.lang.data.Map.Standard.from(
                null,
                Keyword.create("source"),
                Keyword.create("right"),
                Keyword.create("added"),
                1L));

    assertTrue(result.equality(updated));
    assertEquals(Keyword.create("right"), updated.context().lookup(Keyword.create("source")));
    assertEquals(Boolean.TRUE, updated.context().lookup(Keyword.create("kept")));
    assertEquals(1L, updated.context().lookup(Keyword.create("added")));
  }

  @Test
  public void errorDerefThrowsThePreservedNativeError() {
    Ex.Info error =
        new Ex.Info(
            "boom",
            hara.lang.data.Map.Standard.from(
                null, Keyword.create("code"), Keyword.create("boom")));
    HaraResult result = HaraResult.error(error);

    assertSame(error, result.errorValue());
    Ex.Info thrown = assertThrows(Ex.Info.class, () -> result.deref());
    assertSame(error, thrown);
    assertTrue(result.display().startsWith("#hara/Result[:error"));
  }

  @Test
  public void synchronizeCapturesValuesDereferencesAndNestedResults() {
    HaraResult raw = HaraResult.synchronize(42L);
    assertTrue(raw.isSuccess());
    assertEquals(42L, raw.data());

    HaraResult nested = HaraResult.success(7L);
    IDeref<Object> dereferenceable = () -> nested;
    HaraResult wrapped = HaraResult.synchronize(dereferenceable);
    assertTrue(wrapped.isSuccess());
    assertSame(nested, wrapped.data());

    Ex.Info failure =
        new Ex.Info(
            "boom",
            hara.lang.data.Map.Standard.from(
                null, Keyword.create("code"), Keyword.create("boom")));
    HaraResult captured =
        HaraResult.synchronize(
            (IDeref<Object>)
                () -> {
                  throw failure;
                });
    assertTrue(captured.isError());
    assertSame(failure, captured.errorValue());

    HaraResult merged =
        HaraResult.synchronize(
            nested,
            hara.lang.data.Map.Standard.from(
                null,
                Keyword.create("context"),
                hara.lang.data.Map.Standard.from(
                    null, Keyword.create("source"), Keyword.create("sync"))));
    assertTrue(nested.equality(merged));
    assertEquals(Keyword.create("sync"), merged.context().lookup(Keyword.create("source")));
  }

  @Test
  public void synchronizeTimeoutCancelsPromisesAndRejectsUnsupportedTimedDerefs() {
    PendingPromise promise = new PendingPromise(false);
    HaraResult timeout =
        HaraResult.synchronize(
            promise,
            hara.lang.data.Map.Standard.from(
                null, Keyword.create("timeout"), 0L));
    assertTrue(timeout.isError());
    assertTrue(promise.cancelled.get());
    assertEquals(
        Keyword.create("result", "timeout"),
        errorData(timeout).lookup(Keyword.create("code")));
    assertEquals(
        Boolean.TRUE,
        timeout.context().lookup(Keyword.create("result", "cancelled")));

    HaraResult unsupported =
        HaraResult.synchronize(
            (IDeref<Object>) () -> 1L,
            hara.lang.data.Map.Standard.from(
                null, Keyword.create("timeout"), 0L));
    assertTrue(unsupported.isError());
    assertEquals(
        Keyword.create("result", "timeout-unsupported"),
        errorData(unsupported).lookup(Keyword.create("code")));

    HaraResult cancellationFailure =
        HaraResult.synchronize(
            new PendingPromise(true),
            hara.lang.data.Map.Standard.from(
                null, Keyword.create("timeout"), 0L));
    assertTrue(cancellationFailure.isError());
    assertEquals(
        Boolean.FALSE,
        cancellationFailure.context().lookup(Keyword.create("result", "cancelled")));
    assertTrue(
        cancellationFailure
                .context()
                .lookup(Keyword.create("result", "cancellation-error"))
            instanceof String);
  }

  @SuppressWarnings("unchecked")
  private static IMapType<Object, Object> errorData(HaraResult result) {
    return (IMapType<Object, Object>) result.errorValue().getData();
  }

  private static final class PendingPromise implements IPromise {
    private final boolean failCancellation;
    private final AtomicBoolean cancelled = new AtomicBoolean();

    private PendingPromise(boolean failCancellation) {
      this.failCancellation = failCancellation;
    }

    @Override
    public Object state() {
      return Keyword.create("pending");
    }

    @Override
    public Object value() {
      throw new HaraException("promise is pending");
    }

    @Override
    public Object then(Object function) {
      return this;
    }

    @Override
    public Object catchError(Object function) {
      return this;
    }

    @Override
    public Object finallyDo(Object function) {
      return this;
    }

    @Override
    public Object cancel() {
      if (failCancellation) throw new HaraException("cannot cancel");
      cancelled.set(true);
      return this;
    }

    @Override
    public Object deref() {
      throw new HaraException("promise is pending");
    }

    @Override
    public Object derefTimeout(long milliseconds, Object timeoutValue) {
      return timeoutValue;
    }
  }

  @Test
  public void languageExportsExposeTheNativeContract() {
    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertTrue(
          context
              .eval(
                  HaraLanguage.ID,
                  "(do "
                      + "(def r (std.native.Result/success 42 {:source :left})) "
                      + "(def e (std.native.Result/error (ex-info \"boom\" {:code :boom}) {:source :test})) "
                      + "(def s (std.native.Result/synchronize 9 {:context {:source :sync}})) "
                      + "(def n (std.native.Result/synchronize (atom r))) "
                      + "(and "
                      + "(= :hara/Result (type r)) "
                      + "(std.native.Result/result? r) "
                      + "(std.native.Result/success? r) "
                      + "(not (std.native.Result/error? r)) "
                      + "(= :success (std.native.Result/status r)) "
                      + "(= 42 (std.native.Result/data r)) "
                      + "(= nil (std.native.Result/error-value r)) "
                      + "(= 42 (deref r)) "
                      + "(= r (std.native.Result/success 42 {:source :right})) "
                      + "(= :right (get (std.native.Result/context "
                      + "(std.native.Result/with-context r {:source :right})) :source)) "
                      + "(std.native.Result/error? e) "
                      + "(= :hara/Error (type (std.native.Result/error-value e))) "
                      + "(std.native.Result/success? s) "
                      + "(= 9 (std.native.Result/data s)) "
                      + "(= :sync (get (std.native.Result/context s) :source)) "
                      + "(std.native.Result/result? (std.native.Result/data n))))")
              .asBoolean());
    }
  }
}
