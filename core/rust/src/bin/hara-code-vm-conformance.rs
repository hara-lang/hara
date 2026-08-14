use hara_wasm::vm::conformance::run_embedded;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let command = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "check".into());
    let report = run_embedded()?;
    match command.as_str() {
        "check" => {
            if !report.passed() {
                eprintln!("{}", report.to_json(true)?);
                return Err(format!(
                    "code.vm conformance failed with {} failed checks",
                    report.failed_checks()
                ));
            }
            println!(
                "code.vm conformance passed: {} cases",
                report.cases.len()
            );
            Ok(())
        }
        "report" => {
            println!("{}", report.to_json(true)?);
            Ok(())
        }
        "browser" => {
            println!("{}", report.browser_json(true)?);
            Ok(())
        }
        other => Err(format!(
            "unknown code.vm conformance command `{other}`; use check, report, or browser"
        )),
    }
}