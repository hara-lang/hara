//! Opt-in live stepping for the production CPS evaluator.
//!
//! The ordinary [`EvalFiber::start`] path still drains every trampoline
//! continuation immediately. `start_observed` stores that same continuation
//! inside the existing fiber and executes at most one `Step::Continue`
//! boundary per `step_observed` call. Promise suspension keeps the real
//! promise and resume closure; no journal replay or alternate evaluator is
//! involved.

use super::super::*;
use super::semantic;
use crate::kernel::{read_forms, SpannedForm};

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
        semantic::advance_pending(&self.env);
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
