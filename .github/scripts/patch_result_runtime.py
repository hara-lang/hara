from __future__ import annotations

from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one anchor, found {count}")
    return text.replace(old, new, 1)


def insert_once(text: str, anchor: str, addition: str, label: str) -> str:
    return replace_once(text, anchor, addition + anchor, label)


def patch_rust_core() -> None:
    path = Path("core/rust/src/core.rs")
    text = path.read_text()

    text = replace_once(
        text,
        '''        &[
            "success",
            "error",
            "result?",
''',
        '''        &[
            "success",
            "error",
            "synchronize",
            "result?",
''',
        "Rust Result native type descriptor",
    )

    text = replace_once(
        text,
        '''fn result_context(value: Option<Value>) -> Result<Value, String> {
    let context = value.unwrap_or_else(|| Value::Map(PMap::new()));
    map_entries(&context)
        .is_some()
        .then_some(context)
        .ok_or_else(|| "Result context must be a map".into())
}

fn native_result_operation(
''',
        '''fn result_context(value: Option<Value>) -> Result<Value, String> {
    let context = value.unwrap_or_else(|| Value::Map(PMap::new()));
    map_entries(&context)
        .is_some()
        .then_some(context)
        .ok_or_else(|| "Result context must be a map".into())
}

fn result_synchronize_options(options: Option<Value>) -> Result<(Option<u64>, Value), String> {
    let Some(options) = options else {
        return Ok((None, Value::Map(PMap::new())));
    };
    if map_entries(&options).is_none() {
        return Err("std.native.Result/synchronize expects an options map".into());
    }
    let timeout_key = Value::Keyword(Keyword::from("timeout"));
    let context_key = Value::Keyword(Keyword::from("context"));
    let timeout = match map_value(&options, &timeout_key) {
        None | Some(Value::Nil) => None,
        Some(value) => Some(
            value_u64_integer(value, "std.native.Result/synchronize").map_err(|_| {
                "std.native.Result/synchronize timeout must be a non-negative integer"
                    .to_string()
            })?,
        ),
    };
    let context = result_context(map_value(&options, &context_key).cloned())?;
    Ok((timeout, context))
}

fn native_result_operation(
''',
        "Rust Result synchronize options",
    )

    text = replace_once(
        text,
        '''        "error" => {
            if !(1..=2).contains(&forms.len()) {
                return Err(
                    "std.native.Result/error expects an error and optional context".into(),
                );
            }
            let error = eval(&forms[0], env)?;
            let context = result_context(
                forms
                    .get(1)
                    .map(|form| eval(form, env))
                    .transpose()?,
            )?;
            Ok(Value::Result(Rc::new(ResultValue::error(error, context)?)))
        }
        "result?" | "success?" | "error?" => {
''',
        '''        "error" => {
            if !(1..=2).contains(&forms.len()) {
                return Err(
                    "std.native.Result/error expects an error and optional context".into(),
                );
            }
            let error = eval(&forms[0], env)?;
            let context = result_context(
                forms
                    .get(1)
                    .map(|form| eval(form, env))
                    .transpose()?,
            )?;
            Ok(Value::Result(Rc::new(ResultValue::error(error, context)?)))
        }
        "synchronize" => {
            if !(1..=2).contains(&forms.len()) {
                return Err(
                    "std.native.Result/synchronize expects a value and optional options map"
                        .into(),
                );
            }
            let value = eval(&forms[0], env)?;
            let options = forms
                .get(1)
                .map(|form| eval(form, env))
                .transpose()?;
            let (timeout, context) = result_synchronize_options(options)?;
            native_result::synchronize_value(value, timeout, context)
        }
        "result?" | "success?" | "error?" => {
''',
        "Rust Result synchronize operation",
    )
    path.write_text(text)


def patch_rust_result() -> None:
    path = Path("core/rust/src/core/native_result.rs")
    text = path.read_text()

    text = replace_once(
        text,
        '''use super::{map_entries, thrown_error, ExceptionInfo, Value};
''',
        '''use super::{
    caught_error, map_entries, protocol_deref, protocol_deref_timeout, thrown_error,
    ExceptionInfo, PromiseRejection, PromiseState, Value,
};
''',
        "Rust Result imports",
    )
    text = replace_once(
        text,
        '''use std::cmp::Ordering;
use std::rc::Rc;
''',
        '''use std::cell::RefCell;
use std::cmp::Ordering;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;
use std::time::Duration;
''',
        "Rust Result standard imports",
    )

    synchronization = r'''const DEREF_UNSUPPORTED: &str = "IDeref/deref has no implementation for this value";
const DEREF_TIMEOUT_UNSUPPORTED: &str =
    "IDerefTimeout/deref-timeout expects a dereferenceable value, milliseconds, and timeout value";

pub(super) fn synchronize_value(
    value: Value,
    timeout: Option<u64>,
    context: Value,
) -> Result<Value, String> {
    let context = validate_context(context)?;
    if let Value::Result(result) = value {
        if map_entries(&context)
            .expect("validated Result context")
            .is_empty()
        {
            return Ok(Value::Result(result));
        }
        return Ok(Value::Result(Rc::new(result.with_context(context)?)));
    }

    let result = match value {
        Value::Promise(promise) => synchronize_promise(promise, timeout, context)?,
        value => match timeout {
            Some(milliseconds) => synchronize_timed(value, milliseconds, context)?,
            None => synchronize_untimed(value, context)?,
        },
    };
    Ok(Value::Result(Rc::new(result)))
}

fn synchronize_untimed(value: Value, context: Value) -> Result<ResultValue, String> {
    match protocol_deref(std::slice::from_ref(&value)) {
        Ok(data) => ResultValue::success(data, context),
        Err(error) if error == DEREF_UNSUPPORTED => ResultValue::success(value, context),
        Err(error) => ResultValue::error(caught_error(&error), context),
    }
}

fn synchronize_timed(
    value: Value,
    milliseconds: u64,
    context: Value,
) -> Result<ResultValue, String> {
    let marker = Value::Array(Rc::new(RefCell::new(Vec::new())));
    let milliseconds_value = Value::Number(i64::try_from(milliseconds).unwrap_or(i64::MAX));
    match protocol_deref_timeout(&[value.clone(), milliseconds_value, marker.clone()]) {
        Ok(resolved) if same_marker(&resolved, &marker) => {
            timeout_result(milliseconds, context, None)
        }
        Ok(data) => ResultValue::success(data, context),
        Err(error) if error == DEREF_TIMEOUT_UNSUPPORTED => {
            if matches!(value, Value::Pointer(_)) {
                timeout_unsupported_result(milliseconds, context)
            } else {
                ResultValue::success(value, context)
            }
        }
        Err(error) => ResultValue::error(caught_error(&error), context),
    }
}

fn synchronize_promise(
    promise: super::Promise,
    timeout: Option<u64>,
    context: Value,
) -> Result<ResultValue, String> {
    let state = match timeout {
        Some(milliseconds) => promise.wait_state_timeout(Duration::from_millis(milliseconds)),
        None => promise.wait_state(),
    };
    match state {
        PromiseState::Fulfilled(data) => ResultValue::success(data, context),
        PromiseState::Rejected(error) => {
            ResultValue::error(promise_rejection_value(error), context)
        }
        PromiseState::Pending => timeout_result(
            timeout.expect("only timed Promise synchronization can remain pending"),
            context,
            Some(promise),
        ),
    }
}

fn promise_rejection_value(error: PromiseRejection) -> Value {
    error.value()
}

fn timeout_result(
    milliseconds: u64,
    context: Value,
    promise: Option<super::Promise>,
) -> Result<ResultValue, String> {
    let mut details = vec![
        (
            Value::Keyword(Keyword::from("result/timeout")),
            Value::Number(i64::try_from(milliseconds).unwrap_or(i64::MAX)),
        ),
        (
            Value::Keyword(Keyword::from("result/cancellation-requested")),
            Value::Bool(promise.is_some()),
        ),
    ];

    if let Some(promise) = promise {
        match catch_unwind(AssertUnwindSafe(|| promise.cancel())) {
            Ok(cancelled) => details.push((
                Value::Keyword(Keyword::from("result/cancelled")),
                Value::Bool(cancelled),
            )),
            Err(payload) => {
                details.push((
                    Value::Keyword(Keyword::from("result/cancelled")),
                    Value::Bool(false),
                ));
                details.push((
                    Value::Keyword(Keyword::from("result/cancellation-error")),
                    Value::String(panic_message(payload)),
                ));
            }
        }
    }

    ResultValue::error(
        result_error(
            "result/timeout",
            "Result synchronization timed out",
            milliseconds,
        ),
        context_with(context, details),
    )
}

fn timeout_unsupported_result(
    milliseconds: u64,
    context: Value,
) -> Result<ResultValue, String> {
    ResultValue::error(
        result_error(
            "result/timeout-unsupported",
            "Timed synchronization is unsupported for this dereferenceable value",
            milliseconds,
        ),
        context_with(
            context,
            [(
                Value::Keyword(Keyword::from("result/timeout")),
                Value::Number(i64::try_from(milliseconds).unwrap_or(i64::MAX)),
            )],
        ),
    )
}

fn result_error(code: &str, message: &str, milliseconds: u64) -> Value {
    Value::ExceptionInfo(Rc::new(ExceptionInfo {
        message: message.into(),
        data: Box::new(Value::Map(PMap::from_iter([
            (
                Value::Keyword(Keyword::from("code")),
                Value::Keyword(Keyword::from(code)),
            ),
            (
                Value::Keyword(Keyword::from("message")),
                Value::String(message.into()),
            ),
            (
                Value::Keyword(Keyword::from("timeout")),
                Value::Number(i64::try_from(milliseconds).unwrap_or(i64::MAX)),
            ),
        ]))),
        cause: None,
    }))
}

fn context_with(
    context: Value,
    entries: impl IntoIterator<Item = (Value, Value)>,
) -> Value {
    let mut merged = PMap::new();
    for (key, value) in map_entries(&context).expect("validated Result context") {
        merged = merged.assoc_value(key, value);
    }
    for (key, value) in entries {
        merged = merged.assoc_value(key, value);
    }
    Value::Map(merged)
}

fn same_marker(left: &Value, right: &Value) -> bool {
    matches!(
        (left, right),
        (Value::Array(left), Value::Array(right)) if Rc::ptr_eq(left, right)
    )
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "Promise cancellation panicked".into())
}

'''
    text = insert_once(
        text,
        '''impl PartialEq for ResultValue {
''',
        synchronization,
        "Rust Result synchronization helpers",
    )
    path.write_text(text)


def patch_java_context() -> None:
    path = Path("core/java/src/main/java/hara/truffle/HaraContext.java")
    text = path.read_text()
    text = replace_once(
        text,
        '''                  "success", "error", "result?", "success?", "error?", "status",
''',
        '''                  "success", "error", "synchronize", "result?", "success?", "error?", "status",
''',
        "Java Result native type descriptor",
    )
    text = insert_once(
        text,
        '''    result.define(
        "result?",
''',
        '''    result.define(
        "synchronize",
        new VariadicBuiltin(
            "std.native.Result/synchronize",
            values -> {
              if (values.length < 1 || values.length > 2) {
                throw new HaraException(
                    "std.native.Result/synchronize expects a value and optional options map");
              }
              return values.length == 1
                  ? HaraResult.synchronize(HaraBox.unwrap(values[0]))
                  : HaraResult.synchronize(
                      HaraBox.unwrap(values[0]), HaraBox.unwrap(values[1]));
            }));
''',
        "Java Result synchronize builtin",
    )
    path.write_text(text)


def patch_java_result() -> None:
    path = Path("core/java/src/main/java/hara/truffle/HaraResult.java")
    text = path.read_text()
    text = replace_once(
        text,
        '''import hara.lang.protocol.IDeref;
import hara.lang.protocol.IDisplay;
''',
        '''import hara.lang.protocol.IDeref;
import hara.lang.protocol.IDerefTimeout;
import hara.lang.protocol.IDisplay;
''',
        "Java Result timed deref import",
    )
    text = replace_once(
        text,
        '''import hara.lang.protocol.IHash;
import java.util.Map.Entry;
''',
        '''import hara.lang.protocol.IHash;
import hara.lang.protocol.IPromise;
import java.util.Map.Entry;
''',
        "Java Result Promise import",
    )
    text = replace_once(
        text,
        '''  private static final IMapType<Object, Object> EMPTY_CONTEXT =
      hara.lang.data.Map.Standard.EMPTY;

''',
        '''  private static final IMapType<Object, Object> EMPTY_CONTEXT =
      hara.lang.data.Map.Standard.EMPTY;
  private static final Object MISSING = new Object();
  private static final Object TIMEOUT = new Object();
  private static final Keyword TIMEOUT_KEY = Keyword.create("timeout");
  private static final Keyword CONTEXT_KEY = Keyword.create("context");

''',
        "Java Result synchronization constants",
    )

    synchronization = r'''  public static HaraResult synchronize(Object value) {
    return synchronize(value, null);
  }

  @SuppressWarnings({"rawtypes", "unchecked"})
  public static HaraResult synchronize(Object value, Object options) {
    Object raw = HaraBox.unwrap(value);
    SynchronizeOptions parsed = synchronizeOptions(options);

    if (raw instanceof HaraResult result) {
      return parsed.context().count() == 0 ? result : result.withContext(parsed.context());
    }

    if (parsed.timeout() != null) {
      if (raw instanceof IDerefTimeout<?> timed) {
        try {
          Object resolved = ((IDerefTimeout) timed).derefTimeout(parsed.timeout(), TIMEOUT);
          if (resolved == TIMEOUT) {
            return timeoutResult(raw, parsed.timeout(), parsed.context());
          }
          return success(resolved, parsed.context());
        } catch (Throwable error) {
          return error(error, parsed.context());
        }
      }
      if (raw instanceof IDeref<?>) {
        return timeoutUnsupportedResult(parsed.timeout(), parsed.context());
      }
      return success(raw, parsed.context());
    }

    if (raw instanceof IDeref<?> dereferenceable) {
      try {
        return success(dereferenceable.deref(), parsed.context());
      } catch (Throwable error) {
        return error(error, parsed.context());
      }
    }
    return success(raw, parsed.context());
  }

  private static SynchronizeOptions synchronizeOptions(Object options) {
    Object raw = HaraBox.unwrap(options);
    if (raw == null) return new SynchronizeOptions(null, EMPTY_CONTEXT);
    if (!(raw instanceof IMapType<?, ?>)) {
      throw new HaraException("std.native.Result/synchronize expects an options map");
    }
    @SuppressWarnings("unchecked")
    IMapType<Object, Object> map = (IMapType<Object, Object>) raw;

    Object timeoutValue = map.lookup(TIMEOUT_KEY, MISSING);
    Long timeout = null;
    if (timeoutValue != MISSING && HaraBox.unwrap(timeoutValue) != null) {
      Object numeric = HaraBox.unwrap(timeoutValue);
      if (!(numeric instanceof Number number)
          || number.longValue() < 0
          || number.doubleValue() != (double) number.longValue()) {
        throw new HaraException(
            "std.native.Result/synchronize timeout must be a non-negative integer");
      }
      timeout = number.longValue();
    }

    Object contextValue = map.lookup(CONTEXT_KEY, MISSING);
    IMapType<Object, Object> context =
        contextValue == MISSING ? EMPTY_CONTEXT : contextMap(contextValue);
    return new SynchronizeOptions(timeout, context);
  }

  private static HaraResult timeoutResult(
      Object value, long milliseconds, IMapType<Object, Object> context) {
    IMapType<Object, Object> enriched =
        assocContext(
            context,
            Keyword.create("result", "timeout"),
            milliseconds,
            Keyword.create("result", "cancellation-requested"),
            value instanceof IPromise);

    if (value instanceof IPromise promise) {
      try {
        promise.cancel();
        enriched =
            assocContext(enriched, Keyword.create("result", "cancelled"), Boolean.TRUE);
      } catch (Throwable cancellationError) {
        enriched =
            assocContext(
                enriched,
                Keyword.create("result", "cancelled"),
                Boolean.FALSE,
                Keyword.create("result", "cancellation-error"),
                errorMessage(cancellationError));
      }
    }

    return error(
        resultError("timeout", "Result synchronization timed out", milliseconds),
        enriched);
  }

  private static HaraResult timeoutUnsupportedResult(
      long milliseconds, IMapType<Object, Object> context) {
    return error(
        resultError(
            "timeout-unsupported",
            "Timed synchronization is unsupported for this dereferenceable value",
            milliseconds),
        assocContext(context, Keyword.create("result", "timeout"), milliseconds));
  }

  private static Ex.Info resultError(String code, String message, long milliseconds) {
    return new Ex.Info(
        message,
        hara.lang.data.Map.Standard.from(
            null,
            Keyword.create("code"),
            Keyword.create("result", code),
            Keyword.create("message"),
            message,
            Keyword.create("timeout"),
            milliseconds));
  }

  @SuppressWarnings({"rawtypes", "unchecked"})
  private static IMapType<Object, Object> assocContext(
      IMapType<Object, Object> context, Object... entries) {
    IMapType result = context;
    for (int index = 0; index < entries.length; index += 2) {
      result = (IMapType) result.assoc(entries[index], entries[index + 1]);
    }
    return (IMapType<Object, Object>) result;
  }

  private record SynchronizeOptions(Long timeout, IMapType<Object, Object> context) {}

'''
    text = insert_once(
        text,
        '''  public Keyword status() {
''',
        synchronization,
        "Java Result synchronization",
    )
    path.write_text(text)


def main() -> None:
    patch_rust_core()
    patch_rust_result()
    patch_java_context()
    patch_java_result()
    print("runtime synchronization patch applied")


if __name__ == "__main__":
    main()
