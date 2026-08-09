use crate::Runtime;

#[test]
fn bytecode_uses_the_explicit_long_and_double_predicates() {
    let mut runtime = Runtime::core();
    runtime.prepare_foundation_bytecode();
    assert_eq!(
        runtime.eval_bytecode_native(
            "[(long? 42) (long? 1.5) (double? 1.5) (double? 42) \
              (number? 42) (number? 1.5)]",
        ),
        Ok("[true false true false true true]".into()),
    );
    for unsupported in ["(integer? 42)", "(decimal? 1.5)"] {
        let error = runtime
            .compile_bytecode(unsupported)
            .expect_err("unsupported numeric predicate must not compile");
        assert!(error.contains("unbound symbol"), "{unsupported}: {error}");
    }
}
