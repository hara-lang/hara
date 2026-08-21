#[test]
fn bootstrap_direct_callable_catalog_extends_the_closed_inventory() {
    validate_direct_callable_catalog().unwrap();
    assert_eq!(
        direct_callable_catalog().count(),
        runtime_callable_inventory().count()
    );
    for specification in DIRECT_CALLABLE_BOOTSTRAP_CATALOG {
        assert!(
            direct_callable_value(specification.symbol).is_some(),
            "missing direct bootstrap callable value for {}",
            specification.symbol
        );
    }
}

#[test]
fn bootstrap_direct_callables_never_reenter_the_evaluator() {
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
                                    for specification in DIRECT_CALLABLE_BOOTSTRAP_CATALOG {
                                        let callable = direct_callable_value(specification.symbol)
                                            .unwrap_or_else(|| {
                                                panic!(
                                                    "missing direct bootstrap callable value for {}",
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
fn bootstrap_direct_callables_preserve_predicate_and_promise_behavior() {
    let boolean_predicate = direct_callable_value("boolean?").unwrap();
    assert_eq!(
        call_value(boolean_predicate.clone(), vec![Value::Bool(true)]).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        call_value(boolean_predicate, vec![Value::Number(1)]).unwrap(),
        Value::Bool(false)
    );

    let runtime = crate::Runtime::empty();
    with_promise_provider(runtime.providers.promise(), || {
        let constructor = direct_callable_value("promise/new").unwrap();
        let executor = native_function("resolve-immediately", 2, |arguments| {
            call_value(arguments[0].clone(), vec![Value::Number(7)])
        });
        assert!(matches!(
            call_value(constructor, vec![executor]).unwrap(),
            Value::Promise(_)
        ));

        let delay = direct_callable_value("promise/delay").unwrap();
        let delayed = native_function("delayed-value", 0, |_| Ok(Value::Number(9)));
        assert!(matches!(
            call_value(delay, vec![Value::Number(0), delayed]).unwrap(),
            Value::Promise(_)
        ));
    });
}

#[test]
#[ignore = "requires the authoritative hara-specs-registry checkout"]
fn specs_owned_direct_callable_bootstrap_fixture_runs_before_foundation_source_loading() {
    let registry = std::env::var_os("HARA_SPECS_REGISTRY")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("..")
                .join("hara-specs-registry")
        });
    let path = registry.join(
        "01-lang/004-foundation/draft/conformance/fixtures/direct_callable_bootstrap.hal",
    );
    let source = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "authoritative direct-callable bootstrap fixture is required at {}: {error}",
            path.display()
        )
    });

    let mut runtime = crate::Runtime::core();
    let report = runtime
        .eval_text(&(source + "\n(direct-callable-bootstrap-report)"))
        .unwrap();
    assert_eq!(
        report,
        "[true false :std.native.Promise :std.native.Promise]"
    );
}
