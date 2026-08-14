use crate::Runtime;

#[test]
fn bytecode_uses_representation_independent_numeric_predicates() {
    let mut runtime = Runtime::core();
    runtime.prepare_foundation_bytecode();
    assert_eq!(
        runtime.eval_bytecode_native(
            "[(long? 42) (long? (double 1.5)) (double? (double 1.5)) (double? 42) \
              (number? 42) (number? 1.5) (integer? 9223372036854775808) \
              (decimal? 1.5) (integer? 1.5)]",
        ),
        Ok("[true false true false true true true true false]".into()),
    );
}
