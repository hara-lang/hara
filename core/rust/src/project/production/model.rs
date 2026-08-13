#[path = "model/analysis.rs"]
mod analysis;
#[path = "model/kind.rs"]
mod kind;
#[path = "model/location.rs"]
mod location;
#[path = "model/unit.rs"]
mod unit;

pub use analysis::{Analysis, AnalysisOutput, RetentionReason};
pub use kind::{Effect, UnitKind};
pub use location::{Diagnostic, SourceLocation};
pub use unit::{ModuleAnalysis, UnitAnalysis};

pub const ANALYSIS_FORMAT: &str = "hara.production-analysis/0-alpha";
pub const SHAKE_FORMAT: &str = "hara.production-shake/0-alpha";
