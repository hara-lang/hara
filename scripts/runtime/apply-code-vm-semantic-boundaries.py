#!/usr/bin/env python3
"""Apply the focused #403 live semantic boundary slice."""

from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    source = target.read_text()
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one replacement, found {count}")
    target.write_text(source.replace(old, new, 1))


replace_once(
    "core/rust/src/fiber.rs",
    '''fn forms_cps(
    forms: Rc<Vec<Form>>,
    i: usize,
    last: Value,
    env: Rc<RefCell<HashMap<String, Value>>>,
    k: Cont,
) -> Step {
    if i == forms.len() || matches!(last, Value::Recur(_)) {
        return k(Ok(last));
    }
    let next = forms.clone();
    let e = env.clone();
    one(
        forms[i].clone(),
        env,
        Box::new(move |r| match r {
            Ok(v) => Step::Continue(Box::new(move || forms_cps(next, i + 1, v, e, k))),
            Err(x) => k(Err(x)),
        }),
    )
}
fn values_cps(
    forms: Rc<Vec<Form>>,
    i: usize,
    values: Vec<Value>,
    env: Rc<RefCell<HashMap<String, Value>>>,
    k: Box<dyn FnOnce(Result<Vec<Value>, String>) -> Step>,
) -> Step {
    if i == forms.len() {
        return k(Ok(values));
    }
    let next = forms.clone();
    let e = env.clone();
    one(
        forms[i].clone(),
        env,
        Box::new(move |r| match r {
            Ok(v) => {
                let mut values = values;
                values.push(v);
                Step::Continue(Box::new(move || values_cps(next, i + 1, values, e, k)))
            }
            Err(x) => k(Err(x)),
        }),
    )
}
''',
    '''fn forms_cps(
    forms: Rc<Vec<Form>>,
    i: usize,
    last: Value,
    env: Rc<RefCell<HashMap<String, Value>>>,
    k: Cont,
) -> Step {
    if i == forms.len() || matches!(last, Value::Recur(_)) {
        return k(Ok(last));
    }
    let next = forms.clone();
    let e = env.clone();
    let form = forms[i].clone();
    let boundary_form = form.clone();
    let boundary_env = env.clone();
    one(
        form,
        env,
        Box::new(move |r| match r {
            Ok(v) => {
                coroutine::semantic::record_boundary(
                    coroutine::semantic::EvalSemanticRule::FormReturn,
                    &boundary_form,
                    &v,
                    &boundary_env,
                );
                Step::Continue(Box::new(move || forms_cps(next, i + 1, v, e, k)))
            }
            Err(x) => k(Err(x)),
        }),
    )
}
fn values_cps(
    forms: Rc<Vec<Form>>,
    i: usize,
    values: Vec<Value>,
    env: Rc<RefCell<HashMap<String, Value>>>,
    k: Box<dyn FnOnce(Result<Vec<Value>, String>) -> Step>,
) -> Step {
    if i == forms.len() {
        return k(Ok(values));
    }
    let next = forms.clone();
    let e = env.clone();
    let form = forms[i].clone();
    let boundary_form = form.clone();
    let boundary_env = env.clone();
    one(
        form,
        env,
        Box::new(move |r| match r {
            Ok(v) => {
                coroutine::semantic::record_boundary(
                    coroutine::semantic::EvalSemanticRule::ValueReturn,
                    &boundary_form,
                    &v,
                    &boundary_env,
                );
                let mut values = values;
                values.push(v);
                Step::Continue(Box::new(move || values_cps(next, i + 1, values, e, k)))
            }
            Err(x) => k(Err(x)),
        }),
    )
}
''',
)

replace_once(
    "core/rust/src/fiber/coroutine.rs",
    '''#[path = "coroutine/observation.rs"]
mod observation;
#[path = "coroutine/snapshot.rs"]
mod snapshot;
pub use snapshot::{
    EvalBindingSnapshot, EvalErrorSnapshot, EvalObservationLimits, EvalObservationSnapshot,
    EvalObservationStatus, EvalObservedBoundary, EvalObservedBoundaryKind, EvalPendingSnapshot,
    EvalValueSnapshot, INTERPRETER_LIVE_BOUNDARY_SCHEMA, INTERPRETER_LIVE_SNAPSHOT_SCHEMA,
};
''',
    '''#[path = "coroutine/observation.rs"]
mod observation;
#[path = "coroutine/semantic.rs"]
pub(super) mod semantic;
#[path = "coroutine/snapshot.rs"]
mod snapshot;
pub use snapshot::{
    EvalBindingSnapshot, EvalErrorSnapshot, EvalFocusSnapshot, EvalFrameSnapshot,
    EvalObservationLimits, EvalObservationSnapshot, EvalObservationStatus, EvalObservedBoundary,
    EvalObservedBoundaryKind, EvalPendingSnapshot, EvalPositionSnapshot, EvalSemanticSnapshot,
    EvalSourceSpanSnapshot, EvalValueSnapshot, INTERPRETER_LIVE_BOUNDARY_SCHEMA,
    INTERPRETER_LIVE_SNAPSHOT_SCHEMA,
};
''',
)

Path("core/rust/src/fiber/coroutine/observation.rs").write_text(
    '''//! Opt-in live stepping for the production CPS evaluator.
//!
//! The ordinary [`EvalFiber::start`] path still drains every trampoline
//! continuation immediately. `start_observed` stores that same continuation
//! inside the existing fiber and executes at most one `Step::Continue`
//! boundary per `step_observed` call. Promise suspension keeps the real
//! promise and resume closure; no journal replay or alternate evaluator is
//! involved.

use super::semantic;
use super::super::*;

impl EvalFiber {
    /// Creates a live evaluator paused before the first production CPS step.
    pub fn start_observed(source: &str, env: HashMap<String, Value>) -> Result<Self, String> {
        let spanned = read_forms(source).map_err(|error| error.to_string())?;
        let forms = spanned.iter().map(|form| form.form.clone()).collect();
        Self::start_forms_observed_internal(forms, Some(Rc::new(spanned)), env)
    }

    /// Creates a live evaluator paused before evaluating forms without source spans.
    pub fn start_forms_observed(
        forms: Vec<Form>,
        env: HashMap<String, Value>,
    ) -> Result<Self, String> {
        Self::start_forms_observed_internal(forms, None, env)
    }

    fn start_forms_observed_internal(
        forms: Vec<Form>,
        source_forms: Option<Rc<Vec<SpannedForm>>>,
        env: HashMap<String, Value>,
    ) -> Result<Self, String> {
        let env = Rc::new(RefCell::new(env));
        semantic::register_context(&env, source_forms);
        let execution_env = env.clone();
        let forms = Rc::new(forms);
        let resume: Resume =
            Box::new(move |_| forms_cps(forms, 0, Value::Nil, execution_env, Box::new(Step::Done)));
        Ok(Self {
            env,
            pending: None,
            resume: Some(resume),
            state: EvalFiberState::Running,
        })
    }

    /// Returns true while an observed fiber owns a retained CPS continuation.
    pub fn observed_paused(&self) -> bool {
        matches!(self.state, EvalFiberState::Running)
            && self.pending.is_none()
            && self.resume.is_some()
    }

    /// Executes at most one retained production continuation boundary.
    pub fn step_observed(&mut self) -> EvalFiberState {
        if !matches!(self.state, EvalFiberState::Running) {
            return self.state();
        }
        let Some(resume) = self.resume.take() else {
            self.state = EvalFiberState::Failed("observed evaluator continuation missing".into());
            return self.state();
        };
        let step = semantic::with_active_context(&self.env, || resume(PromiseState::Pending));
        self.accept_observed(step);
        self.state()
    }

    /// Runs up to `boundary_limit` observed continuation boundaries.
    pub fn run_observed(&mut self, boundary_limit: usize) -> EvalFiberState {
        for _ in 0..boundary_limit {
            if !matches!(self.state, EvalFiberState::Running) {
                break;
            }
            self.step_observed();
        }
        self.state()
    }

    /// Applies one real promise settlement without draining later boundaries.
    pub fn resume_observed(&mut self, state: PromiseState) -> EvalFiberState {
        if !matches!(self.state, EvalFiberState::Suspended) {
            return self.state();
        }
        let Some(resume) = self.resume.take() else {
            self.state = EvalFiberState::Failed("fiber continuation missing".into());
            return self.state();
        };
        self.pending = None;
        self.state = EvalFiberState::Running;
        let step = semantic::with_active_context(&self.env, || resume(state));
        self.accept_observed(step);
        self.state()
    }

    fn accept_observed(&mut self, step: Step) {
        match step {
            Step::Continue(next) => {
                self.resume = Some(Box::new(move |_| next()));
                self.pending = None;
                self.state = EvalFiberState::Running;
            }
            Step::Done(Ok(value)) => {
                self.resume = None;
                self.pending = None;
                self.state = EvalFiberState::Completed(value);
            }
            Step::Done(Err(error)) => {
                self.resume = None;
                self.pending = None;
                self.state = EvalFiberState::Failed(error);
            }
            Step::Wait(promise, resume) => {
                self.pending = Some(promise);
                self.resume = Some(resume);
                self.state = EvalFiberState::Suspended;
            }
            Step::Yield(_, _) => {
                self.resume = None;
                self.pending = None;
                self.state =
                    EvalFiberState::Failed("coroutine/yield used outside of a coroutine".into());
            }
        }
    }
}

impl Drop for EvalFiber {
    fn drop(&mut self) {
        semantic::remove_context(&self.env);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_fiber_starts_paused_and_executes_one_trampoline_at_a_time() {
        let mut fiber =
            EvalFiber::start_observed("(do 1 2 (+ 1 (* 2 3)))", HashMap::new()).unwrap();
        assert_eq!(fiber.state(), EvalFiberState::Running);
        assert!(fiber.observed_paused());

        let first = fiber.run_observed(1);
        assert_eq!(first, EvalFiberState::Running);
        assert!(fiber.observed_paused());

        let mut boundaries = 1;
        while matches!(fiber.state(), EvalFiberState::Running) {
            fiber.step_observed();
            boundaries += 1;
            assert!(boundaries < 64, "observed evaluation did not terminate");
        }
        assert!(boundaries > 2);
        assert_eq!(fiber.state(), EvalFiberState::Completed(Value::Number(7)));
    }

    #[test]
    fn promise_suspension_retains_the_real_promise_and_resume_continuation() {
        let promise = Promise::new();
        let mut env = HashMap::new();
        env.insert("pending-value".into(), Value::Promise(promise.clone()));
        let mut fiber = EvalFiber::start_observed("(Coroutine/await pending-value)", env).unwrap();

        fiber.run_observed(16);
        assert_eq!(fiber.state(), EvalFiberState::Suspended);
        let retained = fiber.pending().expect("retained promise");
        assert!(retained.same_identity(&promise));

        assert!(promise.resolve(Value::Number(42)));
        let resumed = fiber.resume_observed(promise.state());
        assert_eq!(resumed, EvalFiberState::Running);
        assert!(fiber.observed_paused());

        let completed = fiber.run_observed(16);
        assert_eq!(completed, EvalFiberState::Completed(Value::Number(42)));
    }

    #[test]
    fn cancellation_discards_a_paused_live_continuation() {
        let mut fiber = EvalFiber::start_observed("(do 1 2 3)", HashMap::new()).unwrap();
        assert!(fiber.cancel());
        assert_eq!(fiber.state(), EvalFiberState::Cancelled);
        assert!(!fiber.observed_paused());
        assert_eq!(fiber.step_observed(), EvalFiberState::Cancelled);
        assert!(!fiber.cancel());
    }

    #[test]
    fn ordinary_eval_fiber_remains_full_speed() {
        let fiber = EvalFiber::start("(+ 19 23)", HashMap::new()).unwrap();
        assert_eq!(fiber.state(), EvalFiberState::Completed(Value::Number(42)));
    }
}
'''
)

replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    "use super::super::*;\nuse crate::lang::data::{OrderedMap, Vector};\n",
    "use super::semantic;\nuse super::super::*;\nuse crate::lang::data::{OrderedMap, Vector};\n",
)

replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalPendingSnapshot {
    pub state: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalObservationSnapshot {
''',
    '''#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalPendingSnapshot {
    pub state: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalPositionSnapshot {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalSourceSpanSnapshot {
    pub start: EvalPositionSnapshot,
    pub end: EvalPositionSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalFocusSnapshot {
    pub form: String,
    pub form_truncated: bool,
    pub form_kind: &'static str,
    pub path: Option<Vec<usize>>,
    pub span: Option<EvalSourceSpanSnapshot>,
    pub source_candidates: usize,
    pub ambiguous: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalFrameSnapshot {
    pub kind: &'static str,
    pub binding_count: usize,
    pub bindings: Vec<EvalBindingSnapshot>,
    pub bindings_omitted: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalSemanticSnapshot {
    pub sequence: usize,
    pub rule: &'static str,
    pub focus: EvalFocusSnapshot,
    pub result: EvalValueSnapshot,
    pub frames: Vec<EvalFrameSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalObservationSnapshot {
''',
)

replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''    pub bindings: Vec<EvalBindingSnapshot>,
    pub bindings_omitted: usize,
    pub pending: Option<EvalPendingSnapshot>,
''',
    '''    pub bindings: Vec<EvalBindingSnapshot>,
    pub bindings_omitted: usize,
    pub semantic: Option<EvalSemanticSnapshot>,
    pub pending: Option<EvalPendingSnapshot>,
''',
)

replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''            ("bindings", vector(self.bindings.iter().map(binding_value))),
            ("bindingsOmitted", integer(self.bindings_omitted)),
            (
                "pending",
''',
    '''            ("bindings", vector(self.bindings.iter().map(binding_value))),
            ("bindingsOmitted", integer(self.bindings_omitted)),
            (
                "semantic",
                optional_value(self.semantic.as_ref().map(semantic_value)),
            ),
            (
                "pending",
''',
)

replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''        let status = observation_status(self);
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
''',
    '''        let status = observation_status(self);
        let (binding_count, bindings, bindings_omitted) = {
            let environment = self.env.borrow();
            binding_projection(&environment, limits)
        };
        let semantic = semantic_snapshot(self, limits);
        let pending = self.pending.as_ref().map(|promise| EvalPendingSnapshot {
''',
)

replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''            bindings,
            bindings_omitted,
            pending,
''',
    '''            bindings,
            bindings_omitted,
            semantic,
            pending,
''',
)

replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''fn value_snapshot(value: &Value, display_chars: usize) -> EvalValueSnapshot {
''',
    '''fn binding_projection(
    environment: &HashMap<String, Value>,
    limits: EvalObservationLimits,
) -> (usize, Vec<EvalBindingSnapshot>, usize) {
    let mut bindings = environment
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
    (binding_count, bindings, bindings_omitted)
}

fn frame_snapshot(
    kind: &'static str,
    environment: &HashMap<String, Value>,
    limits: EvalObservationLimits,
) -> EvalFrameSnapshot {
    let (binding_count, bindings, bindings_omitted) = binding_projection(environment, limits);
    EvalFrameSnapshot {
        kind,
        binding_count,
        bindings,
        bindings_omitted,
    }
}

fn semantic_snapshot(
    fiber: &EvalFiber,
    limits: EvalObservationLimits,
) -> Option<EvalSemanticSnapshot> {
    let boundary = semantic::current_boundary(&fiber.env)?;
    let source_forms = semantic::source_forms(&fiber.env);
    let focus = focus_snapshot(
        &boundary.form,
        source_forms.as_deref().map(Vec::as_slice),
        limits.display_chars,
    );
    let current = frame_snapshot("current", &boundary.environment, limits);
    let session = {
        let environment = fiber.env.borrow();
        frame_snapshot("session", &environment, limits)
    };
    Some(EvalSemanticSnapshot {
        sequence: boundary.sequence,
        rule: boundary.rule.as_keyword(),
        focus,
        result: value_snapshot(&boundary.result, limits.display_chars),
        frames: vec![current, session],
    })
}

#[derive(Clone)]
struct SourceMatch {
    path: Vec<usize>,
    span: Span,
}

fn focus_snapshot(
    form: &Form,
    source_forms: Option<&[SpannedForm]>,
    display_chars: usize,
) -> EvalFocusSnapshot {
    let matches = source_forms
        .map(|forms| source_matches(forms, form))
        .unwrap_or_default();
    let unique = matches.len() == 1;
    let (path, span) = if unique {
        let matched = matches.into_iter().next().expect("one source match");
        (Some(matched.path), Some(span_snapshot(&matched.span)))
    } else {
        (None, None)
    };
    let source_candidates = if unique { 1 } else { matches.len() };
    let (form, form_truncated) = bounded_text(&form.to_string(), display_chars);
    EvalFocusSnapshot {
        form,
        form_truncated,
        form_kind: form_kind_from_display_source(path.as_ref(), span.as_ref()),
        path,
        span,
        source_candidates,
        ambiguous: source_candidates > 1,
    }
}

fn form_kind_from_display_source(
    _path: Option<&Vec<usize>>,
    _span: Option<&EvalSourceSpanSnapshot>,
) -> &'static str {
    "form"
}

fn source_matches(forms: &[SpannedForm], target: &Form) -> Vec<SourceMatch> {
    let mut output = Vec::new();
    collect_source_matches(forms, target, &[], &mut output);
    output
}

fn collect_source_matches(
    forms: &[SpannedForm],
    target: &Form,
    prefix: &[usize],
    output: &mut Vec<SourceMatch>,
) {
    for (index, form) in forms.iter().enumerate() {
        let mut path = prefix.to_vec();
        path.push(index);
        if &form.form == target {
            output.push(SourceMatch {
                path: path.clone(),
                span: form.span.clone(),
            });
        }
        collect_source_matches(&form.children, target, &path, output);
    }
}

fn span_snapshot(span: &Span) -> EvalSourceSpanSnapshot {
    EvalSourceSpanSnapshot {
        start: position_snapshot(span.start),
        end: position_snapshot(span.end),
    }
}

fn position_snapshot(position: Position) -> EvalPositionSnapshot {
    EvalPositionSnapshot {
        offset: position.offset,
        line: position.line,
        column: position.column,
    }
}

fn value_snapshot(value: &Value, display_chars: usize) -> EvalValueSnapshot {
''',
)

# Replace the temporary form-kind helper with the actual form-aware call and function.
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''    let (form, form_truncated) = bounded_text(&form.to_string(), display_chars);
    EvalFocusSnapshot {
        form,
        form_truncated,
        form_kind: form_kind_from_display_source(path.as_ref(), span.as_ref()),
''',
    '''    let form_kind = form_kind(form);
    let (form, form_truncated) = bounded_text(&form.to_string(), display_chars);
    EvalFocusSnapshot {
        form,
        form_truncated,
        form_kind,
''',
)
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''fn form_kind_from_display_source(
    _path: Option<&Vec<usize>>,
    _span: Option<&EvalSourceSpanSnapshot>,
) -> &'static str {
    "form"
}

fn source_matches''',
    '''fn form_kind(form: &Form) -> &'static str {
    match form {
        Form::Symbol(_) => "symbol",
        Form::List(values) => match values.first() {
            Some(Form::Symbol(name)) if SYNC_SPECIAL_FORMS.contains(&name.as_str()) => {
                "special-form"
            }
            _ => "call",
        },
        Form::Map(_) | Form::Set(_) | Form::Vector(_) => "collection",
        Form::Metadata(_, _) => "metadata",
        Form::Tagged(_, _) => "tagged",
        _ => "literal",
    }
}

fn source_matches''',
)

replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''fn binding_value(binding: &EvalBindingSnapshot) -> Value {
''',
    '''fn semantic_value(semantic: &EvalSemanticSnapshot) -> Value {
    object([
        ("sequence", integer(semantic.sequence)),
        ("rule", string(semantic.rule)),
        ("focus", focus_value(&semantic.focus)),
        ("result", value_snapshot_value(&semantic.result)),
        ("frames", vector(semantic.frames.iter().map(frame_value))),
    ])
}

fn focus_value(focus: &EvalFocusSnapshot) -> Value {
    object([
        ("form", string(&focus.form)),
        ("formTruncated", Value::Bool(focus.form_truncated)),
        ("formKind", string(focus.form_kind)),
        (
            "path",
            optional_value(
                focus
                    .path
                    .as_ref()
                    .map(|path| vector(path.iter().copied().map(integer))),
            ),
        ),
        (
            "span",
            optional_value(focus.span.as_ref().map(source_span_value)),
        ),
        ("sourceCandidates", integer(focus.source_candidates)),
        ("ambiguous", Value::Bool(focus.ambiguous)),
    ])
}

fn source_span_value(span: &EvalSourceSpanSnapshot) -> Value {
    object([
        ("start", position_value(&span.start)),
        ("end", position_value(&span.end)),
    ])
}

fn position_value(position: &EvalPositionSnapshot) -> Value {
    object([
        ("offset", integer(position.offset)),
        ("line", integer(position.line)),
        ("column", integer(position.column)),
    ])
}

fn frame_value(frame: &EvalFrameSnapshot) -> Value {
    object([
        ("kind", string(frame.kind)),
        ("bindingCount", integer(frame.binding_count)),
        ("bindings", vector(frame.bindings.iter().map(binding_value))),
        ("bindingsOmitted", integer(frame.bindings_omitted)),
    ])
}

fn binding_value(binding: &EvalBindingSnapshot) -> Value {
''',
)

replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''        assert!(!json.contains("identity"));
    }
}
''',
    '''        assert!(!json.contains("identity"));
    }

    fn collect_semantics(source: &str) -> Vec<EvalSemanticSnapshot> {
        let mut fiber = EvalFiber::start_observed(source, HashMap::new()).unwrap();
        let mut output = Vec::new();
        let mut sequence = 0;
        while matches!(fiber.state(), EvalFiberState::Running) {
            let boundary = fiber.step_observed_snapshot(
                "fixture/semantic.hal",
                EvalObservationLimits::default(),
            );
            if let Some(semantic) = boundary.after.semantic {
                if semantic.sequence > sequence {
                    sequence = semantic.sequence;
                    output.push(semantic);
                }
            }
            assert!(sequence < 128, "semantic evaluation did not terminate");
        }
        output
    }

    #[test]
    fn nested_calls_retain_actual_result_form_path_and_span() {
        let semantics = collect_semantics("(+ 1 (* 2 3))");
        let multiply = semantics
            .iter()
            .find(|semantic| semantic.focus.form == "(* 2 3)")
            .expect("inner multiply boundary");
        assert_eq!(multiply.result.display, "6");
        assert_eq!(multiply.focus.form_kind, "call");
        assert_eq!(multiply.focus.path.as_deref(), Some(&[0, 2][..]));
        assert_eq!(multiply.focus.source_candidates, 1);
        assert_eq!(
            multiply.focus.span.as_ref().map(|span| (span.start.offset, span.end.offset)),
            Some((5, 12))
        );

        let outer = semantics
            .iter()
            .find(|semantic| {
                semantic.focus.form == "(+ 1 (* 2 3))" && semantic.result.display == "7"
            })
            .expect("outer addition boundary");
        assert_eq!(outer.focus.path.as_deref(), Some(&[0][..]));
    }

    #[test]
    fn lexical_boundary_captures_binding_before_scope_restoration() {
        let semantics = collect_semantics("(let [x 41] (+ x 1))");
        let resolved = semantics
            .iter()
            .find(|semantic| semantic.focus.form == "x" && semantic.result.display == "41")
            .expect("resolved lexical symbol boundary");
        let current = resolved
            .frames
            .iter()
            .find(|frame| frame.kind == "current")
            .expect("current lexical frame");
        let x = current
            .bindings
            .iter()
            .find(|binding| binding.name == "x")
            .expect("captured x binding");
        assert_eq!(x.value.display, "41");
    }

    #[test]
    fn duplicate_source_forms_are_explicitly_ambiguous() {
        let semantics = collect_semantics("(+ 1 1)");
        let literal = semantics
            .iter()
            .find(|semantic| semantic.focus.form == "1")
            .expect("literal boundary");
        assert_eq!(literal.focus.source_candidates, 2);
        assert!(literal.focus.ambiguous);
        assert!(literal.focus.path.is_none());
        assert!(literal.focus.span.is_none());
    }
}
''',
)

replace_once(
    ".github/workflows/code-vm-live-interpreter.yml",
    '''      - 'notes/issue-403-live-evaluator-fiber.md'
      - '.github/workflows/code-vm-live-interpreter.yml'
''',
    '''      - 'notes/issue-403-live-evaluator-fiber.md'
      - 'notes/issue-403-live-semantic-boundaries.md'
      - '.github/workflows/code-vm-live-interpreter.yml'
''',
)
replace_once(
    ".github/workflows/code-vm-live-interpreter.yml",
    '''      - 'notes/issue-403-live-evaluator-fiber.md'
      - '.github/workflows/code-vm-live-interpreter.yml'
''',
    '''      - 'notes/issue-403-live-evaluator-fiber.md'
      - 'notes/issue-403-live-semantic-boundaries.md'
      - '.github/workflows/code-vm-live-interpreter.yml'
''',
)
replace_once(
    ".github/workflows/code-vm-live-interpreter.yml",
    '''          rustfmt --edition 2021 --check \\
            core/rust/src/fiber/coroutine.rs \\
            core/rust/src/fiber/coroutine/observation.rs
''',
    '''          rustfmt --edition 2021 --check \\
            core/rust/src/fiber.rs \\
            core/rust/src/fiber/coroutine.rs \\
            core/rust/src/fiber/coroutine/observation.rs \\
            core/rust/src/fiber/coroutine/semantic.rs \\
            core/rust/src/fiber/coroutine/snapshot.rs
''',
)
replace_once(
    ".github/workflows/code-vm-live-interpreter.yml",
    '''          cargo test --manifest-path core/rust/Cargo.toml \\
            core::fiber::coroutine::observation --lib
''',
    '''          cargo test --manifest-path core/rust/Cargo.toml \\
            core::fiber::coroutine --lib
''',
)
