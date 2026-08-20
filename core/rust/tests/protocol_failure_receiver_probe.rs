use hara_wasm::Runtime;

const SOURCE: &str = include_str!(
    "../hal-test-fixtures/std/foundation/protocol_functionality.hal"
);

fn failure_forms(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let start = line.find("'(std.protocol.")? + 1;
            let form = &line[start..];
            let mut depth = 0_usize;
            for (index, character) in form.char_indices() {
                match character {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(form[..=index].to_owned());
                        }
                    }
                    _ => {}
                }
            }
            None
        })
        .collect()
}

#[test]
fn discover_explicit_protocol_failure_receivers() {
    let candidates = [
        ("struct", "(UnsupportedUseCase)"),
        ("protocol", "std.protocol.icount/ICount"),
        ("function", "(fn:> [] 0)"),
        ("atom", "(atom 0)"),
        ("number", "1"),
        ("nil", "nil"),
    ];

    let mut runtime = Runtime::new();
    runtime
        .eval_native(SOURCE)
        .expect("the shared protocol fixture must load before probing receivers");

    let mut report = Vec::new();
    for form in failure_forms(SOURCE) {
        let mut outcomes = Vec::new();
        let mut selected = None;
        for (name, receiver) in candidates {
            let call = form.replacen("unsupported", receiver, 1);
            match runtime.eval_native(&call) {
                Err(error) if error.contains("protocol/unsupported-receiver") => {
                    outcomes.push(format!("{name}=unsupported"));
                    if selected.is_none() {
                        selected = Some((name, receiver));
                    }
                }
                Err(error) => outcomes.push(format!(
                    "{name}=other-error:{}",
                    error.replace('\n', " ")
                )),
                Ok(value) => outcomes.push(format!("{name}=ok:{value}")),
            }
        }
        let selection = selected
            .map(|(name, receiver)| format!("{name}:{receiver}"))
            .unwrap_or_else(|| "NONE".to_owned());
        report.push(format!(
            "{form} => {selection} [{}]",
            outcomes.join(", ")
        ));
    }

    panic!(
        "temporary #846 receiver probe; replace this test with corpus assertions:\n{}",
        report.join("\n")
    );
}
