use super::*;
use crate::vm::{compile_source, disassemble, execute_program};

#[test]
fn programs_round_trip_and_execute() {
    let source = "(do (defn add-one [x] (+ x 1)) (add-one 41))";
    let mut program = compile_source(source).unwrap();
    program.namespace = Some("demo".into());
    program.schema_types.insert(
        "demo/Customer".into(),
        SchemaType::Map(vec![SchemaField {
            name: crate::kernel::parse(":id").unwrap(),
            value_type: SchemaType::Primitive("int".into()),
        }]),
    );
    program.function_types.insert(
        "demo/add-one".into(),
        SchemaType::Function(vec![FunctionSchema {
            fixed: vec![SchemaType::Primitive("int".into())],
            rest: None,
            output: Box::new(SchemaType::Primitive("int".into())),
        }]),
    );
    program.inferred_function_types.insert(
        "demo/inferred".into(),
        SchemaType::Function(vec![FunctionSchema {
            fixed: vec![],
            rest: None,
            output: Box::new(SchemaType::Primitive("int".into())),
        }]),
    );
    let encoded = encode_program(&program).unwrap();
    assert!(encoded.starts_with(b"HBC0"));
    let decoded = decode_program(&encoded).unwrap();
    assert_eq!(disassemble(&decoded), disassemble(&program));
    assert_eq!(decoded.schema_types, program.schema_types);
    assert_eq!(decoded.function_types, program.function_types);
    assert_eq!(
        decoded.inferred_function_types,
        program.inferred_function_types
    );
    assert_eq!(decoded.namespace, program.namespace);
    assert_eq!(
        execute_program(Rc::new(decoded)).unwrap(),
        Value::Number(42)
    );
}

#[test]
fn corruption_is_rejected_before_decode() {
    let program = compile_source("(+ 19 23)").unwrap();
    let mut encoded = encode_program(&program).unwrap();
    encoded[12] ^= 1;
    assert_eq!(
        decode_program(&encoded).unwrap_err(),
        "bytecode artifact checksum mismatch"
    );
}
