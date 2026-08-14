#![cfg(feature = "bytecode-vm")]

use hara_wasm::Runtime;

fn eval(runtime: &mut Runtime, source: &str) -> String {
    runtime
        .eval_bytecode_native(source)
        .unwrap_or_else(|error| panic!("bytecode evaluation failed for {source}: {error}"))
}

#[test]
fn public_runtime_covers_globals_arities_destructuring_and_named_values() {
    let mut runtime = Runtime::core();

    assert_eq!(
        eval(
            &mut runtime,
            "(do (def answer 1) (set! answer 42) [answer (var answer)])",
        ),
        "[42 #'user/answer]"
    );

    assert_eq!(
        eval(
            &mut runtime,
            "(do (defn choose ([x] x) ([x y] y)) [(choose 1) (choose 1 2)])",
        ),
        "[1 2]"
    );
    assert_eq!(
        eval(
            &mut runtime,
            "((fn [head & tail] [head tail]) 1 2 3)",
        ),
        "[1 (2 3)]"
    );
    assert_eq!(
        eval(&mut runtime, "(let [[left right] [19 23]] (+ left right))"),
        "42"
    );

    assert_eq!(
        eval(
            &mut runtime,
            "(do (defstruct Point [x y]) [(:x (Point 19 23)) (instance? Point (Point 0 0))])",
        ),
        "[19 true]"
    );
    assert_eq!(
        eval(
            &mut runtime,
            "(do (defmutable Cursor [x]) (let [cursor (Cursor 1)] (set! (field cursor :x) 42) (field cursor :x)))",
        ),
        "42"
    );
}

#[test]
fn namespace_and_module_forms_stay_at_the_runtime_boundary() {
    let runtime = Runtime::core();
    for (source, operator) in [("(ns demo)", "ns"), ("(require demo)", "require")] {
        let error = runtime
            .compile_bytecode(source)
            .expect_err("namespace forms must not become VM instructions");
        assert!(
            error.contains(&format!("unsupported operator: {operator}")),
            "unexpected {operator} diagnostic: {error}"
        );
    }
}

#[test]
fn named_multi_arity_is_supported_but_anonymous_multi_arity_is_not_language_surface() {
    let runtime = Runtime::core();
    let error = runtime
        .compile_bytecode("(fn ([x] x) ([x y] y))")
        .expect_err("anonymous multi-arity fn remains outside the portable evaluator surface");
    assert!(
        error.contains("fn multi-arity is not supported"),
        "unexpected anonymous multi-arity diagnostic: {error}"
    );
}
