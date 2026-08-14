use super::super::source::{Diagnostic, SourceLocation};
use super::{Effect, UnitKind};
use std::collections::BTreeSet;

/// Canonical native/runtime roots discovered while compiling one expanded
/// definition unit. The compatibility projections on [`UnitAnalysis`] remain
/// until the target generators consume this typed inventory directly.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NativeRootInventory {
    pub primitives: BTreeSet<String>,
    pub methods: BTreeSet<String>,
    pub dynamic_methods: BTreeSet<String>,
    pub types: BTreeSet<String>,
    pub protocols: BTreeSet<String>,
    pub protocol_methods: BTreeSet<String>,
    pub multimethods: BTreeSet<String>,
    pub host_calls: BTreeSet<String>,
    pub callbacks: BTreeSet<String>,
    pub runtime_shims: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitAnalysis {
    pub id: String,
    pub module: String,
    pub index: usize,
    /// Deterministic source for the macro-expanded top-level unit that was
    /// analyzed. Production emission compiles this exact source instead of
    /// reparsing or re-expanding the complete module.
    pub form_source: String,
    pub kind: UnitKind,
    pub effect: Effect,
    pub location: SourceLocation,
    pub provides: BTreeSet<String>,
    pub runtime_edges: BTreeSet<String>,
    pub compile_time_edges: BTreeSet<String>,
    pub namespace_edges: BTreeSet<String>,
    /// Typed root contract consumed by #553 runtime specialization.
    pub native_roots: NativeRootInventory,
    /// Compatibility projections retained for the existing 0-alpha report.
    pub native_primitives: BTreeSet<String>,
    pub native_types: BTreeSet<String>,
    pub native_protocols: BTreeSet<String>,
    pub diagnostics: Vec<Diagnostic>,
}
