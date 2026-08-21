#[test]
fn direct_callable_catalog_closes_the_runtime_inventory() {
    validate_direct_callable_catalog().unwrap();
    assert_eq!(
        DIRECT_CALLABLE_CATALOG.len(),
        RUNTIME_CALLABLE_INVENTORY.len()
    );
    assert!(
        DIRECT_CALLABLE_CATALOG.len() >= 150,
        "the complete ordinary callable catalog unexpectedly shrank"
    );
    for specification in DIRECT_CALLABLE_CATALOG {
        assert!(
            direct_callable_value(specification.symbol).is_some(),
            "missing direct callable value for {}",
            specification.symbol
        );
    }
}

#[test]
fn complete_ordinary_callable_catalog_never_reenters_the_evaluator() {
    let runtime = crate::Runtime::empty();
    with_test_runner(&runtime.test_runner, || {
        with_capability_providers(
            runtime.providers.file(),
            runtime.providers.socket(),
            runtime.providers.process(),
            runtime.providers.kernel(),
            || {
                with_package_catalog(&runtime.package_catalog, || {
                    with_promise_provider(runtime.providers.promise(), || {
                        with_macros(runtime.macros.clone(), || {
                            with_namespace_registry(&runtime.namespace_registry, || {
                                with_protocols(&runtime.protocols, || {
                                    for specification in DIRECT_CALLABLE_CATALOG
                                        .iter()
                                        .filter(|specification| specification.origin.ordinary())
                                    {
                                        let callable = direct_callable_value(specification.symbol)
                                            .unwrap_or_else(|| {
                                                panic!(
                                                    "missing direct callable value for {}",
                                                    specification.symbol
                                                )
                                            });
                                        let arguments =
                                            direct_callable_probe_arguments(specification);
                                        let (_, evaluator_invocations) =
                                            with_evaluator_invocation_count(|| {
                                                call_value(callable, arguments)
                                            });
                                        assert_eq!(
                                            evaluator_invocations, 0,
                                            "{} must dispatch directly at the value boundary",
                                            specification.symbol
                                        );
                                    }
                                })
                            })
                        })
                    })
                })
            },
        )
    });
}

#[test]
fn representative_direct_callables_preserve_value_behavior() {
    let count = direct_callable_value("count").unwrap();
    let result = call_value(
        count,
        vec![Value::Vector(
            vec![Value::Number(1), Value::Number(2)].into(),
        )],
    )
    .unwrap();
    assert_eq!(result, Value::Number(2));

    let increment = direct_callable_value("inc").unwrap();
    assert_eq!(
        call_value(increment, vec![Value::Number(41)]).unwrap(),
        Value::Number(42)
    );

    let ifn = foundation_protocol_values()
        .into_iter()
        .find_map(|(name, value)| (name == "IFn").then_some(value))
        .expect("the Foundation IFn protocol must be installed");
    let identity = direct_callable_value("identity").unwrap();
    let satisfies = direct_callable_value("satisfies?").unwrap();
    assert_eq!(
        call_value(satisfies, vec![ifn, identity]).unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn every_native_inventory_entry_builds_a_direct_value() {
    for (native_type, methods) in NATIVE_TYPES {
        for method in *methods {
            let (value, evaluator_invocations) =
                with_evaluator_invocation_count(|| native_type_function_value(native_type, method));
            value.unwrap_or_else(|error| panic!("{error}"));
            assert_eq!(
                evaluator_invocations, 0,
                "std.native.{native_type}/{method} construction must not re-enter eval"
            );
        }
    }
}
