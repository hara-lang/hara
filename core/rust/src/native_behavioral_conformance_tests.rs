use crate::{core, Runtime};
use std::collections::BTreeSet;
use std::path::Path;

const CORPUS: &str =
    include_str!("../../lib/test-fixtures/std/foundation/native_method_conformance.hal");

fn corpus_methods() -> BTreeSet<String> {
    let mut runtime = Runtime::new();
    let value = runtime
        .eval_native_value(&format!("{CORPUS}\n(native-method-keys)"))
        .expect("native corpus keys must evaluate");
    let core::Value::Vector(values) = value else {
        panic!("native-method-keys must return a vector");
    };
    let methods = values
        .iter()
        .map(core::Value::display)
        .collect::<BTreeSet<_>>();
    assert_eq!(values.len(), methods.len(), "duplicate native corpus method");
    assert!(!methods.is_empty(), "native corpus must not be empty");
    methods
}

fn live_methods() -> BTreeSet<String> {
    core::NATIVE_TYPES
        .iter()
        .flat_map(|(native_type, methods)| {
            methods
                .iter()
                .map(move |method| format!("{native_type}/{method}"))
        })
        .collect()
}

fn closure_error(
    live: &BTreeSet<String>,
    classified: &BTreeSet<String>,
) -> Result<(), String> {
    let missing = live.difference(classified).cloned().collect::<Vec<_>>();
    let extra = classified.difference(live).cloned().collect::<Vec<_>>();
    if missing.is_empty() && extra.is_empty() {
        Ok(())
    } else {
        Err(format!("missing={missing:?}, extra={extra:?}"))
    }
}

#[test]
fn source_owned_native_corpus_closes_over_live_inventory_and_rejects_drift() {
    let mut runtime = Runtime::new();
    assert_eq!(
        "true",
        runtime
            .eval_text(&format!("{CORPUS}\n(native-corpus-valid?)"))
            .expect("native corpus validation must evaluate")
    );
    eprintln!(
        "native behavioral classifications {}",
        runtime
            .eval_text(&format!("{CORPUS}\n(native-classification-summary)"))
            .expect("native classification summary must evaluate")
    );

    let classified = corpus_methods();
    let live = live_methods();
    closure_error(&live, &classified).expect("native behavioral corpus must exactly close");

    let first = classified
        .iter()
        .next()
        .expect("classified native method")
        .clone();

    let mut removed = classified.clone();
    removed.remove(&first);
    assert!(closure_error(&live, &removed).is_err());

    let mut added = classified.clone();
    added.insert("Unclassified/addition".to_owned());
    assert!(closure_error(&live, &added).is_err());

    let mut renamed = classified;
    renamed.remove(&first);
    renamed.insert(format!("{first}-renamed"));
    assert!(closure_error(&live, &renamed).is_err());

    let local_copy = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("hal-test-fixtures/std/foundation/native_method_conformance.hal");
    assert!(
        !local_copy.exists(),
        "the divergent Rust-local native fixture must not reappear"
    );
}

#[test]
fn evaluator_runs_every_classification_and_normalized_boundary_probe() {
    let methods = corpus_methods();
    let cases = methods
        .iter()
        .map(|method| format!("(native-method-result '{method} nil)"))
        .collect::<Vec<_>>()
        .join(" ");
    let source = format!("{CORPUS}\n[{cases}]");
    let mut runtime = Runtime::new();
    let result = runtime
        .eval_text(&source)
        .expect("shared native behavioral corpus must evaluate");
    assert!(!result.contains(":pass false"), "{result}");
    assert_eq!(methods.len(), result.matches(":pass true").count());

    let mut runtime = Runtime::new();
    let report = runtime
        .eval_text(&format!("{CORPUS}\n(native-boundary-report)"))
        .expect("portable native boundary report must evaluate");
    assert_eq!(
        "[true true true true true true true true true true true true]",
        report
    );
}

#[test]
fn evaluator_and_bytecode_agree_on_portable_native_calls() {
    const PROBE: &str = "[(Maths/abs -2) (Bits/and 6 3) (Num/long 4.0)]";
    let mut runtime = Runtime::new();
    let interpreted = runtime
        .eval_text(PROBE)
        .expect("evaluator native probe must run");
    let compiled = runtime
        .eval_bytecode_native(PROBE)
        .expect("bytecode native probe must run");
    assert_eq!(interpreted, compiled);
}

#[test]
fn rust_base_identity_fast_paths_remain_explicit() {
    let source = include_str!("core/protocol.rs");
    assert!(
        source.contains("[value @ Value::Vector(_)] => Ok(value.clone())"),
        "Base/vec must return an existing vector value without materialization"
    );
    assert!(
        source.contains(
            "value @ (Value::Set(_) | Value::OrderedSet(_) | Value::SortedSet(_))"
        ),
        "Base/set must return any existing persistent set without materialization"
    );
}
