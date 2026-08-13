#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliOutcome {
    Success,
    Failed,
    UsageError,
    ReadError,
    ResolutionError,
    Unavailable,
    InternalError,
    Interrupted,
}

impl CliOutcome {
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::Failed => 1,
            Self::UsageError
            | Self::ReadError
            | Self::ResolutionError
            | Self::Unavailable
            | Self::InternalError => 2,
            Self::Interrupted => 130,
        }
    }
}
