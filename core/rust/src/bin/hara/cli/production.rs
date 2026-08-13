use super::Options;
use hara_wasm::project;
use hara_wasm::vm::production;
use std::path::PathBuf;

pub(crate) fn analyze(options: &Options, arguments: &[String]) -> Result<(), String> {
    if arguments.len() != 1 {
        return Err("project production analysis requires one serialized build plan".into());
    }
    let start = options
        .project
        .clone()
        .or_else(|| options.root.clone())
        .unwrap_or_else(|| PathBuf::from("."));
    let project = project::discover(&start)?;
    let output = production::analyze_and_write(&project, &arguments[0])?;
    println!("project production analysis: {}", output.report_path.display());
    if output.analysis.succeeded() {
        Err(format!(
            "unavailable: production reachability analysis completed; pruned HBX emission is tracked by #552; report: {}",
            output.report_path.display()
        ))
    } else {
        Err(format!(
            "production reachability analysis failed with {} diagnostic(s); report: {}",
            output.analysis.diagnostics.len(),
            output.report_path.display()
        ))
    }
}
