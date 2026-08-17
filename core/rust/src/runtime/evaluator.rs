/// Runtime-owned lexical evaluation state.
///
/// Namespace, provider, package, Session, and Kernel state deliberately stay
/// outside this type. `Runtime` installs those capabilities around each call.
#[derive(Default)]
struct Evaluator {
    environment: HashMap<String, core::Value>,
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
    }
}
