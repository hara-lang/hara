//! Runtime-owned semantic evidence for the opt-in live evaluator.
//!
//! The ordinary evaluator records nothing. While an observed [`EvalFiber`] is
//! actively executing, authoritative CPS and mutation seams enqueue owned
//! semantic evidence. The observed driver publishes at most one queued event
//! per host step; it never replays source or predicts evaluation order.
//!
//! Instrumented targets can disable evidence and environment capture between
//! safepoints. This keeps the compatibility observer unchanged while allowing
//! the shared instrumentation hub to avoid environment clones unless a matching
//! registration requested an environment-backed projection.

use super::super::*;
use crate::kernel::SpannedForm;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::core::fiber) enum EvalSemanticRule {
    FormReturn,
    ValueReturn,
    CallEnter,
    VarDefine,
    VarSet,
    FieldSet,
    ErrorRaise,
    ErrorCatch,
}

impl EvalSemanticRule {
    pub(super) const fn as_keyword(self) -> &'static str {
        match self {
            Self::FormReturn => "form/return",
            Self::ValueReturn => "value/return",
            Self::CallEnter => "call/enter",
            Self::VarDefine => "effect/var-define",
            Self::VarSet => "effect/var-set",
            Self::FieldSet => "effect/field-set",
            Self::ErrorRaise => "error/raise",
            Self::ErrorCatch => "error/catch",
        }
    }
}

#[derive(Clone)]
pub(in crate::core::fiber) enum EvalSemanticPayload {
    Result(Value),
    Call {
        name: String,
        arguments: Vec<Value>,
    },
    Effect {
        target: String,
        before: Option<Value>,
        after: Value,
    },
    Error {
        message: String,
        caught: bool,
    },
}

#[derive(Clone)]
pub(super) struct EvalSemanticBoundary {
    pub(super) sequence: usize,
    pub(super) rule: EvalSemanticRule,
    pub(super) form: Form,
    pub(super) payload: EvalSemanticPayload,
    pub(super) environment: HashMap<String, Value>,
}

struct EvalObservationContext {
    source_forms: Option<Rc<Vec<SpannedForm>>>,
    sequence: usize,
    current: Option<EvalSemanticBoundary>,
    pending: VecDeque<EvalSemanticBoundary>,
    capture_events: bool,
    capture_environment: bool,
    environment_clones: u64,
}

thread_local! {
    static OBSERVED_CONTEXTS: RefCell<HashMap<usize, Rc<RefCell<EvalObservationContext>>>> =
        RefCell::new(HashMap::new());
    static ACTIVE_CONTEXTS: RefCell<Vec<Rc<RefCell<EvalObservationContext>>>> =
        RefCell::new(Vec::new());
}

fn environment_key(environment: &Rc<RefCell<HashMap<String, Value>>>) -> usize {
    Rc::as_ptr(environment) as usize
}

pub(super) fn register_context(
    environment: &Rc<RefCell<HashMap<String, Value>>>,
    source_forms: Option<Rc<Vec<SpannedForm>>>,
    capture_events: bool,
    capture_environment: bool,
) {
    let context = Rc::new(RefCell::new(EvalObservationContext {
        source_forms,
        sequence: 0,
        current: None,
        pending: VecDeque::new(),
        capture_events,
        capture_environment,
        environment_clones: 0,
    }));
    OBSERVED_CONTEXTS.with(|contexts| {
        contexts
            .borrow_mut()
            .insert(environment_key(environment), context);
    });
}

pub(super) fn configure_capture(
    environment: &Rc<RefCell<HashMap<String, Value>>>,
    capture_events: bool,
    capture_environment: bool,
) {
    if let Some(context) = context_for(environment) {
        let mut context = context.borrow_mut();
        context.capture_events = capture_events;
        context.capture_environment = capture_environment;
        if !capture_events {
            context.current = None;
            context.pending.clear();
        }
    }
}

pub(super) fn environment_clone_count(environment: &Rc<RefCell<HashMap<String, Value>>>) -> u64 {
    context_for(environment)
        .map(|context| context.borrow().environment_clones)
        .unwrap_or(0)
}

pub(super) fn remove_context(environment: &Rc<RefCell<HashMap<String, Value>>>) {
    OBSERVED_CONTEXTS.with(|contexts| {
        contexts.borrow_mut().remove(&environment_key(environment));
    });
}

fn context_for(
    environment: &Rc<RefCell<HashMap<String, Value>>>,
) -> Option<Rc<RefCell<EvalObservationContext>>> {
    OBSERVED_CONTEXTS.with(|contexts| {
        contexts
            .borrow()
            .get(&environment_key(environment))
            .cloned()
    })
}

struct ActiveContextGuard;

impl Drop for ActiveContextGuard {
    fn drop(&mut self) {
        ACTIVE_CONTEXTS.with(|contexts| {
            contexts.borrow_mut().pop();
        });
    }
}

pub(super) fn with_active_context<T>(
    environment: &Rc<RefCell<HashMap<String, Value>>>,
    operation: impl FnOnce() -> T,
) -> T {
    let Some(context) = context_for(environment) else {
        return operation();
    };
    ACTIVE_CONTEXTS.with(|contexts| contexts.borrow_mut().push(context));
    let _guard = ActiveContextGuard;
    operation()
}

fn active_context() -> Option<Rc<RefCell<EvalObservationContext>>> {
    ACTIVE_CONTEXTS.with(|contexts| contexts.borrow().last().cloned())
}

fn capture_enabled() -> bool {
    active_context().is_some_and(|context| context.borrow().capture_events)
}

fn enqueue(
    rule: EvalSemanticRule,
    form: &Form,
    payload: EvalSemanticPayload,
    environment: &Rc<RefCell<HashMap<String, Value>>>,
) {
    let Some(context) = active_context() else {
        return;
    };
    let capture_environment = {
        let context = context.borrow();
        if !context.capture_events {
            return;
        }
        context.capture_environment
    };
    let captured_environment = if capture_environment {
        let environment = environment.borrow().clone();
        let mut captured = context.borrow_mut();
        captured.environment_clones = captured.environment_clones.saturating_add(1);
        drop(captured);
        environment
    } else {
        HashMap::new()
    };
    let mut context = context.borrow_mut();
    context.sequence = context.sequence.saturating_add(1);
    let sequence = context.sequence;
    context.pending.push_back(EvalSemanticBoundary {
        sequence,
        rule,
        form: form.clone(),
        payload,
        environment: captured_environment,
    });
}

pub(in crate::core::fiber) fn record_boundary(
    rule: EvalSemanticRule,
    form: &Form,
    result: &Value,
    environment: &Rc<RefCell<HashMap<String, Value>>>,
) {
    if !capture_enabled() {
        return;
    }
    enqueue(
        rule,
        form,
        EvalSemanticPayload::Result(result.clone()),
        environment,
    );
}

pub(in crate::core::fiber) fn record_call(
    form: &Form,
    name: impl Into<String>,
    arguments: &[Value],
    environment: &Rc<RefCell<HashMap<String, Value>>>,
) {
    if !capture_enabled() {
        return;
    }
    enqueue(
        EvalSemanticRule::CallEnter,
        form,
        EvalSemanticPayload::Call {
            name: name.into(),
            arguments: arguments.to_vec(),
        },
        environment,
    );
}

pub(in crate::core::fiber) fn record_effect(
    rule: EvalSemanticRule,
    form: &Form,
    target: impl Into<String>,
    before: Option<Value>,
    after: Value,
    environment: &Rc<RefCell<HashMap<String, Value>>>,
) {
    if !capture_enabled() {
        return;
    }
    debug_assert!(matches!(
        rule,
        EvalSemanticRule::VarDefine | EvalSemanticRule::VarSet | EvalSemanticRule::FieldSet
    ));
    enqueue(
        rule,
        form,
        EvalSemanticPayload::Effect {
            target: target.into(),
            before,
            after,
        },
        environment,
    );
}

pub(in crate::core::fiber) fn record_error(
    rule: EvalSemanticRule,
    form: &Form,
    message: impl Into<String>,
    caught: bool,
    environment: &Rc<RefCell<HashMap<String, Value>>>,
) {
    if !capture_enabled() {
        return;
    }
    debug_assert!(matches!(
        rule,
        EvalSemanticRule::ErrorRaise | EvalSemanticRule::ErrorCatch
    ));
    let message = message.into();
    let duplicate = active_context().is_some_and(|context| {
        context.borrow().pending.back().is_some_and(|boundary| {
            matches!(
                &boundary.payload,
                EvalSemanticPayload::Error {
                    message: prior,
                    caught: prior_caught,
                } if prior == &message && *prior_caught == caught
            )
        })
    });
    if duplicate {
        return;
    }
    enqueue(
        rule,
        form,
        EvalSemanticPayload::Error { message, caught },
        environment,
    );
}

/// Publishes one queued semantic event without executing another continuation.
pub(super) fn advance_pending(environment: &Rc<RefCell<HashMap<String, Value>>>) -> bool {
    let Some(context) = context_for(environment) else {
        return false;
    };
    let mut context = context.borrow_mut();
    let Some(next) = context.pending.pop_front() else {
        return false;
    };
    context.current = Some(next);
    true
}

pub(super) fn pending_count(environment: &Rc<RefCell<HashMap<String, Value>>>) -> usize {
    context_for(environment)
        .map(|context| context.borrow().pending.len())
        .unwrap_or(0)
}

pub(super) fn current_boundary(
    environment: &Rc<RefCell<HashMap<String, Value>>>,
) -> Option<EvalSemanticBoundary> {
    let context = context_for(environment)?;
    context.borrow().current.clone()
}

pub(super) fn source_forms(
    environment: &Rc<RefCell<HashMap<String, Value>>>,
) -> Option<Rc<Vec<SpannedForm>>> {
    let context = context_for(environment)?;
    context.borrow().source_forms.clone()
}
