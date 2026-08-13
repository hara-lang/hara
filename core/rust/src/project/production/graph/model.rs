use super::super::source::{Diagnostic, SourceLocation};
use super::UnitAnalysis;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleAnalysis {
    pub name: String,
    pub path: String,
    pub digest: String,
    pub input_bytes: usize,
    pub dependencies: Vec<String>,
    pub unit_ids: Vec<String>,
    pub standard_library: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionReason {
    pub unit_id: String,
    pub subject: Option<String>,
    pub code: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis {
    pub modules: Vec<ModuleAnalysis>,
    pub units: Vec<UnitAnalysis>,
    pub runtime_roots: BTreeSet<String>,
    pub runtime_closure: BTreeSet<String>,
    pub compile_time_roots: BTreeSet<String>,
    pub compile_time_closure: BTreeSet<String>,
    pub retained_unit_ids: BTreeSet<String>,
    pub removed_unit_ids: BTreeSet<String>,
    pub retained_vars: BTreeSet<String>,
    pub removed_vars: BTreeSet<String>,
    pub retained_namespaces: BTreeSet<String>,
    pub removed_namespaces: BTreeSet<String>,
    pub reasons: Vec<RetentionReason>,
    pub diagnostics: Vec<Diagnostic>,
    pub native_primitives: BTreeSet<String>,
    pub native_types: BTreeSet<String>,
    pub native_protocols: BTreeSet<String>,
    pub input_bytes: usize,
    pub input_digest: String,
}

impl Analysis {
    pub fn succeeded(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisOutput {
    pub analysis: Analysis,
    pub report_path: std::path::PathBuf,
    pub report_source: String,
}

pub(super) fn project_location() -> SourceLocation {
    SourceLocation {
        path: "project.edn".into(),
        line: 1,
        column: 1,
        end_line: 1,
        end_column: 1,
    }
}
