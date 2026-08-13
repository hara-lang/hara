#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BundleSummary {
    pub output_bytes: usize,
    pub output_digest: String,
    pub module_count: usize,
}
