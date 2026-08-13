use super::super::source::{Diagnostic, SourceLocation};
use super::{Effect, UnitKind};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitAnalysis {
    pub id: String,
    pub module: String,
    pub index: usize,
    pub kind: UnitKind,
    pub effect: Effect,
    pub location: SourceLocation,
    pub provides: BTreeSet<String>,
    pub runtime_edges: BTreeSet<String>,
    pub compile_time_edges: BTreeSet<String>,
    pub namespace_edges: BTreeSet<String>,
    pub native_primitives: BTreeSet<String>,
    pub native_types: BTreeSet<String>,
    pub native_protocols: BTreeSet<String>,
    pub diagnostics: Vec<Diagnostic>,
}