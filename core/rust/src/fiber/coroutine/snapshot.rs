//! Portable, bounded projections of the live production evaluator fiber.
//!
//! These snapshots observe the retained CPS continuation introduced by the
//! live-fiber seam. They contain only owned scalar and string data: executable
//! values, promises, continuations, mutable cells, and host handles remain
//! owned by [`EvalFiber`].

use super::super::*;
use crate::lang::data::{OrderedMap, Vector};

pub const INTERPRETER_LIVE_SNAPSHOT_SCHEMA: &str = "hal.interpreter-live-snapshot/0-alpha";
pub const INTERPRETER_LIVE_BOUNDARY_SCHEMA: &str = "hal.interpreter-live-boundary/0-alpha";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvalObservationLimits {
    pub bindings: usize,
    pub display_chars: usize,
}

impl Default for EvalObservationLimits {
    fn default() -> Self {
        Self {
            bindings: 64,
            display_chars: 160,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvalObservationStatus {
    Running,
    Paused,
    Suspended,
    Returned,
    Failed,
    Cancelled,
}

impl EvalObservationStatus {
    pub const fn as_keyword(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Suspended => "suspended",
            Self::Returned => "returned",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Returned | Self::Failed | Self::Cancelled)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvalObservedBoundaryKind {
    Continue,
    Suspend,
    Resume,
    Return,
    Fail,
    Noop,
}

impl EvalObservedBoundaryKind {
    pub const fn as_keyword(self) -> &'static str {
        match self {
            Self::Continue => "evaluation/continue",
            Self::Suspend => "evaluation/suspend",
            Self::Resume => "evaluation/resume",
            Self::Return => "evaluation/return",
            Self::Fail => "evaluation/fail",
            Self::Noop => "evaluation/noop",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalValueSnapshot {
    pub kind: &'static str,
    pub display: String,
    pub truncated: bool,
    pub redacted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalBindingSnapshot {
    pub name: String,
    pub value: EvalValueSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalErrorSnapshot {
    pub message: String,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalPendingSnapshot {
    pub state: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalObservationSnapshot {
    pub schema: &'static str,
    pub source_id: String,
    pub status: EvalObservationStatus,
    pub paused: bool,
    pub binding_count: usize,
    pub bindings: Vec<EvalBindingSnapshot>,
    pub bindings_omitted: usize,
    pub pending: Option<EvalPendingSnapshot>,
    pub result: Option<EvalValueSnapshot>,
    pub error: Option<EvalErrorSnapshot>,
}

impl EvalObservationSnapshot {
    pub fn to_value(&self) -> Value {
        object([
            ("schema", string(self.schema)),
            ("sourceId", string(&self.source_id)),
            ("status", string(self.status.as_keyword())),
            ("paused", Value::Bool(self.paused)),
            ("bindingCount", integer(self.binding_count)),
            ("bindings", vector(self.bindings.iter().map(binding_value))),
            ("bindingsOmitted", integer(self.bindings_omitted)),
            (
                "pending",
                optional_value(self.pending.as_ref().map(pending_value)),
            ),
            (
                "result",
                optional_value(self.result.as_ref().map(value_snapshot_value)),
            ),
            (
                "error",
                optional_value(self.error.as_ref().map(error_value)),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalObservedBoundary {
    pub schema: &'static str,
    pub kind: EvalObservedBoundaryKind,
    pub before: EvalObservationSnapshot,
    pub after: EvalObservationSnapshot,
}

impl EvalObservedBoundary {
    pub fn to_value(&self) -> Value {
        object([
            ("schema", string(self.schema)),
            ("kind", string(self.kind.as_keyword())),
            ("before", self.before.to_value()),
            ("after", self.after.to_value()),
        ])
    }
}

impl EvalFiber {
    /// Returns a bounded JSON-safe document with default observation limits.
    pub fn snapshot_observed_value(&self, source_id: impl Into<String>) -> Value {
        self.snapshot_observed(source_id, EvalObservationLimits::default())
            .to_value()
    }

    /// Returns a bounded JSON-safe document without exposing runtime handles.
    pub fn snapshot_observed_value_with_limits(
        &self,
        source_id: impl Into<String>,
        binding_limit: usize,
        display_chars: usize,
    ) -> Value {
        self.snapshot_observed(
            source_id,
            EvalObservationLimits {
                bindings: binding_limit,
                display_chars,
            },
        )
        .to_value()
    }

    /// Executes one production continuation and returns before/after evidence.
    pub fn step_observed_value(&mut self, source_id: impl Into<String>) -> Value {
        self.step_observed_snapshot(source_id, EvalObservationLimits::default())
            .to_value()
    }

    /// Executes one production continuation with caller-selected evidence bounds.
    pub fn step_observed_value_with_limits(
        &mut self,
        source_id: impl Into<String>,
        binding_limit: usize,
        display_chars: usize,
    ) -> Value {
        self.step_observed_snapshot(
            source_id,
            EvalObservationLimits {
                bindings: binding_limit,
                display_chars,
            },
        )
        .to_value()
    }

    /// Applies one real promise settlement and returns before/after evidence.
    pub fn resume_observed_value(
        &mut self,
        state: PromiseState,
        source_id: impl Into<String>,
    ) -> Value {
        self.resume_observed_snapshot(state, source_id, EvalObservationLimits::default())
            .to_value()
    }

    /// Applies one promise settlement with caller-selected evidence bounds.
    pub fn resume_observed_value_with_limits(
        &mut self,
        state: PromiseState,
        source_id: impl Into<String>,
        binding_limit: usize,
        display_chars: usize,
    ) -> Value {
        self.resume_observed_snapshot(
            state,
            source_id,
            EvalObservationLimits {
                bindings: binding_limit,
                display_chars,
            },
        )
        .to_value()
    }

    /// Projects the current evaluator state without exposing executable values.
    pub(crate) fn snapshot_observed(
        &self,
        source_id: impl Into<String>,
        limits: EvalObservationLimits,
    ) -> EvalObservationSnapshot {
        let source_id = source_id.into();
        let status = observation_status(self);
        let mut bindings = self
            .env
            .borrow()
            .iter()
            .map(|(name, value)| EvalBindingSnapshot {
                name: name.clone(),
                value: value_snapshot(value, limits.display_chars),
            })
            .collect::<Vec<_>>();
        bindings.sort_by(|left, right| left.name.cmp(&right.name));
        let binding_count = bindings.len();
        bindings.truncate(limits.bindings);
        let bindings_omitted = binding_count.saturating_sub(bindings.len());
        let pending = self.pending.as_ref().map(|promise| EvalPendingSnapshot {
            state: promise_state_keyword(&promise.state()),
        });
        let result = match &self.state {
            EvalFiberState::Completed(value) => Some(value_snapshot(value, limits.display_chars)),
            _ => None,
        };
        let error = match &self.state {
            EvalFiberState::Failed(message) => {
                let (message, truncated) = bounded_text(message, limits.display_chars);
                Some(EvalErrorSnapshot { message, truncated })
            }
            _ => None,
        };

        EvalObservationSnapshot {
            schema: INTERPRETER_LIVE_SNAPSHOT_SCHEMA,
            source_id,
            status,
            paused: self.observed_paused(),
            binding_count,
            bindings,
            bindings_omitted,
            pending,
            result,
            error,
        }
    }

    /// Executes one live evaluator boundary and returns bounded before/after state.
    pub(crate) fn step_observed_snapshot(
        &mut self,
        source_id: impl Into<String>,
        limits: EvalObservationLimits,
    ) -> EvalObservedBoundary {
        let source_id = source_id.into();
        let before = self.snapshot_observed(source_id.clone(), limits);
        self.step_observed();
        let after = self.snapshot_observed(source_id, limits);
        EvalObservedBoundary {
            schema: INTERPRETER_LIVE_BOUNDARY_SCHEMA,
            kind: boundary_kind(&before, &after, false),
            before,
            after,
        }
    }

    /// Applies one promise settlement and returns the resulting live boundary.
    pub(crate) fn resume_observed_snapshot(
        &mut self,
        state: PromiseState,
        source_id: impl Into<String>,
        limits: EvalObservationLimits,
    ) -> EvalObservedBoundary {
        let source_id = source_id.into();
        let before = self.snapshot_observed(source_id.clone(), limits);
        self.resume_observed(state);
        let after = self.snapshot_observed(source_id, limits);
        EvalObservedBoundary {
            schema: INTERPRETER_LIVE_BOUNDARY_SCHEMA,
            kind: boundary_kind(&before, &after, true),
            before,
            after,
        }
    }
}

fn observation_status(fiber: &EvalFiber) -> EvalObservationStatus {
    match &fiber.state {
        EvalFiberState::Running if fiber.observed_paused() => EvalObservationStatus::Paused,
        EvalFiberState::Running => EvalObservationStatus::Running,
        EvalFiberState::Suspended => EvalObservationStatus::Suspended,
        EvalFiberState::Completed(_) => EvalObservationStatus::Returned,
        EvalFiberState::Failed(_) => EvalObservationStatus::Failed,
        EvalFiberState::Cancelled => EvalObservationStatus::Cancelled,
    }
}

fn boundary_kind(
    before: &EvalObservationSnapshot,
    after: &EvalObservationSnapshot,
    resumed: bool,
) -> EvalObservedBoundaryKind {
    match after.status {
        EvalObservationStatus::Suspended => EvalObservedBoundaryKind::Suspend,
        EvalObservationStatus::Returned => EvalObservedBoundaryKind::Return,
        EvalObservationStatus::Failed => EvalObservedBoundaryKind::Fail,
        EvalObservationStatus::Cancelled => EvalObservedBoundaryKind::Noop,
        EvalObservationStatus::Running | EvalObservationStatus::Paused if resumed => {
            EvalObservedBoundaryKind::Resume
        }
        EvalObservationStatus::Running | EvalObservationStatus::Paused => {
            if before.status.is_terminal() {
                EvalObservedBoundaryKind::Noop
            } else {
                EvalObservedBoundaryKind::Continue
            }
        }
    }
}

fn value_snapshot(value: &Value, display_chars: usize) -> EvalValueSnapshot {
    let kind = value_kind(value);
    let (display, redacted) = safe_display(value);
    let (display, truncated) = bounded_text(&display, display_chars);
    EvalValueSnapshot {
        kind,
        display,
        truncated,
        redacted,
    }
}

fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Number(_) | Value::BigInteger(_) => "integer",
        Value::Float(_) => "float",
        Value::Decimal(_) => "decimal",
        Value::Character(_) => "character",
        Value::Bool(_) => "boolean",
        Value::String(_) => "string",
        Value::Keyword(_) => "keyword",
        Value::Symbol(_) => "symbol",
        Value::Bytes(_) => "bytes",
        Value::Promise(_) => "promise",
        Value::Function(_) => "function",
        Value::Var(_) => "var",
        Value::Extension(_) => "extension",
        Value::Coroutine(_) => "coroutine",
        Value::Iterator(_) => "iterator",
        Value::Nil => "nil",
        _ => "value",
    }
}

fn safe_display(value: &Value) -> (String, bool) {
    match value {
        Value::Promise(promise) => (
            format!("<promise {}>", promise_state_keyword(&promise.state())),
            true,
        ),
        Value::Function(_) => ("<function>".into(), true),
        Value::Coroutine(_) => ("<coroutine>".into(), true),
        Value::Iterator(_) => ("<iterator>".into(), true),
        Value::Extension(extension) => (
            format!("<extension {}/{}>", extension.provider, extension.type_name),
            true,
        ),
        Value::ByteBuffer(_) => ("<byte-buffer>".into(), true),
        Value::Array(_) => ("<array>".into(), true),
        Value::Object(_) => ("<object>".into(), true),
        Value::MutableCollection(_) => ("<mutable-collection>".into(), true),
        Value::Mutable(_) => ("<mutable>".into(), true),
        _ => (value.display(), false),
    }
}

fn promise_state_keyword(state: &PromiseState) -> &'static str {
    match state {
        PromiseState::Pending => "pending",
        PromiseState::Fulfilled(_) => "fulfilled",
        PromiseState::Rejected(_) => "rejected",
    }
}

fn bounded_text(value: &str, limit: usize) -> (String, bool) {
    let mut characters = value.chars();
    let mut retained = characters.by_ref().take(limit).collect::<String>();
    let truncated = characters.next().is_some();
    if truncated {
        retained.push('…');
    }
    (retained, truncated)
}

fn binding_value(binding: &EvalBindingSnapshot) -> Value {
    object([
        ("name", string(&binding.name)),
        ("value", value_snapshot_value(&binding.value)),
    ])
}

fn value_snapshot_value(value: &EvalValueSnapshot) -> Value {
    object([
        ("kind", string(value.kind)),
        ("display", string(&value.display)),
        ("truncated", Value::Bool(value.truncated)),
        ("redacted", Value::Bool(value.redacted)),
    ])
}

fn pending_value(pending: &EvalPendingSnapshot) -> Value {
    object([("state", string(pending.state))])
}

fn error_value(error: &EvalErrorSnapshot) -> Value {
    object([
        ("message", string(&error.message)),
        ("truncated", Value::Bool(error.truncated)),
    ])
}

fn object<const N: usize>(fields: [(&str, Value); N]) -> Value {
    Value::OrderedMap(Box::new(OrderedMap::from_iter(
        fields
            .into_iter()
            .map(|(key, value)| (Value::String(key.into()), value)),
    )))
}

fn vector(values: impl IntoIterator<Item = Value>) -> Value {
    Value::Vector(Vector::from_iter(values))
}

fn string(value: impl Into<String>) -> Value {
    Value::String(value.into())
}

fn integer(value: usize) -> Value {
    Value::Number(i64::try_from(value).unwrap_or(i64::MAX))
}

fn optional_value(value: Option<Value>) -> Value {
    value.unwrap_or(Value::Nil)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshots_sort_bound_and_redact_environment_bindings() {
        let mut environment = HashMap::new();
        environment.insert("zeta".into(), Value::Number(3));
        environment.insert("alpha".into(), Value::String("abcdefgh".into()));
        environment.insert(
            "extension".into(),
            Value::Extension(ExtensionValue {
                provider: "demo".into(),
                type_name: "socket".into(),
                handle: 999,
            }),
        );
        let fiber = EvalFiber::start_observed("nil", environment).unwrap();
        let snapshot = fiber.snapshot_observed(
            "fixture/snapshot.hal",
            EvalObservationLimits {
                bindings: 2,
                display_chars: 4,
            },
        );

        assert_eq!(snapshot.status, EvalObservationStatus::Paused);
        assert_eq!(snapshot.binding_count, 3);
        assert_eq!(snapshot.bindings_omitted, 1);
        assert_eq!(snapshot.bindings[0].name, "alpha");
        assert_eq!(snapshot.bindings[1].name, "extension");
        assert!(snapshot.bindings[0].value.truncated);
        assert!(snapshot.bindings[1].value.redacted);
        assert!(!snapshot.bindings[1].value.display.contains("999"));
        let json = crate::json::write(&snapshot.to_value()).unwrap();
        assert!(json.contains("hal.interpreter-live-snapshot/0-alpha"));
        assert!(!json.contains("999"));
    }

    #[test]
    fn live_boundaries_project_before_after_state_and_terminal_result() {
        let limits = EvalObservationLimits::default();
        let mut fiber = EvalFiber::start_observed("(+ 19 23)", HashMap::new()).unwrap();
        let first = fiber.step_observed_snapshot("fixture/add.hal", limits);
        assert_eq!(first.kind, EvalObservedBoundaryKind::Continue);
        assert_eq!(first.before.status, EvalObservationStatus::Paused);
        assert_eq!(first.after.status, EvalObservationStatus::Paused);

        let returned = fiber.step_observed_snapshot("fixture/add.hal", limits);
        assert_eq!(returned.kind, EvalObservedBoundaryKind::Return);
        assert_eq!(returned.after.status, EvalObservationStatus::Returned);
        assert_eq!(
            returned
                .after
                .result
                .as_ref()
                .map(|value| value.display.as_str()),
            Some("42")
        );
        let json = crate::json::write(&returned.to_value()).unwrap();
        assert!(json.contains("evaluation/return"));
        assert!(json.contains("\"display\":\"42\""));
    }

    #[test]
    fn promise_boundaries_expose_state_without_identity_or_automatic_drain() {
        let promise = Promise::new();
        let mut environment = HashMap::new();
        environment.insert("pending-value".into(), Value::Promise(promise.clone()));
        let limits = EvalObservationLimits::default();
        let mut fiber =
            EvalFiber::start_observed("(Coroutine/await pending-value)", environment).unwrap();

        while matches!(fiber.state(), EvalFiberState::Running) {
            fiber.step_observed_snapshot("fixture/await.hal", limits);
        }
        let suspended = fiber.snapshot_observed("fixture/await.hal", limits);
        assert_eq!(suspended.status, EvalObservationStatus::Suspended);
        assert_eq!(
            suspended.pending.as_ref().map(|pending| pending.state),
            Some("pending")
        );

        promise.resolve(Value::Number(42));
        let resumed = fiber.resume_observed_snapshot(promise.state(), "fixture/await.hal", limits);
        assert_eq!(resumed.kind, EvalObservedBoundaryKind::Resume);
        assert_eq!(resumed.after.status, EvalObservationStatus::Paused);
        assert!(resumed.after.pending.is_none());
        let json = crate::json::write(&resumed.to_value()).unwrap();
        assert!(!json.contains("identity"));
    }
}
