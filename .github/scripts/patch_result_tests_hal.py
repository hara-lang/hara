from __future__ import annotations

import argparse
from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one anchor, found {count}")
    return text.replace(old, new, 1)


def insert_once(text: str, anchor: str, addition: str, label: str) -> str:
    return replace_once(text, anchor, addition + anchor, label)


FOUNDATION_SECTION = r'''
;; ---------------------------------------------------------------------------
;; Completed outcomes
;; ---------------------------------------------------------------------------

(defn ^{:schema [:function [:fn [:any] :any]
                            [:fn [:any :map] :any]]}
  res-success
  "Creates a successful native Result carrying data and optional context."
  ([data]
   (Result/success data))
  ([data context]
   (Result/success data context)))

(defn ^{:schema [:function [:fn [:any] :any]
                            [:fn [:any :map] :any]]}
  res-error
  "Creates an error native Result carrying a normalized Error and optional context."
  ([error]
   (Result/error error))
  ([error context]
   (Result/error error context)))

(defn ^{:schema [:function [:fn [:any] :any]
                            [:fn [:any :map] :any]]}
  res-synchronize
  "Normalizes one completed value, dereference, or Promise into a native Result."
  ([value]
   (Result/synchronize value))
  ([value options]
   (Result/synchronize value options)))

(defn ^{:schema [:fn [:any] :bool]}
  res?
  "Returns true when value is a native Result."
  [value]
  (Result/result? value))

(defn ^{:schema [:fn [:any] :bool]}
  res-success?
  "Returns true when result records a successful outcome."
  [result]
  (Result/success? result))

(defn ^{:schema [:fn [:any] :bool]}
  res-error?
  "Returns true when result records an error outcome."
  [result]
  (Result/error? result))

(defn ^{:schema [:fn [:any] :any]}
  res-status
  "Returns :success or :error from result."
  [result]
  (Result/status result))

(defn ^{:schema [:fn [:any] :any]}
  res-data
  "Returns the success data stored by result."
  [result]
  (Result/data result))

(defn ^{:schema [:fn [:any] :any]}
  res-error-value
  "Returns the normalized native Error stored by an error result, or nil."
  [result]
  (Result/error-value result))

(defn ^{:schema [:fn [:any] :map]}
  res-context
  "Returns result's diagnostic context map."
  [result]
  (Result/context result))

(defn ^{:schema [:fn [:any :map] :any]}
  res-with-context
  "Shallow-merges additional diagnostic context into result."
  [result additional-context]
  (Result/with-context result additional-context))

'''


def foundation_candidate() -> str:
    path = Path("core/lib/src/std/foundation.hal")
    text = path.read_text()
    return insert_once(
        text,
        ''';; ---------------------------------------------------------------------------
;; Functions and composition
;; ---------------------------------------------------------------------------
''',
        FOUNDATION_SECTION,
        "Foundation res-* facade",
    )


def write_candidate() -> None:
    target = Path("core/target/foundation-result-candidate.hal")
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(foundation_candidate())
    print(target)


def patch_foundation_sources() -> None:
    candidate_path = Path("core/target/foundation-result-candidate.hal")
    candidate = candidate_path.read_text() if candidate_path.exists() else foundation_candidate()
    Path("core/lib/src/std/foundation.hal").write_text(candidate)
    Path("core/rust/hal-src/std/foundation.hal").write_text(candidate)


def patch_foundation_test() -> None:
    path = Path("core/lib/test/std/foundation_test.hal")
    text = path.read_text()
    tests = r'''   (fact "res-* constructs and inspects native Results"
     (let [success (res-success 42 {:source :left})
           updated (res-with-context success {:source :right :added true})
           error (res-error (ex-info "boom" {:code :boom}) {:source :test})]
       [(res? success)
        (res-success? success)
        (res-error? success)
        (res-status success)
        (res-data success)
        (res-error-value success)
        (get (res-context updated) :source)
        (get (res-context updated) :added)
        (res-error? error)
        (type error)])
     => [true true false :success 42 nil :right true true :hara/Result])
   (fact "res-synchronize dereferences once without flattening nested Results"
     (let [nested (res-success 7)
           raw (res-synchronize 42)
           merged (res-synchronize nested {:context {:source :sync}})
           wrapped (res-synchronize (atom nested))]
       [(res-success? raw)
        (res-data raw)
        (= nested merged)
        (= :sync (get (res-context merged) :source))
        (res-success? wrapped)
        (res? (res-data wrapped))])
     => [true 42 true true true true])
'''
    text = insert_once(
        text,
        '''   (fact "comp composes right-to-left"
''',
        tests,
        "Foundation res-* tests",
    )
    path.write_text(text)


def patch_rust_tests() -> None:
    path = Path("core/rust/src/core/native_result.rs")
    text = path.read_text()
    tests = r'''    #[test]
    fn synchronize_raw_existing_and_nested_results() {
        let raw = synchronize_value(Value::Number(42), None, Value::Map(PMap::new()))
            .expect("raw synchronization");
        let Value::Result(raw) = raw else {
            panic!("expected Result");
        };
        assert!(raw.is_success());
        assert_eq!(raw.data, Value::Number(42));

        let existing = Rc::new(
            ResultValue::success(
                Value::Number(7),
                context("source", Value::String("left".into())),
            )
            .expect("existing Result"),
        );
        let synchronized = synchronize_value(
            Value::Result(existing.clone()),
            None,
            context("source", Value::String("right".into())),
        )
        .expect("existing synchronization");
        let Value::Result(synchronized) = synchronized else {
            panic!("expected Result");
        };
        assert_eq!(synchronized.as_ref(), existing.as_ref());
        let source = super::super::map_value(
            &synchronized.context,
            &Value::Keyword(Keyword::from("source")),
        )
        .expect("source context");
        assert!(matches!(source, Value::String(value) if value == "right"));

        let promise = super::super::Promise::new();
        promise.resolve(Value::Result(existing.clone()));
        let wrapped = synchronize_value(
            Value::Promise(promise),
            None,
            Value::Map(PMap::new()),
        )
        .expect("nested synchronization");
        let Value::Result(wrapped) = wrapped else {
            panic!("expected Result");
        };
        assert!(matches!(
            &wrapped.data,
            Value::Result(value) if Rc::ptr_eq(value, &existing)
        ));
    }

    #[test]
    fn synchronize_captures_rejection_timeout_and_cancellation_failure() {
        let error = Rc::new(ExceptionInfo {
            message: "rejected".into(),
            data: Box::new(context(
                "code",
                Value::Keyword(Keyword::from("rejected")),
            )),
            cause: None,
        });
        let rejected = super::super::Promise::new();
        rejected.reject_value(Value::ExceptionInfo(error.clone()));
        let captured = synchronize_value(
            Value::Promise(rejected),
            None,
            Value::Map(PMap::new()),
        )
        .expect("rejection synchronization");
        let Value::Result(captured) = captured else {
            panic!("expected Result");
        };
        assert!(captured.is_error());
        assert!(matches!(
            captured.error_value(),
            Value::ExceptionInfo(value) if Rc::ptr_eq(&value, &error)
        ));

        let timed = super::super::Promise::new();
        let timeout = synchronize_value(
            Value::Promise(timed.clone()),
            Some(0),
            Value::Map(PMap::new()),
        )
        .expect("timeout synchronization");
        let Value::Result(timeout) = timeout else {
            panic!("expected Result");
        };
        assert!(timeout.is_error());
        let Value::ExceptionInfo(timeout_error) = timeout.error_value() else {
            panic!("expected timeout Error");
        };
        let code = super::super::map_value(
            timeout_error.data.as_ref(),
            &Value::Keyword(Keyword::from("code")),
        )
        .expect("timeout code");
        assert_eq!(code, &Value::Keyword(Keyword::from("result/timeout")));
        assert!(matches!(timed.state(), PromiseState::Rejected(_)));

        let cancellation_failure = super::super::Promise::new();
        cancellation_failure.set_cancel_hook(Rc::new(|| panic!("cannot cancel")));
        let timeout = synchronize_value(
            Value::Promise(cancellation_failure),
            Some(0),
            Value::Map(PMap::new()),
        )
        .expect("cancellation failure synchronization");
        let Value::Result(timeout) = timeout else {
            panic!("expected Result");
        };
        assert!(super::super::map_value(
            &timeout.context,
            &Value::Keyword(Keyword::from("result/cancellation-error")),
        )
        .is_some());
    }

'''
    text = insert_once(
        text,
        '''    #[test]
    fn native_result_error_preserves_native_error_and_deref_throws() {
''',
        tests,
        "Rust Result synchronize tests",
    )
    path.write_text(text)


def patch_java_tests() -> None:
    path = Path("core/java/src/test/java/hara/truffle/HaraResultTest.java")
    text = path.read_text()
    text = replace_once(
        text,
        '''import hara.lang.protocol.Constant;
import org.graalvm.polyglot.Context;
''',
        '''import hara.lang.protocol.Constant;
import hara.lang.protocol.IDeref;
import hara.lang.protocol.IPromise;
import java.util.concurrent.atomic.AtomicBoolean;
import org.graalvm.polyglot.Context;
''',
        "Java Result test imports",
    )

    tests = r'''  @Test
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

'''
    text = insert_once(
        text,
        '''  @Test
  public void languageExportsExposeTheNativeContract() {
''',
        tests,
        "Java Result synchronization tests",
    )

    text = replace_once(
        text,
        '''                      + "(def e (std.native.Result/error (ex-info \\"boom\\" {:code :boom}) {:source :test})) "
                      + "(and "
''',
        '''                      + "(def e (std.native.Result/error (ex-info \\"boom\\" {:code :boom}) {:source :test})) "
                      + "(def s (std.native.Result/synchronize 9 {:context {:source :sync}})) "
                      + "(def n (std.native.Result/synchronize (atom r))) "
                      + "(def f (res-success 11 {:source :foundation})) "
                      + "(and "
''',
        "Java Result language setup",
    )
    text = replace_once(
        text,
        '''                      + "(std.native.Result/error? e) "
                      + "(= :hara/Error (type (std.native.Result/error-value e)))))")
''',
        '''                      + "(std.native.Result/error? e) "
                      + "(= :hara/Error (type (std.native.Result/error-value e))) "
                      + "(std.native.Result/success? s) "
                      + "(= 9 (std.native.Result/data s)) "
                      + "(= :sync (get (std.native.Result/context s) :source)) "
                      + "(std.native.Result/result? (std.native.Result/data n)) "
                      + "(res? f) "
                      + "(res-success? f) "
                      + "(= 11 (res-data f)) "
                      + "(= :foundation (get (res-context f) :source))))")
''',
        "Java Result language checks",
    )
    path.write_text(text)


def apply() -> None:
    patch_foundation_sources()
    patch_foundation_test()
    patch_rust_tests()
    patch_java_tests()
    print("Result tests and std.foundation res-* facade applied")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=("candidate", "apply"))
    args = parser.parse_args()
    if args.command == "candidate":
        write_candidate()
    else:
        apply()


if __name__ == "__main__":
    main()
