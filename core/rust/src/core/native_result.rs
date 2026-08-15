use super::{map_entries, thrown_error, ExceptionInfo, Value};

fn native_equal(left: &Value, right: &Value) -> bool {
    left == right
}
use crate::lang::data::{Keyword, Map as PMap};
use crate::lang::hash::{self as jh, JavaHash};
use crate::lang::protocol::HashType;
use std::cmp::Ordering;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResultStatus {
    Success,
    Error,
}

impl ResultStatus {
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResultValue {
    pub status: ResultStatus,
    pub data: Value,
    pub error: Option<Rc<ExceptionInfo>>,
    pub context: Value,
}

impl ResultValue {
    pub fn success(data: Value, context: Value) -> Result<Self, String> {
        Ok(Self {
            status: ResultStatus::Success,
            data,
            error: None,
            context: validate_context(context)?,
        })
    }

    pub fn error(error: Value, context: Value) -> Result<Self, String> {
        Ok(Self {
            status: ResultStatus::Error,
            data: Value::Nil,
            error: Some(normalize_error(error)),
            context: validate_context(context)?,
        })
    }

    pub fn status_value(&self) -> Value {
        Value::Keyword(Keyword::from(self.status.keyword()))
    }

    pub fn error_value(&self) -> Value {
        self.error
            .as_ref()
            .map(|error| Value::ExceptionInfo(error.clone()))
            .unwrap_or(Value::Nil)
    }

    pub fn is_success(&self) -> bool {
        self.status == ResultStatus::Success
    }

    pub fn is_error(&self) -> bool {
        self.status == ResultStatus::Error
    }

    pub fn with_context(&self, additional: Value) -> Result<Self, String> {
        let additional = validate_context(additional)?;
        let mut merged = PMap::new();
        for (key, value) in map_entries(&self.context)
            .expect("validated Result context")
            .into_iter()
            .chain(
                map_entries(&additional)
                    .expect("validated additional Result context")
                    .into_iter(),
            )
        {
            merged = merged.assoc_value(key, value);
        }
        let mut updated = self.clone();
        updated.context = Value::Map(merged);
        Ok(updated)
    }

    pub(crate) fn deref_value(&self) -> Result<Value, String> {
        match self.status {
            ResultStatus::Success => Ok(self.data.clone()),
            ResultStatus::Error => self
                .error
                .as_ref()
                .map(|error| Err(thrown_error(Value::ExceptionInfo(error.clone()))))
                .unwrap_or_else(|| Err("invalid Result/error without a native Error".into())),
        }
    }

    pub fn display(&self) -> String {
        format!(
            "#hara/Result[{} {} {} {}]",
            self.status_value().display(),
            self.data.display(),
            self.error_value().display(),
            self.context.display()
        )
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.status
            .cmp(&other.status)
            .then_with(|| self.data.cmp(&other.data))
            .then_with(|| compare_error(self.error.as_deref(), other.error.as_deref()))
    }

    pub fn java_hash(&self, hash_type: HashType) -> i64 {
        jh::compose_ordered(
            "RESULT",
            [
                match self.status {
                    ResultStatus::Success => 1,
                    ResultStatus::Error => 2,
                },
                self.data.java_hash(hash_type),
                self.error
                    .as_deref()
                    .map_or(0, |error| error_hash(error, hash_type)),
            ],
        )
    }
}

impl PartialEq for ResultValue {
    fn eq(&self, other: &Self) -> bool {
        self.status == other.status
            && native_equal(&self.data, &other.data)
            && error_equal(self.error.as_deref(), other.error.as_deref())
    }
}

impl Eq for ResultValue {}

fn validate_context(context: Value) -> Result<Value, String> {
    map_entries(&context)
        .is_some()
        .then_some(context)
        .ok_or_else(|| "Result context must be a map".into())
}

fn normalize_error(value: Value) -> Rc<ExceptionInfo> {
    match value {
        Value::ExceptionInfo(error) => error,
        value => {
            let message = value.display();
            Rc::new(ExceptionInfo {
                message,
                data: Box::new(Value::Map(PMap::from_iter([(
                    Value::Keyword(Keyword::from("error/value")),
                    value,
                )]))),
                cause: None,
            })
        }
    }
}

fn error_equal(left: Option<&ExceptionInfo>, right: Option<&ExceptionInfo>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => {
            left.message == right.message
                && native_equal(&left.data, &right.data)
                && match (&left.cause, &right.cause) {
                    (None, None) => true,
                    (Some(left), Some(right)) => native_equal(left, right),
                    _ => false,
                }
        }
        _ => false,
    }
}

fn compare_error(left: Option<&ExceptionInfo>, right: Option<&ExceptionInfo>) -> Ordering {
    match (left, right) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(left), Some(right)) => left
            .message
            .cmp(&right.message)
            .then_with(|| left.data.cmp(&right.data))
            .then_with(|| left.cause.cmp(&right.cause)),
    }
}

fn error_hash(error: &ExceptionInfo, hash_type: HashType) -> i64 {
    jh::compose_ordered(
        "RESULT_ERROR",
        [
            jh::java_string_hash("hara/Error") as i64,
            jh::java_string_hash(&error.message) as i64,
            error.data.java_hash(hash_type),
            error
                .cause
                .as_deref()
                .map_or(0, |cause| cause.java_hash(hash_type)),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(key: &str, value: Value) -> Value {
        Value::Map(PMap::from_iter([(
            Value::Keyword(Keyword::from(key)),
            value,
        )]))
    }

    #[test]
    fn native_result_equality_and_hash_ignore_context() {
        let left = ResultValue::success(
            Value::Number(42),
            context("source", Value::String("left".into())),
        )
        .expect("left Result");
        let right = ResultValue::success(
            Value::Number(42),
            context("source", Value::String("right".into())),
        )
        .expect("right Result");
        assert_eq!(left, right);
        assert_eq!(
            left.java_hash(crate::lang::hash::DEFAULT_HASH),
            right.java_hash(crate::lang::hash::DEFAULT_HASH)
        );
        assert_eq!(
            left.deref_value().expect("success deref"),
            Value::Number(42)
        );
    }

    #[test]
    fn native_result_context_merge_uses_supplied_keys() {
        let result = ResultValue::success(
            Value::Number(7),
            Value::Map(PMap::from_iter([
                (
                    Value::Keyword(Keyword::from("source")),
                    Value::String("left".into()),
                ),
                (Value::Keyword(Keyword::from("kept")), Value::Bool(true)),
            ])),
        )
        .expect("Result");
        let updated = result
            .with_context(Value::Map(PMap::from_iter([
                (
                    Value::Keyword(Keyword::from("source")),
                    Value::String("right".into()),
                ),
                (Value::Keyword(Keyword::from("added")), Value::Number(1)),
            ])))
            .expect("merged Result");
        let source =
            super::super::map_value(&updated.context, &Value::Keyword(Keyword::from("source")))
                .expect("source context");
        assert!(matches!(source, Value::String(value) if value.as_str() == "right"));
        assert_eq!(result, updated);
    }

    #[test]
    fn native_result_error_preserves_native_error_and_deref_throws() {
        let error = Rc::new(ExceptionInfo {
            message: "boom".into(),
            data: Box::new(context("code", Value::Keyword(Keyword::from("boom")))),
            cause: None,
        });
        let result =
            ResultValue::error(Value::ExceptionInfo(error.clone()), Value::Map(PMap::new()))
                .expect("error Result");
        assert!(result.is_error());
        let preserved = match result.error_value() {
            Value::ExceptionInfo(preserved) => preserved,
            other => panic!("expected native Error, got {}", other.display()),
        };
        assert_eq!(preserved.message, error.message);
        assert_eq!(preserved.data.display(), error.data.display());
        assert!(result.deref_value().is_err());
        assert!(result.display().contains("#hara/Result[:error"));
    }
}
