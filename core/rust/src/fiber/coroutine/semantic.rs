//! Runtime-owned semantic evidence for the opt-in live evaluator.
//!
//! The ordinary evaluator records nothing. While an observed [`EvalFiber`] is
//! actively executing, the existing CPS continuation producers call
//! [`record_boundary`] with the form and value they actually completed. The
//! context keeps owned clones for later bounded projection; it never executes
//! forms or predicts evaluation order.

use super::super::*;
use crate::kernel::SpannedForm;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EvalSemanticRule {
    FormReturn,
    ValueReturn,
}

impl EvalSemanticRule {
    pub(super) const fn as_keyword(self) -> &'static str {
        match self {
            Self::FormReturn => "form/return",
            Self::ValueReturn => "value/return",
        }
    }
}

#[derive(Clone)]
pub(super) struct EvalSemanticBoundary {
    pub(super) sequence: usize,
    pub(super) rule: EvalSemanticRule,
    pub(super) form: Form,
    pub(super) result: Value,
    pub(super) environment: HashMap<String, Value>,
}

struct EvalObservationContext {
    source_forms: Option<Rc<Vec<SpannedForm>>>,
    sequence: usize,
    current: Option<EvalSemanticBoundary>,
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
) {
    let context = Rc::new(RefCell::new(EvalObservationContext {
        source_forms,
        sequence: 0,
        current: None,
    }));
    OBSERVED_CONTEXTS.with(|contexts| {
        contexts
            .borrow_mut()
            .insert(environment_key(environment), context);
    });
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

pub(super) fn record_boundary(
    rule: EvalSemanticRule,
    form: &Form,
    result: &Value,
    environment: &Rc<RefCell<HashMap<String, Value>>>,
) {
    let context = ACTIVE_CONTEXTS.with(|contexts| contexts.borrow().last().cloned());
    let Some(context) = context else {
        return;
    };
    let environment = environment.borrow().clone();
    let mut context = context.borrow_mut();
    context.sequence = context.sequence.saturating_add(1);
    context.current = Some(EvalSemanticBoundary {
        sequence: context.sequence,
        rule,
        form: form.clone(),
        result: result.clone(),
        environment,
    });
}

pub(super) fn current_boundary(
    environment: &Rc<RefCell<HashMap<String, Value>>>,
) -> Option<EvalSemanticBoundary> {
    let context = context_for(environment)?;
    let boundary = context.borrow().current.clone();
    boundary
}

pub(super) fn source_forms(
    environment: &Rc<RefCell<HashMap<String, Value>>>,
) -> Option<Rc<Vec<SpannedForm>>> {
    let context = context_for(environment)?;
    let source_forms = context.borrow().source_forms.clone();
    source_forms
}
