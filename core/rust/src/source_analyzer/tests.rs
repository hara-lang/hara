use std::io::Cursor;

const TEST_ANALYZER: &str = r#"
  (defn ^{:schema [:fn [] :any]}
    describe
    []
    ["test-hara-analyzer"
     "0.1.0"
     ["clojure" "babashka"]
     [".clj" ".bb"]
     ["symbols"]
     1048576])

  (defn ^{:schema [:fn [:any] :any]}
    analyze
    [tree]
    [-1 [] [] []])

  0
"#;

#[test]
fn direct_value_worker_compiles_and_materializes() {
    let mut analyzer = SourceAnalyzer::compile(TEST_ANALYZER).expect("compile analyzer");
    let request = crate::json::read(
        r#"{"protocol_version":"1.0","request_id":"x","op":"analyze","language":"clojure","path":"x.clj","blob_oid":"abc","source":"(def x 1)"}"#,
    )
    .unwrap();
    let response = analyzer.handle(&request);
    assert_eq!(
        response
            .object_field("result")
            .and_then(|result| result.object_field("file"))
            .and_then(|file| file.object_field("path"))
            .and_then(Json::as_str),
        Some("x.clj")
    );
}

#[test]
fn persistent_stream_handles_repeated_requests_without_stale_handles() {
    let requests = vec![
        serde_json::json!({
            "protocol_version": "1.0",
            "request_id": "describe",
            "op": "describe"
        })
        .to_string(),
        serde_json::json!({
            "protocol_version": "1.0",
            "request_id": "ping",
            "op": "ping"
        })
        .to_string(),
        serde_json::json!({
            "protocol_version": "1.0",
            "request_id": "one",
            "op": "analyze",
            "language": "clojure",
            "path": "one.clj",
            "blob_oid": "one",
            "source": "(ns one)\n(defn answer [x] (+ x 1))"
        })
        .to_string(),
        serde_json::json!({
            "protocol_version": "1.0",
            "request_id": "unicode",
            "op": "analyze",
            "language": "babashka",
            "path": "unicode.bb",
            "blob_oid": "unicode",
            "source": "(ns café)\n(defn привет [x] (+ x 1))"
        })
        .to_string(),
        serde_json::json!({
            "protocol_version": "1.0",
            "request_id": "broken",
            "op": "analyze",
            "language": "clojure",
            "path": "broken.clj",
            "blob_oid": "broken",
            "source": "(defn broken ["
        })
        .to_string(),
        serde_json::json!({
            "protocol_version": "1.0",
            "request_id": "unknown-op",
            "op": "explode"
        })
        .to_string(),
        "{not-json".to_owned(),
        serde_json::json!({
            "protocol_version": "1.0",
            "request_id": "shutdown",
            "op": "shutdown"
        })
        .to_string(),
        serde_json::json!({
            "protocol_version": "1.0",
            "request_id": "after-shutdown",
            "op": "ping"
        })
        .to_string(),
    ]
    .join("\n");

    let mut output = Vec::new();
    run_jsonl_source(TEST_ANALYZER, Cursor::new(requests), &mut output)
        .expect("persistent analyzer stream");
    let text = String::from_utf8(output).unwrap();
    assert!(!text.contains("stale whole-Wasm runtime handle"));
    let responses = text
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 8);

    let response = |request_id: &str| {
        responses
            .iter()
            .find(|value| value["request_id"] == request_id)
            .unwrap_or_else(|| panic!("missing response for {request_id}"))
    };
    assert_eq!(response("describe")["result"]["name"], "test-hara-analyzer");
    assert_eq!(response("ping")["result"]["ok"], true);
    assert_eq!(response("one")["result"]["file"]["path"], "one.clj");
    assert_eq!(
        response("unicode")["result"]["file"]["path"],
        "unicode.bb"
    );
    assert_eq!(response("broken")["error"]["code"], "parse_error");
    assert_eq!(
        response("unknown-op")["error"]["code"],
        "unsupported_operation"
    );
    assert!(responses.iter().any(|value| {
        value["request_id"] == "unknown" && value["error"]["code"] == "invalid_request"
    }));
    assert_eq!(response("shutdown")["result"]["ok"], true);
    assert!(!text.contains("after-shutdown"));
}

#[test]
fn unsupported_languages_are_rejected_from_the_module_descriptor() {
    let mut analyzer = SourceAnalyzer::compile(TEST_ANALYZER).expect("compile analyzer");
    let request = crate::json::read(
        r#"{"protocol_version":"1.0","request_id":"x","op":"analyze","language":"python","path":"x.py","blob_oid":"abc","source":"x = 1"}"#,
    )
    .unwrap();
    assert!(matches!(
        analyzer.handle(&request).object_field("error").and_then(|error| error.object_field("code")),
        Some(Json::String(code)) if code == "unsupported_language"
    ));
}

#[test]
fn source_index_uses_utf16_display_columns() {
    let index = SourceIndex::new("😀x");
    assert!(matches!(
        index.position("😀x", 4).object_field("column"),
        Some(Json::Integer(3))
    ));
}

#[test]
fn normalization_preserves_semicolons_inside_strings_and_characters() {
    assert_eq!(
        normalize_form("(defn x ; comment\n [] \"semi;colon\" \\; )"),
        "(defn x [] \"semi;colon\" \\; )"
    );
}

#[test]
fn collection_nodes_do_not_render_subtrees_during_indexing() {
    assert_eq!(token_text(&Form::List(vec![])), None);
    assert_eq!(token_text(&Form::Symbol("x".into())), Some("x".into()));
}

#[test]
fn definition_kind_codes_are_protocol_data_not_token_indexes() {
    assert_eq!(definition_kind(1), Ok("variable"));
    assert_eq!(definition_kind(9), Ok("test"));
    assert!(definition_kind(10).is_err());
}
