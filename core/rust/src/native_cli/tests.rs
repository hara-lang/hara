use super::{DocumentationValue, RuntimeBroker};

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
