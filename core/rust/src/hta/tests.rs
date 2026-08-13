use super::*;
#[test]
fn canonical_round_trip() {
    let value = Value::Map(
        vec![
            (Value::Keyword("b".into()), Value::Number(2)),
            (
                Value::Keyword("a".into()),
                Value::Vector(PVector::from(vec![Value::Bool(true), Value::Nil])),
            ),
        ]
        .into_iter()
        .collect(),
    );
    let encoded = encode(&value).unwrap();
    assert_eq!(encode(&decode(&encoded).unwrap()).unwrap(), encoded);
}
#[test]
fn compact_tuple_preserves_its_portable_identity() {
    let tuple = Value::Tuple(Box::new(
        PTuple::from_values(vec![Value::Number(1), Value::Number(2)]).unwrap(),
    ));
    let decoded = decode(&encode(&tuple).unwrap()).unwrap();
    assert!(matches!(decoded, Value::Tuple(_)));
}

#[test]
fn immutable_v3_values_round_trip_without_collection_normalization() {
    let queue = Value::Queue(Box::new(
        vec![Value::Number(1), Value::Number(2)]
            .into_iter()
            .collect(),
    ));
    assert!(matches!(
        decode(&encode(&queue).unwrap()).unwrap(),
        Value::Queue(_)
    ));
    let tagged = Value::Tagged(Box::new(crate::lang::data::TaggedLiteral::new(
        crate::lang::data::Symbol::parse("demo/tag"),
        Value::Number(42),
    )));
    assert!(matches!(
        decode(&encode(&tagged).unwrap()).unwrap(),
        Value::Tagged(_)
    ));
}
#[test]
fn floats_round_trip_with_ieee_754_bits() {
    for value in [0.28, -0.0, f64::INFINITY, f64::NEG_INFINITY] {
        let decoded = decode(&encode(&Value::Float(value)).unwrap()).unwrap();
        let Value::Float(decoded) = decoded else {
            panic!("float value")
        };
        assert_eq!(decoded.to_bits(), value.to_bits());
    }
}

#[test]
fn portable_language_scalars_round_trip() {
    for value in [
        Value::Character('雪'),
        Value::BigInteger("123456789012345678901234567890".into()),
        Value::Decimal("1.2500".into()),
        Value::Regex("^[a-z]+$".into()),
    ] {
        assert_eq!(decode(&encode(&value).unwrap()).unwrap(), value);
    }
}
#[test]
fn canonical_maps_ignore_insertion_order() {
    let a = Value::Map(
        vec![
            (Value::String("b".into()), Value::Number(2)),
            (Value::String("a".into()), Value::Number(1)),
        ]
        .into_iter()
        .collect(),
    );
    let b = Value::Map(
        vec![
            (Value::String("a".into()), Value::Number(1)),
            (Value::String("b".into()), Value::Number(2)),
        ]
        .into_iter()
        .collect(),
    );
    assert_eq!(encode(&a).unwrap(), encode(&b).unwrap());
}
#[test]
fn namespaces_and_vars_round_trip_as_snapshots() {
    let namespace = crate::kernel::Namespace::new("example.lib");
    let var = namespace.intern("answer", Value::Number(42));
    let value = Value::Map(
        vec![
            (
                Value::Keyword("namespace".into()),
                Value::Namespace(std::rc::Rc::new(namespace)),
            ),
            (Value::Keyword("var".into()), Value::Var(var)),
        ]
        .into_iter()
        .collect(),
    );
    let decoded = decode(&encode(&value).unwrap()).unwrap();
    let Value::Map(decoded) = decoded else {
        panic!("map snapshot")
    };
    let Value::Namespace(namespace) = decoded.get(&Value::Keyword("namespace".into())).unwrap()
    else {
        panic!("namespace snapshot")
    };
    assert_eq!(namespace.name().as_str(), "example.lib");
    let Value::Var(var) = decoded.get(&Value::Keyword("var".into())).unwrap() else {
        panic!("var snapshot")
    };
    assert_eq!(var.symbol().as_str(), "example.lib/answer");
    assert_eq!(var.deref_value(), Value::Number(42));
}

#[test]
fn opaque_handles_round_trip() {
    let value = Value::Extension(crate::core::ExtensionValue {
        provider: "runtime".into(),
        type_name: "cursor".into(),
        handle: 42,
    });
    assert_eq!(decode(&encode(&value).unwrap()).unwrap(), value);
}

#[test]
fn structs_preserve_wire_shape_and_mutables_are_rejected() {
    let ty = std::rc::Rc::new(crate::core::StructType {
        name: "demo/Point".into(),
        fields: vec!["x".into(), "y".into()],
    });
    let value = Value::Struct(std::rc::Rc::new(
        crate::core::StructValue::from_values(ty, vec![Value::Number(1), Value::Number(2)], None)
            .unwrap(),
    ));
    let decoded = decode(&encode(&value).unwrap()).unwrap();
    let Value::Struct(decoded) = decoded else {
        panic!("struct value")
    };
    assert_eq!(decoded.ty.name, "demo/Point");
    assert_eq!(decoded.ty.fields, vec!["x", "y"]);
    assert_eq!(
        decoded
            .ordered_values()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>(),
        vec![Value::Number(1), Value::Number(2)]
    );

    let mutable = Value::Mutable(std::rc::Rc::new(
        crate::core::MutableValue::from_values(
            std::rc::Rc::new(crate::core::MutableType {
                name: "demo/Cursor".into(),
                fields: vec!["x".into()],
            }),
            vec![Value::Number(1)],
            None,
        )
        .unwrap(),
    ));
    assert_eq!(
        encode(&mutable).unwrap_err(),
        "hta/value-unsupported: mutable values are not serializable; use (into {} value)"
    );
}

#[test]
fn nesting_depth_is_bounded_on_encode_and_decode() {
    let mut value = Value::Nil;
    for _ in 0..=MAX_NESTING_DEPTH {
        value = Value::Vector(PVector::from(vec![value]));
    }
    assert!(encode(&value).unwrap_err().contains("value-too-deep"));

    let mut bytes = MAGIC.to_vec();
    for _ in 0..=MAX_NESTING_DEPTH {
        bytes.extend_from_slice(&[VECTOR, 0, 0, 0, 1]);
    }
    bytes.push(NIL);
    assert!(decode(&bytes).unwrap_err().contains("value-too-deep"));
}

#[test]
fn impossible_container_lengths_fail_before_allocating() {
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&[VECTOR, 0xff, 0xff, 0xff, 0xff]);
    assert!(decode(&bytes)
        .unwrap_err()
        .contains("impossible sequence length"));
}
