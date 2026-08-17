/// Provider-side live sandbox. Implementations own backend launch, execution,
/// cancellation, and termination details.
pub trait SandboxInstance {
    fn eval(&mut self, source: &str) -> Result<String, SandboxError>;
    fn call(&mut self, callable: &str, arguments_hta: &[u8]) -> Result<Vec<u8>, SandboxError>;
    fn cancel(&mut self) -> Result<bool, SandboxError>;
    fn state(&self) -> SandboxState;
    fn close(&mut self) -> Result<(), SandboxError>;
}

pub trait SandboxProvider {
    fn name(&self) -> &str;
    fn secure(&self) -> bool;
    fn open(&self, spec: &SandboxSpec) -> Result<Box<dyn SandboxInstance>, SandboxError>;
}

/// Conformance-only provider. Runtime separation is logical and is not a
/// security boundary.
#[derive(Default)]
pub struct InProcessSandboxProvider;

impl SandboxProvider for InProcessSandboxProvider {
    fn name(&self) -> &str {
        "in-process"
    }

    fn secure(&self) -> bool {
        false
    }

    fn open(&self, spec: &SandboxSpec) -> Result<Box<dyn SandboxInstance>, SandboxError> {
        spec.validate()?;
        let session_spec = SessionSpec::new(
            SessionId::parse("SANDBOX")
                .map_err(|error| SandboxError::new(SandboxErrorCode::InvalidSpec, error))?,
            SessionAuthorityPolicy::ZERO,
        );
        let mut runtime = Runtime::new();
        runtime.use_namespace(&spec.entry_namespace);
        Ok(Box::new(InProcessSandbox {
            session: Session::open(session_spec, runtime),
            limits: spec.limits.clone(),
            state: SandboxState::Open,
        }))
    }
}

struct InProcessSandbox {
    session: Session,
    limits: SandboxLimits,
    state: SandboxState,
}

impl InProcessSandbox {
    fn ensure_open(&self) -> Result<(), SandboxError> {
        if self.state == SandboxState::Closed {
            Err(SandboxError::new(
                SandboxErrorCode::Closed,
                "sandbox is closed",
            ))
        } else if self.state == SandboxState::Running {
            Err(SandboxError::new(SandboxErrorCode::Busy, "sandbox is busy"))
        } else {
            Ok(())
        }
    }

    fn evaluate(&mut self, source: &str) -> Result<String, SandboxError> {
        self.ensure_open()?;
        if source.len() > self.limits.source_bytes {
            return Err(SandboxError::new(
                SandboxErrorCode::LimitExceeded,
                "sandbox source limit exceeded",
            ));
        }
        self.state = SandboxState::Running;
        match self.session.eval(source) {
            Ok(result) if result.len() <= self.limits.result_bytes => {
                self.state = SandboxState::Open;
                Ok(result)
            }
            Ok(_) => {
                self.state = SandboxState::Failed;
                Err(SandboxError::new(
                    SandboxErrorCode::LimitExceeded,
                    "sandbox result limit exceeded",
                ))
            }
            Err(error) => {
                self.state = SandboxState::Failed;
                Err(SandboxError::new(SandboxErrorCode::EvaluationFailed, error))
            }
        }
    }
}

impl SandboxInstance for InProcessSandbox {
    fn eval(&mut self, source: &str) -> Result<String, SandboxError> {
        self.evaluate(source)
    }

    fn call(&mut self, callable: &str, arguments_hta: &[u8]) -> Result<Vec<u8>, SandboxError> {
        self.ensure_open()?;
        self.state = SandboxState::Running;
        match self
            .session
            .runtime_mut()
            .map_err(|error| SandboxError::new(SandboxErrorCode::EvaluationFailed, error))?
            .invoke_hta(callable, arguments_hta)
        {
            Ok(result) if result.len() <= self.limits.result_bytes => {
                self.state = SandboxState::Open;
                Ok(result)
            }
            Ok(_) => {
                self.state = SandboxState::Failed;
                Err(SandboxError::new(
                    SandboxErrorCode::LimitExceeded,
                    "sandbox result limit exceeded",
                ))
            }
            Err(error) => {
                self.state = SandboxState::Failed;
                Err(SandboxError::new(
                    SandboxErrorCode::EvaluationFailed,
                    error.to_string(),
                ))
            }
        }
    }

    fn cancel(&mut self) -> Result<bool, SandboxError> {
        self.ensure_open()?;
        // The conformance provider evaluates synchronously. It can record a
        // cancellation between calls but cannot preempt host execution.
        self.state = SandboxState::Cancelled;
        Ok(false)
    }

    fn state(&self) -> SandboxState {
        self.state
    }

    fn close(&mut self) -> Result<(), SandboxError> {
        if self.state != SandboxState::Closed {
            self.session.release();
            self.state = SandboxState::Closed;
        }
        Ok(())
    }
}

struct Sandbox {
    id: SandboxId,
    provider: String,
    instance: Box<dyn SandboxInstance>,
}

impl SessionKernel {
    pub fn register_sandbox_provider(&mut self, provider: Rc<dyn SandboxProvider>) {
        self.sandbox_provider_registry
            .entries
            .insert(provider.name().into(), provider);
    }

    pub fn open_sandbox(&mut self, spec: SandboxSpec) -> Result<SandboxId, SandboxError> {
        spec.validate()?;
        let provider = self
            .sandbox_provider_registry
            .entries
            .get(&spec.provider)
            .ok_or_else(|| {
                SandboxError::new(SandboxErrorCode::ProviderNotFound, spec.provider.clone())
            })?;
        let instance = provider.open(&spec)?;
        let id = SandboxId(self.sandbox_registry.next_id);
        self.sandbox_registry.next_id = self
            .sandbox_registry
            .next_id
            .checked_add(1)
            .expect("sandbox identifiers exhausted");
        self.sandbox_registry.entries.insert(
            id.get(),
            Sandbox {
                id,
                provider: spec.provider,
                instance,
            },
        );
        Ok(id)
    }

    fn sandbox_mut(&mut self, id: SandboxId) -> Result<&mut Sandbox, SandboxError> {
        self.sandbox_registry
            .entries
            .get_mut(&id.get())
            .ok_or_else(|| SandboxError::new(SandboxErrorCode::NotFound, id.to_string()))
    }

    pub fn sandbox_eval(&mut self, id: SandboxId, source: &str) -> Result<String, SandboxError> {
        self.sandbox_mut(id)?.instance.eval(source)
    }

    pub fn sandbox_call(
        &mut self,
        id: SandboxId,
        callable: &str,
        arguments_hta: &[u8],
    ) -> Result<Vec<u8>, SandboxError> {
        self.sandbox_mut(id)?.instance.call(callable, arguments_hta)
    }

    pub fn cancel_sandbox(&mut self, id: SandboxId) -> Result<bool, SandboxError> {
        self.sandbox_mut(id)?.instance.cancel()
    }

    pub fn sandbox_status(&self, id: SandboxId) -> Result<SandboxStatus, SandboxError> {
        let sandbox = self
            .sandbox_registry
            .entries
            .get(&id.get())
            .ok_or_else(|| SandboxError::new(SandboxErrorCode::NotFound, id.to_string()))?;
        Ok(SandboxStatus {
            id: sandbox.id,
            provider: sandbox.provider.clone(),
            state: sandbox.instance.state(),
        })
    }

    pub fn close_sandbox(&mut self, id: SandboxId) -> Result<(), SandboxError> {
        let mut sandbox = self
            .sandbox_registry
            .entries
            .remove(&id.get())
            .ok_or_else(|| SandboxError::new(SandboxErrorCode::NotFound, id.to_string()))?;
        sandbox.instance.close()
    }
}
