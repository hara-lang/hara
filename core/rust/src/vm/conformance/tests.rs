use super::*;
use crate::journal::SCHEMA as JOURNAL_SCHEMA;
use crate::kernel::halc_trace::HALC_TRACE_SCHEMA;
use crate::vm::session::BYTECODE_TRACE_SCHEMA;

fn case<'a>(report: &'a ProductionReport, id: &str) -> &'a CaseObservation {
    report
        .cases
        .iter()
        .find(|case| case.case.id == id)
        .unwrap_or_else(|| panic!("missing case {id}"))
}

#[test]
fn embedded_production_corpus_passes_real_runtime_checks() {
    let report = run_embedded().expect("production corpus runs");
    assert!(report.passed(), "{} failed checks", report.failed_checks());
    assert_eq!(report.cases.len(), 14);
    assert!(report
        .cases
        .iter()
        .all(|case| case.journal.schema == JOURNAL_SCHEMA));
    assert!(report
        .cases
        .iter()
        .all(|case| case.halc_trace.schema == HALC_TRACE_SCHEMA));
}

#[test]
fn arithmetic_fixture_carries_all_three_production_traces() {
    let report = run_embedded().expect("production corpus runs");
    let arithmetic = case(&report, "arith/nested");
    assert_eq!(arithmetic.interpreter.display.as_deref(), Some("7"));
    assert_eq!(arithmetic.halc.handoff_status.as_deref(), Some("ready"));
    assert_eq!(arithmetic.bytecode.outcome.display.as_deref(), Some("7"));
    assert!(arithmetic.halc_trace.events.iter().any(|event| {
        event.stage == "handoff/bytecode"
    }));
    let trace_json = crate::json::write(&arithmetic.bytecode_trace).unwrap();
    assert!(trace_json.contains(BYTECODE_TRACE_SCHEMA));
    assert!(trace_json.contains("code.vm/arith/nested"));
    assert!(arithmetic
        .teaching
        .iter()
        .any(|annotation| annotation.stage == "bytecode"));
}

#[test]
fn deep_fixture_is_bounded_without_changing_its_result() {
    let report = run_embedded().expect("production corpus runs");
    let looping = case(&report, "loop/many-iterations");
    assert_eq!(looping.bytecode.outcome.display.as_deref(), Some("1024"));
    assert!(looping.bytecode.step_count <= looping.case.trace_limit);
    assert!(looping.bytecode.dropped > 0);
    assert!(looping
        .checks
        .iter()
        .find(|check| check.id == "trace/bounded")
        .is_some_and(|check| check.pass));
}

#[test]
fn unsupported_bytecode_form_is_typed_and_never_falls_back() {
    let report = run_embedded().expect("production corpus runs");
    let unsupported = case(&report, "compile/fn-multi-arity");
    assert_eq!(unsupported.bytecode.outcome.status, "compile-error");
    assert_eq!(
        unsupported.bytecode.outcome.category.as_deref(),
        Some("unsupported form")
    );
    assert_eq!(unsupported.halc.status, "error");
    assert_eq!(unsupported.halc.fallback, Some(false));
    assert!(unsupported
        .checks
        .iter()
        .find(|check| check.id == "fallback/forbidden")
        .is_some_and(|check| check.pass));
}

#[test]
fn reports_are_deterministic_and_browser_view_is_terminal_neutral() {
    let first = run_embedded().expect("first run");
    let second = run_embedded().expect("second run");
    assert_eq!(first.to_json(false).unwrap(), second.to_json(false).unwrap());

    let browser = first.browser_json(false).unwrap();
    assert!(browser.contains("\"view\":\"browser\""));
    assert!(browser.contains("\"terminalNeutral\":true"));
    assert!(browser.contains("\"browserSafe\":true"));
    assert!(browser.contains("\"supported\":false"));
    assert!(!browser.contains("loop/many-iterations"));
}
