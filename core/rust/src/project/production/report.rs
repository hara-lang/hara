#[path = "report/analysis.rs"]
mod analysis;
#[path = "report/bundle.rs"]
mod bundle;
#[path = "report/document.rs"]
mod document;
#[path = "report/form.rs"]
mod form;

pub const ANALYSIS_FORMAT: &str = "hara.production-analysis/0-alpha";
pub const SHAKE_FORMAT: &str = "hara.production-shake/0-alpha";

pub(super) use bundle::BundleSummary;

use super::graph::Analysis;
use super::plan::BuildPlan;

pub fn report_source(
    plan: &BuildPlan,
    analysis: &Analysis,
    output: Option<&BundleSummary>,
) -> String {
    format!("{}\n", document::report_form(plan, analysis, output))
}
