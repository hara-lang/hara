use super::super::{graph::Analysis, plan::BuildPlan};
use crate::vm::BytecodeBundleModule;

#[derive(Debug, Clone)]
pub(super) struct ProductionBuild {
    pub(super) plan: BuildPlan,
    pub(super) analysis: Analysis,
}

#[derive(Clone)]
pub(super) struct CompiledBundle {
    pub(super) bytes: Vec<u8>,
    pub(super) modules: Vec<BytecodeBundleModule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RenderedModule {
    pub(super) resource: String,
    pub(super) namespace_form: String,
    pub(super) body: String,
    pub(super) source: String,
    pub(super) dependencies: Vec<String>,
}
