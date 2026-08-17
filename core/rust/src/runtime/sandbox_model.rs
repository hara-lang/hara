use std::error::Error;

pub const SANDBOX_SPEC_PROTOCOL: &str = "hara.sandbox/0-alpha";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SandboxId(u64);

impl SandboxId {
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EvaluationId(u64);

impl EvaluationId {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Display for SandboxId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxState {
    Open,
    Running,
    Cancelling,
    Cancelled,
    Failed,
    Closed,
}

impl SandboxState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Running => "running",
            Self::Cancelling => "cancelling",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
            Self::Closed => "closed",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxLimits {
    pub source_bytes: usize,
    pub result_bytes: usize,
    pub output_bytes: usize,
    pub evaluation_ms: u64,
    pub memory_bytes: usize,
    pub active_evaluations: usize,
}

impl Default for SandboxLimits {
    fn default() -> Self {
        Self {
            source_bytes: 64 * 1024,
            result_bytes: 1024 * 1024,
            output_bytes: 1024 * 1024,
            evaluation_ms: 5_000,
            memory_bytes: 64 * 1024 * 1024,
            active_evaluations: 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxSpec {
    protocol: String,
    provider: String,
    runtime: String,
    entry_namespace: String,
    limits: SandboxLimits,
}

impl SandboxSpec {
    pub fn new(
        protocol: impl Into<String>,
        provider: impl Into<String>,
        runtime: impl Into<String>,
        entry_namespace: impl Into<String>,
        limits: SandboxLimits,
    ) -> Result<Self, SandboxError> {
        let spec = Self {
            protocol: protocol.into(),
            provider: provider.into(),
            runtime: runtime.into(),
            entry_namespace: entry_namespace.into(),
            limits,
        };
        spec.validate()?;
        Ok(spec)
    }

    pub fn in_process() -> Self {
        Self {
            protocol: SANDBOX_SPEC_PROTOCOL.into(),
            provider: "in-process".into(),
            runtime: "hara.standard/0-alpha".into(),
            entry_namespace: "user".into(),
            limits: SandboxLimits::default(),
        }
    }

    pub fn validate(&self) -> Result<(), SandboxError> {
        if self.protocol != SANDBOX_SPEC_PROTOCOL {
            return Err(SandboxError::invalid_spec("unsupported sandbox protocol"));
        }
        if self.provider.is_empty() || self.runtime.is_empty() {
            return Err(SandboxError::invalid_spec(
                "provider and runtime are required",
            ));
        }
        SessionId::parse(&self.entry_namespace)
            .map_err(|_| SandboxError::invalid_spec("invalid entry namespace"))?;
        if self.limits.source_bytes == 0
            || self.limits.result_bytes == 0
            || self.limits.output_bytes == 0
            || self.limits.evaluation_ms == 0
            || self.limits.memory_bytes == 0
            || self.limits.active_evaluations != 1
        {
            return Err(SandboxError::invalid_spec("invalid sandbox limits"));
        }
        Ok(())
    }

    pub fn provider(&self) -> &str {
        &self.provider
    }

    pub fn entry_namespace(&self) -> &str {
        &self.entry_namespace
    }

    pub fn limits(&self) -> &SandboxLimits {
        &self.limits
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxStatus {
    pub id: SandboxId,
    pub provider: String,
    pub state: SandboxState,
    pub secure: bool,
    pub evaluation_active: bool,
    pub error: Option<SandboxError>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxErrorCode {
    InvalidSpec,
    ProviderNotFound,
    ProviderUnavailable,
    BundleNotFound,
    BundleDigestMismatch,
    MountNotFound,
    NotFound,
    Closed,
    Busy,
    Cancelled,
    Timeout,
    LimitExceeded,
    EvaluationFailed,
    ResultNotTransferable,
    TransportFailed,
    ProviderFailed,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxError {
    pub code: SandboxErrorCode,
    pub message: String,
}

impl SandboxError {
    fn new(code: SandboxErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    fn invalid_spec(message: impl Into<String>) -> Self {
        Self::new(SandboxErrorCode::InvalidSpec, message)
    }
}

impl fmt::Display for SandboxError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.message)
    }
}

impl Error for SandboxError {}
