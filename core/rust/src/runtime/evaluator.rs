/// Runtime-owned lexical evaluation and instrumentation state.
///
/// Namespace, provider, package, Session, and Kernel state deliberately stay
/// outside this type. The instrumentation hub follows the Runtime lifecycle
/// here while remaining separate from the lexical environment and execution
/// targets.
#[derive(Default)]
struct Evaluator {
    environment: HashMap<String, core::Value>,
    instrumentation: instrumentation::InstrumentationHub,
}

impl Evaluator {
    fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    fn environment(&self) -> &HashMap<String, core::Value> {
        &self.environment
    }

    fn environment_mut(&mut self) -> &mut HashMap<String, core::Value> {
        &mut self.environment
    }

    fn snapshot(&self) -> HashMap<String, core::Value> {
        self.environment.clone()
    }

    fn restore(&mut self, environment: HashMap<String, core::Value>) {
        self.environment = environment;
    }

    fn eval_tree(&mut self, form: &Form) -> Result<core::Value, String> {
        core::eval_traced(form, &mut self.environment)
    }

    fn start_fiber(&self, form: Form) -> Result<core::EvalFiber, String> {
        core::EvalFiber::start_forms(vec![form], self.environment.clone())
    }

    fn finish_fiber(&mut self, fiber: &core::EvalFiber) {
        self.environment = fiber.environment();
    }

    fn clear(&mut self) {
        self.environment.clear();
        self.instrumentation.clear();
    }
}
