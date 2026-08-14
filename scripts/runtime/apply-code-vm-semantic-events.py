#!/usr/bin/env python3
"""Apply the explicit call/effect/error #403 semantic event slice."""

from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    source = target.read_text()
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one replacement, found {count}")
    target.write_text(source.replace(old, new, 1))


# Publish queued evidence before executing another retained continuation.
replace_once(
    "core/rust/src/fiber/coroutine/observation.rs",
    '''    pub fn step_observed(&mut self) -> EvalFiberState {
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
''',
    '''    pub fn step_observed(&mut self) -> EvalFiberState {
        if semantic::advance_pending(&self.env) {
            return self.state();
        }
        if !matches!(self.state, EvalFiberState::Running) {
            return self.state();
        }
        let Some(resume) = self.resume.take() else {
            self.state = EvalFiberState::Failed("observed evaluator continuation missing".into());
            return self.state();
        };
        let step = semantic::with_active_context(&self.env, || resume(PromiseState::Pending));
        self.accept_observed(step);
        semantic::advance_pending(&self.env);
        self.state()
    }

    /// Runs up to `boundary_limit` evaluator or queued semantic boundaries.
    pub fn run_observed(&mut self, boundary_limit: usize) -> EvalFiberState {
        for _ in 0..boundary_limit {
            if !matches!(self.state, EvalFiberState::Running)
                && semantic::pending_count(&self.env) == 0
            {
                break;
            }
            self.step_observed();
        }
        self.state()
    }
''',
)
replace_once(
    "core/rust/src/fiber/coroutine/observation.rs",
    '''        let step = semantic::with_active_context(&self.env, || resume(state));
        self.accept_observed(step);
        self.state()
''',
    '''        let step = semantic::with_active_context(&self.env, || resume(state));
        self.accept_observed(step);
        semantic::advance_pending(&self.env);
        self.state()
''',
)

# Preserve raised errors at the exact form-producing seams.
replace_once(
    "core/rust/src/fiber.rs",
    '''            Err(x) => k(Err(x)),
        }),
    )
}
fn values_cps''',
    '''            Err(x) => {
                coroutine::semantic::record_error(
                    coroutine::semantic::EvalSemanticRule::ErrorRaise,
                    &boundary_form,
                    &x,
                    false,
                    &boundary_env,
                );
                k(Err(x))
            }
        }),
    )
}
fn values_cps''',
)
replace_once(
    "core/rust/src/fiber.rs",
    '''            Err(x) => k(Err(x)),
        }),
    )
}
fn one''',
    '''            Err(x) => {
                coroutine::semantic::record_error(
                    coroutine::semantic::EvalSemanticRule::ErrorRaise,
                    &boundary_form,
                    &x,
                    false,
                    &boundary_env,
                );
                k(Err(x))
            }
        }),
    )
}
fn one''',
)

# Record authoritative mutable-field commits.
replace_once(
    "core/rust/src/fiber.rs",
    '''fn set_form(v: Vec<Form>, env: Rc<RefCell<HashMap<String, Value>>>, k: Cont) -> Step {
    if v.len() != 3 {
''',
    '''fn set_form(v: Vec<Form>, env: Rc<RefCell<HashMap<String, Value>>>, k: Cont) -> Step {
    let effect_form = Form::List(v.clone());
    if v.len() != 3 {
''',
)
replace_once(
    "core/rust/src/fiber.rs",
    '''    let replacement = v[2].clone();
    let replacement_env = env.clone();
''',
    '''    let replacement = v[2].clone();
    let replacement_env = env.clone();
    let effect_env = replacement_env.clone();
''',
)
replace_once(
    "core/rust/src/fiber.rs",
    '''                Box::new(move |replacement_result| match replacement_result {
                    Ok(replacement) => k(crate::core::mutable_field_set(
                        &receiver,
                        &field,
                        replacement,
                    )),
                    Err(error) => k(Err(error)),
                }),
''',
    '''                Box::new(move |replacement_result| match replacement_result {
                    Ok(replacement) => {
                        let after = replacement.clone();
                        match crate::core::mutable_field_set(&receiver, &field, replacement) {
                            Ok(result) => {
                                coroutine::semantic::record_effect(
                                    coroutine::semantic::EvalSemanticRule::FieldSet,
                                    &effect_form,
                                    field,
                                    None,
                                    after,
                                    &effect_env,
                                );
                                k(Ok(result))
                            }
                            Err(error) => k(Err(error)),
                        }
                    }
                    Err(error) => k(Err(error)),
                }),
''',
)

# Record Var definition/set commits with before and after values.
replace_once(
    "core/rust/src/fiber.rs",
    '''fn bind_form(v: Vec<Form>, env: Rc<RefCell<HashMap<String, Value>>>, k: Cont) -> Step {
    if v.len() != 3 {
''',
    '''fn bind_form(v: Vec<Form>, env: Rc<RefCell<HashMap<String, Value>>>, k: Cont) -> Step {
    let effect_form = Form::List(v.clone());
    if v.len() != 3 {
''',
)
replace_once(
    "core/rust/src/fiber.rs",
    '''            Ok(x) => {
                let mut env = e.borrow_mut();
                let result = if op == "def" {
''',
    '''            Ok(x) => {
                let effect_after = x.clone();
                let mut effect_before = None;
                let mut effect_target = name.clone();
                let mut env = e.borrow_mut();
                let result = if op == "def" {
''',
)
replace_once(
    "core/rust/src/fiber.rs",
    '''                    let origin = crate::core::definition_origin();
                    let var = if let Some(Value::Var(var)) = env.get(&name) {
''',
    '''                    let origin = crate::core::definition_origin();
                    let var = if let Some(Value::Var(var)) = env.get(&name) {
                        effect_before = Some(var.deref_value());
''',
)
replace_once(
    "core/rust/src/fiber.rs",
    '''                    };
                    Value::Var(var)
                } else {
                    let Some(c) = binding_var(&mut env, &name) else {
''',
    '''                    };
                    effect_target = var.display();
                    Value::Var(var)
                } else {
                    let Some(c) = binding_var(&mut env, &name) else {
''',
)
replace_once(
    "core/rust/src/fiber.rs",
    '''                    c.reset_value(x.clone());
                    if let Some(meta) = metadata {
''',
    '''                    effect_before = Some(c.deref_value());
                    effect_target = c.display();
                    c.reset_value(x.clone());
                    if let Some(meta) = metadata {
''',
)
replace_once(
    "core/rust/src/fiber.rs",
    '''                };
                drop(env);
                k(Ok(result))
''',
    '''                };
                drop(env);
                coroutine::semantic::record_effect(
                    if op == "def" {
                        coroutine::semantic::EvalSemanticRule::VarDefine
                    } else {
                        coroutine::semantic::EvalSemanticRule::VarSet
                    },
                    &effect_form,
                    effect_target,
                    effect_before,
                    effect_after,
                    &e,
                );
                k(Ok(result))
''',
)

# Preserve the selected catch/unwind boundary.
replace_once(
    "core/rust/src/fiber.rs",
    '''            let (binding_index, body_index) = match p.len() {
''',
    '''            let catch_form = Form::List(p.clone());
            let (binding_index, body_index) = match p.len() {
''',
)
replace_once(
    "core/rust/src/fiber.rs",
    '''            let old = env.borrow_mut().insert(n.clone(), caught_error(&x));
            let e = env.clone();
''',
    '''            let old = env.borrow_mut().insert(n.clone(), caught_error(&x));
            coroutine::semantic::record_error(
                coroutine::semantic::EvalSemanticRule::ErrorCatch,
                &catch_form,
                &x,
                true,
                &env,
            );
            let e = env.clone();
''',
)

# Record named function calls after authoritative argument evaluation.
replace_once(
    "core/rust/src/fiber.rs",
    '''    if let Some(Value::Function(f)) = f {
        return values_cps(
            Rc::new(v[1..].to_vec()),
            0,
            Vec::new(),
            env,
            Box::new(move |r| match r {
                Ok(a) => call(f, a, k),
                Err(x) => k(Err(x)),
            }),
        );
    }
''',
    '''    if let Some(Value::Function(f)) = f {
        let call_form = Form::List(v.clone());
        let call_name = f.name.clone().unwrap_or_else(|| "<anonymous>".into());
        let call_env = env.clone();
        return values_cps(
            Rc::new(v[1..].to_vec()),
            0,
            Vec::new(),
            env,
            Box::new(move |r| match r {
                Ok(a) => {
                    coroutine::semantic::record_call(
                        &call_form,
                        call_name,
                        &a,
                        &call_env,
                    );
                    call(f, a, k)
                }
                Err(x) => k(Err(x)),
            }),
        );
    }
''',
)

# Record calls whose function value is itself computed.
replace_once(
    "core/rust/src/fiber.rs",
    '''    if head_symbol.is_none() {
        let forms = Rc::new(v[1..].to_vec());
        let arguments_env = env.clone();
        return one(
            v[0].clone(),
            env,
            Box::new(move |result| match result {
                Ok(Value::Function(function)) => values_cps(
                    forms,
                    0,
                    Vec::new(),
                    arguments_env,
                    Box::new(move |arguments| match arguments {
                        Ok(arguments) => call(function, arguments, k),
                        Err(error) => k(Err(error)),
                    }),
                ),
                Ok(value) => values_cps(
                    forms,
                    0,
                    Vec::new(),
                    arguments_env,
                    Box::new(move |arguments| match arguments {
                        Ok(arguments) => k(crate::core::call_value(value, arguments)),
                        Err(error) => k(Err(error)),
                    }),
                ),
                Err(error) => k(Err(error)),
            }),
        );
    }
''',
    '''    if head_symbol.is_none() {
        let forms = Rc::new(v[1..].to_vec());
        let arguments_env = env.clone();
        let call_form = Form::List(v.clone());
        let function_call_form = call_form.clone();
        let function_call_env = arguments_env.clone();
        let value_call_env = arguments_env.clone();
        return one(
            v[0].clone(),
            env,
            Box::new(move |result| match result {
                Ok(Value::Function(function)) => {
                    let call_name = function
                        .name
                        .clone()
                        .unwrap_or_else(|| "<anonymous>".into());
                    values_cps(
                        forms,
                        0,
                        Vec::new(),
                        arguments_env,
                        Box::new(move |arguments| match arguments {
                            Ok(arguments) => {
                                coroutine::semantic::record_call(
                                    &function_call_form,
                                    call_name,
                                    &arguments,
                                    &function_call_env,
                                );
                                call(function, arguments, k)
                            }
                            Err(error) => k(Err(error)),
                        }),
                    )
                }
                Ok(value) => {
                    let call_name = crate::core::portable_type_name(&value).to_owned();
                    values_cps(
                        forms,
                        0,
                        Vec::new(),
                        arguments_env,
                        Box::new(move |arguments| match arguments {
                            Ok(arguments) => {
                                coroutine::semantic::record_call(
                                    &call_form,
                                    call_name,
                                    &arguments,
                                    &value_call_env,
                                );
                                k(crate::core::call_value(value, arguments))
                            }
                            Err(error) => k(Err(error)),
                        }),
                    )
                }
                Err(error) => k(Err(error)),
            }),
        );
    }
''',
)

# Record primitive/native calls before synchronous dispatch.
replace_once(
    "core/rust/src/fiber.rs",
    '''fn eval_special_form(v: Vec<Form>, env: Rc<RefCell<HashMap<String, Value>>>, k: Cont) -> Step {
    let op = v[0].clone();
    let e = env.clone();
''',
    '''fn eval_special_form(v: Vec<Form>, env: Rc<RefCell<HashMap<String, Value>>>, k: Cont) -> Step {
    let call_form = Form::List(v.clone());
    let op = v[0].clone();
    let call_name = match &op {
        Form::Symbol(name) => name.clone(),
        _ => "<callable>".into(),
    };
    let e = env.clone();
''',
)
replace_once(
    "core/rust/src/fiber.rs",
    '''            Ok(values) => {
                let mut env = e.borrow_mut();
''',
    '''            Ok(values) => {
                coroutine::semantic::record_call(&call_form, call_name, &values, &e);
                let mut env = e.borrow_mut();
''',
)

# Extend the public snapshot projection.
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''pub enum EvalObservedBoundaryKind {
    Continue,
''',
    '''pub enum EvalObservedBoundaryKind {
    Semantic,
    Continue,
''',
)
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''        match self {
            Self::Continue => "evaluation/continue",
''',
    '''        match self {
            Self::Semantic => "evaluation/semantic",
            Self::Continue => "evaluation/continue",
''',
)
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalSemanticSnapshot {
    pub sequence: usize,
    pub rule: &'static str,
    pub focus: EvalFocusSnapshot,
    pub result: EvalValueSnapshot,
    pub frames: Vec<EvalFrameSnapshot>,
}
''',
    '''#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalSemanticCallSnapshot {
    pub name: String,
    pub arity: usize,
    pub arguments: Vec<EvalValueSnapshot>,
    pub arguments_omitted: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalSemanticEffectSnapshot {
    pub target: String,
    pub before: Option<EvalValueSnapshot>,
    pub after: EvalValueSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalSemanticErrorSnapshot {
    pub category: &'static str,
    pub message: String,
    pub truncated: bool,
    pub caught: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalSemanticSnapshot {
    pub sequence: usize,
    pub rule: &'static str,
    pub focus: EvalFocusSnapshot,
    pub result: Option<EvalValueSnapshot>,
    pub call: Option<EvalSemanticCallSnapshot>,
    pub effect: Option<EvalSemanticEffectSnapshot>,
    pub error: Option<EvalSemanticErrorSnapshot>,
    pub frames: Vec<EvalFrameSnapshot>,
}
''',
)
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''    pub bindings_omitted: usize,
    pub semantic: Option<EvalSemanticSnapshot>,
''',
    '''    pub bindings_omitted: usize,
    pub semantic_pending: usize,
    pub semantic: Option<EvalSemanticSnapshot>,
''',
)
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''            ("bindingsOmitted", integer(self.bindings_omitted)),
            (
                "semantic",
''',
    '''            ("bindingsOmitted", integer(self.bindings_omitted)),
            ("semanticPending", integer(self.semantic_pending)),
            (
                "semantic",
''',
)
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''        let semantic = semantic_snapshot(self, limits);
        let pending = self.pending.as_ref().map(|promise| EvalPendingSnapshot {
''',
    '''        let semantic_pending = semantic::pending_count(&self.env);
        let semantic = semantic_snapshot(self, limits);
        let pending = self.pending.as_ref().map(|promise| EvalPendingSnapshot {
''',
)
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''            bindings_omitted,
            semantic,
''',
    '''            bindings_omitted,
            semantic_pending,
            semantic,
''',
)

# Classify publication-only steps separately from evaluator progress.
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''fn boundary_kind(
    before: &EvalObservationSnapshot,
    after: &EvalObservationSnapshot,
    resumed: bool,
) -> EvalObservedBoundaryKind {
    match after.status {
''',
    '''fn boundary_kind(
    before: &EvalObservationSnapshot,
    after: &EvalObservationSnapshot,
    resumed: bool,
) -> EvalObservedBoundaryKind {
    let before_sequence = before.semantic.as_ref().map(|semantic| semantic.sequence);
    let after_sequence = after.semantic.as_ref().map(|semantic| semantic.sequence);
    let semantic_advanced = before_sequence != after_sequence;
    if semantic_advanced && before.status == after.status {
        return EvalObservedBoundaryKind::Semantic;
    }
    match after.status {
''',
)

# Project each semantic payload without exposing runtime handles.
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''    Some(EvalSemanticSnapshot {
        sequence: boundary.sequence,
        rule: boundary.rule.as_keyword(),
        focus,
        result: value_snapshot(&boundary.result, limits.display_chars),
        frames: vec![current, session],
    })
}
''',
    '''    let (result, call, effect, error) = match &boundary.payload {
        semantic::EvalSemanticPayload::Result(value) => (
            Some(value_snapshot(value, limits.display_chars)),
            None,
            None,
            None,
        ),
        semantic::EvalSemanticPayload::Call { name, arguments } => {
            let arity = arguments.len();
            let retained = arguments
                .iter()
                .take(limits.bindings)
                .map(|value| value_snapshot(value, limits.display_chars))
                .collect::<Vec<_>>();
            (
                None,
                Some(EvalSemanticCallSnapshot {
                    name: name.clone(),
                    arity,
                    arguments_omitted: arity.saturating_sub(retained.len()),
                    arguments: retained,
                }),
                None,
                None,
            )
        }
        semantic::EvalSemanticPayload::Effect {
            target,
            before,
            after,
        } => (
            None,
            None,
            Some(EvalSemanticEffectSnapshot {
                target: target.clone(),
                before: before
                    .as_ref()
                    .map(|value| value_snapshot(value, limits.display_chars)),
                after: value_snapshot(after, limits.display_chars),
            }),
            None,
        ),
        semantic::EvalSemanticPayload::Error { message, caught } => {
            let (message, truncated) = bounded_text(message, limits.display_chars);
            (
                None,
                None,
                None,
                Some(EvalSemanticErrorSnapshot {
                    category: normalized_error_category(&message),
                    message,
                    truncated,
                    caught: *caught,
                }),
            )
        }
    };
    Some(EvalSemanticSnapshot {
        sequence: boundary.sequence,
        rule: boundary.rule.as_keyword(),
        focus,
        result,
        call,
        effect,
        error,
        frames: vec![current, session],
    })
}

fn normalized_error_category(message: &str) -> &'static str {
    let message = message.to_ascii_lowercase();
    if message.contains("division by zero")
        || message.contains("divide by zero")
        || message.contains("/ by zero")
    {
        "division by zero"
    } else if message.contains("expects numbers")
        || message.contains("expects two numbers")
        || message.contains("expected a number")
        || message.contains("expected numeric")
    {
        "expects numbers"
    } else if message.contains("unbound symbol") || message.contains("unbound var") {
        "unbound symbol"
    } else if message.contains("recur") {
        "recur"
    } else if message.contains("unsupported") {
        "unsupported form"
    } else {
        "runtime"
    }
}
''',
)

# Serialize optional semantic payloads.
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''        ("focus", focus_value(&semantic.focus)),
        ("result", value_snapshot_value(&semantic.result)),
        ("frames", vector(semantic.frames.iter().map(frame_value))),
''',
    '''        ("focus", focus_value(&semantic.focus)),
        (
            "result",
            optional_value(semantic.result.as_ref().map(value_snapshot_value)),
        ),
        (
            "call",
            optional_value(semantic.call.as_ref().map(semantic_call_value)),
        ),
        (
            "effect",
            optional_value(semantic.effect.as_ref().map(semantic_effect_value)),
        ),
        (
            "error",
            optional_value(semantic.error.as_ref().map(semantic_error_value)),
        ),
        ("frames", vector(semantic.frames.iter().map(frame_value))),
''',
)
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''fn focus_value(focus: &EvalFocusSnapshot) -> Value {
''',
    '''fn semantic_call_value(call: &EvalSemanticCallSnapshot) -> Value {
    object([
        ("name", string(&call.name)),
        ("arity", integer(call.arity)),
        (
            "arguments",
            vector(call.arguments.iter().map(value_snapshot_value)),
        ),
        ("argumentsOmitted", integer(call.arguments_omitted)),
    ])
}

fn semantic_effect_value(effect: &EvalSemanticEffectSnapshot) -> Value {
    object([
        ("target", string(&effect.target)),
        (
            "before",
            optional_value(effect.before.as_ref().map(value_snapshot_value)),
        ),
        ("after", value_snapshot_value(&effect.after)),
    ])
}

fn semantic_error_value(error: &EvalSemanticErrorSnapshot) -> Value {
    object([
        ("category", string(error.category)),
        ("message", string(&error.message)),
        ("truncated", Value::Bool(error.truncated)),
        ("caught", Value::Bool(error.caught)),
    ])
}

fn focus_value(focus: &EvalFocusSnapshot) -> Value {
''',
)

# Existing return assertions now unwrap optional result payloads.
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''        assert_eq!(multiply.result.display, "6");
''',
    '''        assert_eq!(
            multiply.result.as_ref().map(|result| result.display.as_str()),
            Some("6")
        );
''',
)
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''                semantic.focus.form == "(+ 1 (* 2 3))" && semantic.result.display == "7"
''',
    '''                semantic.focus.form == "(+ 1 (* 2 3))"
                    && semantic
                        .result
                        .as_ref()
                        .is_some_and(|result| result.display == "7")
''',
)

# Drain queued events even after evaluator termination.
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''        while matches!(fiber.state(), EvalFiberState::Running) {
            let boundary = fiber
                .step_observed_snapshot("fixture/semantic.hal", EvalObservationLimits::default());
''',
    '''        loop {
            let snapshot =
                fiber.snapshot_observed("fixture/semantic.hal", EvalObservationLimits::default());
            if !matches!(fiber.state(), EvalFiberState::Running)
                && snapshot.semantic_pending == 0
            {
                break;
            }
            let boundary = fiber
                .step_observed_snapshot("fixture/semantic.hal", EvalObservationLimits::default());
''',
)

# Add event-order, mutation, and error/catch proofs.
replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''    fn duplicate_source_forms_are_explicitly_ambiguous() {
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
    '''    fn duplicate_source_forms_are_explicitly_ambiguous() {
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

    #[test]
    fn call_entry_is_published_before_the_matching_return() {
        let semantics = collect_semantics("(+ 1 (* 2 3))");
        let enter = semantics
            .iter()
            .position(|semantic| {
                semantic.rule == "call/enter" && semantic.focus.form == "(* 2 3)"
            })
            .expect("inner call entry");
        let returned = semantics
            .iter()
            .position(|semantic| {
                semantic.rule == "form/return"
                    && semantic.focus.form == "(* 2 3)"
                    && semantic
                        .result
                        .as_ref()
                        .is_some_and(|result| result.display == "6")
            })
            .expect("inner call return");
        assert!(enter < returned);
        let call = semantics[enter].call.as_ref().expect("call payload");
        assert_eq!(call.arity, 2);
        assert_eq!(
            call.arguments
                .iter()
                .map(|argument| argument.display.as_str())
                .collect::<Vec<_>>(),
            vec!["2", "3"]
        );
    }

    #[test]
    fn var_mutations_are_explicit_ordered_effects() {
        let semantics = collect_semantics(
            "(do (def counter 1) (set! counter 42) counter)",
        );
        let define = semantics
            .iter()
            .find(|semantic| semantic.rule == "effect/var-define")
            .expect("definition effect");
        let define_effect = define.effect.as_ref().expect("definition payload");
        assert_eq!(define_effect.after.display, "1");

        let set = semantics
            .iter()
            .find(|semantic| semantic.rule == "effect/var-set")
            .expect("set effect");
        let set_effect = set.effect.as_ref().expect("set payload");
        assert_eq!(
            set_effect.before.as_ref().map(|value| value.display.as_str()),
            Some("1")
        );
        assert_eq!(set_effect.after.display, "42");
        assert!(define.sequence < set.sequence);
    }

    #[test]
    fn raised_errors_and_selected_catches_are_explicit() {
        let semantics = collect_semantics(
            "(try (/ 1 0) (catch Exception error 42))",
        );
        let raised = semantics
            .iter()
            .find(|semantic| semantic.rule == "error/raise")
            .expect("raise event");
        let raised_error = raised.error.as_ref().expect("raise payload");
        assert_eq!(raised_error.category, "division by zero");
        assert!(!raised_error.caught);
        assert_eq!(raised.focus.form, "(/ 1 0)");

        let caught = semantics
            .iter()
            .find(|semantic| semantic.rule == "error/catch")
            .expect("catch event");
        assert!(caught.error.as_ref().is_some_and(|error| error.caught));
        assert!(raised.sequence < caught.sequence);
    }
}
''',
)

# Export the new snapshot payload types.
replace_once(
    "core/rust/src/fiber/coroutine.rs",
    '''    EvalObservedBoundaryKind, EvalPendingSnapshot, EvalPositionSnapshot, EvalSemanticSnapshot,
    EvalSourceSpanSnapshot, EvalValueSnapshot, INTERPRETER_LIVE_BOUNDARY_SCHEMA,
''',
    '''    EvalObservedBoundaryKind, EvalPendingSnapshot, EvalPositionSnapshot,
    EvalSemanticCallSnapshot, EvalSemanticEffectSnapshot, EvalSemanticErrorSnapshot,
    EvalSemanticSnapshot, EvalSourceSpanSnapshot, EvalValueSnapshot,
    INTERPRETER_LIVE_BOUNDARY_SCHEMA,
''',
)
