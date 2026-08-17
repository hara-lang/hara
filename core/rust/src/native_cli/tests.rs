use super::{install_native_kernel, DocumentationValue, RuntimeBroker};

#[test]
fn native_sandbox_surface_uses_the_broker_kernel() {
    let broker = RuntimeBroker::start_core().unwrap();
    let mut runtime = crate::Runtime::core();
    install_native_kernel(&mut runtime, broker);
    let sandbox = runtime
        .eval_native(
            "(deref (Sandbox/open {:protocol \"hara.sandbox/0-alpha\" :provider :in-process :runtime \"hara.standard/0-alpha\" :entry-namespace \"user\"}))",
        )
        .unwrap();
    assert_eq!(sandbox, "1");
    assert_eq!(
        runtime
            .eval_native("(deref (Sandbox/eval 1 \"(+ 40 2)\"))")
            .unwrap(),
        "42"
    );
    assert_eq!(
        runtime
            .eval_native("(deref (Sandbox/call 1 \"std.foundation/+\" [1 2 3]))")
            .unwrap(),
        "6"
    );
    assert_eq!(
        runtime.eval_native("(:secure (Sandbox/status 1))").unwrap(),
        "false"
    );
    assert_eq!(
        runtime.eval_native("(deref (Sandbox/close 1))").unwrap(),
        "nil"
    );
}

#[test]
fn sessions_are_isolated_and_root_is_persistent() {
    let broker = RuntimeBroker::start().unwrap();
    assert_eq!(
        broker.eval("ROOT", "(def answer 42)").unwrap(),
        "#'user/answer"
    );
    broker.create("APP").unwrap();
    assert!(broker
        .eval("APP", "answer")
        .unwrap_err()
        .contains("unbound"));
    assert_eq!(broker.eval("ROOT", "answer").unwrap(), "42");
    assert_eq!(broker.list().unwrap(), vec!["APP", "ROOT"]);
    broker.close("APP").unwrap();
    assert!(broker.close("ROOT").is_err());
}

#[test]
fn documentation_preserves_runtime_metadata() {
    let broker = RuntimeBroker::start().unwrap();
    broker
        .eval(
            "ROOT",
            concat!(
                "(defn ^{:file \"/tmp/sample.hal\" :line 12 :column 3} located ",
                "\"A located function.\" [value] value)"
            ),
        )
        .unwrap();
    let documentation = broker.documentation("ROOT", "located").unwrap();
    assert_eq!(documentation.symbol, "located");
    assert_eq!(documentation.doc.as_deref(), Some("A located function."));
    assert_eq!(documentation.file.as_deref(), Some("/tmp/sample.hal"));
    assert_eq!(documentation.line, Some(12));
    assert_eq!(documentation.column, Some(3));
    assert_eq!(
        documentation.arglists,
        DocumentationValue::Array(vec![DocumentationValue::Array(vec![
            DocumentationValue::String("value".into())
        ])])
    );
    assert!(broker.documentation("ROOT", "missing").is_err());
}

#[test]
fn development_resources_are_owned_by_the_kernel_and_seed_future_sessions() {
    let broker = RuntimeBroker::start().unwrap();
    broker
        .register_resource("demo.value", "(ns demo.value) (def answer 42)")
        .unwrap();
    assert_eq!(broker.resources().unwrap(), vec!["demo.value"]);

    broker.create("APP").unwrap();
    assert_eq!(
        broker
            .eval("APP", "(require [demo.value]) demo.value/answer")
            .unwrap(),
        "42"
    );

    broker.remove_resource("demo.value").unwrap();
    assert!(broker.resources().unwrap().is_empty());
}
