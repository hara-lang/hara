use hara_wasm::kernel::{parse, parse_forms, Form};
use hara_wasm::project;
use hara_wasm::SessionKernel;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// The test runner is normally a debug binary, whose tree-evaluator frames are
// larger than the optimized production frames. Release verification still
// exercises the production 8 MiB ceiling.
const TEST_STACK_SIZE: usize = if cfg!(debug_assertions) {
    64 * 1024 * 1024
} else {
    8 * 1024 * 1024
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestSummary {
    pub path: PathBuf,
    pub passed: bool,
    pub facts: usize,
    pub checks: usize,
    pub passed_checks: usize,
    pub failed_checks: usize,
    pub errors: usize,
    pub timeouts: usize,
    pub raw: String,
}

impl TestSummary {
    pub fn failure_message(&self) -> String {
        format!("{} failed: {}", self.path.display(), self.raw)
    }
}

pub fn run_file(root: &Path, file: &Path) -> Result<TestSummary, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("cannot resolve {}: {error}", root.display()))?;
    let file = file
        .canonicalize()
        .map_err(|error| format!("cannot resolve {}: {error}", file.display()))?;
    if file.extension().and_then(|value| value.to_str()) != Some("hal") {
        return Err(format!(
            "test file must use the .hal extension: {}",
            file.display()
        ));
    }
    let source = fs::read_to_string(&file)
        .map_err(|error| format!("cannot read {}: {error}", file.display()))?;
    let root_text = root
        .to_str()
        .ok_or_else(|| format!("project root is not UTF-8: {}", root.display()))?
        .to_owned();

    let execution = std::thread::Builder::new()
        .name("hara-test-file".into())
        .stack_size(TEST_STACK_SIZE)
        .spawn(move || {
            let mut kernel = SessionKernel::new();
            kernel.set_test_runner("native")?;
            register_project_sources(&root, &mut kernel)?;
            let mount = kernel.create_native_filesystem(&root_text);
            kernel.attach_filesystem("ROOT", mount)?;
            let session = kernel.session_mut("ROOT")?;
            session.install_native_socket_provider();
            session.install_native_process_provider();
            let output = match kernel.eval("ROOT", &source) {
                Ok(output) => output,
                Err(error) if error.starts_with("SESSION_TRANSFER_REJECTED ") => String::new(),
                Err(error) => return Err(format!("{}: {error}", file.display())),
            };
            parse_summary(file.clone(), &output)
                .map_err(|error| format!("{}: {error}", file.display()))
        })
        .map_err(|error| format!("cannot start test thread: {error}"))?;

    match execution.join() {
        Ok(result) => result,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

fn declared_namespace(source: &str) -> Option<String> {
    parse_forms(source).ok()?.into_iter().find_map(|form| {
        let Form::List(items) = form else {
            return None;
        };
        match items.as_slice() {
            [Form::Symbol(head), Form::Symbol(namespace), ..]
                if (head == "ns" || head == "ns+") && !namespace.contains('/') =>
            {
                Some(namespace.clone())
            }
            _ => None,
        }
    })
}

fn register_project_sources(root: &Path, kernel: &mut SessionKernel) -> Result<(), String> {
    let current_project = project::read(root)?;
    for path in project::files_in(&current_project.root, &current_project.source_paths)? {
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let namespace = declared_namespace(&source)
            .ok_or_else(|| format!("{} does not declare an ns or ns+ namespace", path.display()))?;
        kernel.register_resource(&namespace, &source);
    }
    Ok(())
}

pub fn run_paths(root: &Path, paths: &[PathBuf]) -> Result<Vec<TestSummary>, String> {
    let mut files = Vec::new();
    for path in paths {
        collect(path, &mut files)?;
    }
    files.sort();
    files.dedup();
    if files.is_empty() {
        return Err("no .hal test files found".into());
    }
    files
        .iter()
        .map(|file| run_file(root, file).map_err(|error| format!("{}: {error}", file.display())))
        .collect::<Result<Vec<_>, _>>()
}

fn collect(path: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        if path.extension().and_then(|value| value.to_str()) == Some("hal") {
            output.push(path.to_path_buf());
        }
        return Ok(());
    }
    if !path.is_dir() {
        return Err(format!("test path not found: {}", path.display()));
    }
    let mut entries = fs::read_dir(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?
        .map(|entry| {
            entry
                .map(|value| value.path())
                .map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    for entry in entries {
        collect(&entry, output)?;
    }
    Ok(())
}

fn parse_summary(path: PathBuf, output: &str) -> Result<TestSummary, String> {
    let form = parse(output).map_err(|error| format!("invalid test result: {error}"))?;
    let form = match form {
        Form::String(encoded) => {
            parse(&encoded).map_err(|error| format!("invalid encoded test result: {error}"))?
        }
        value => value,
    };

    match form {
        Form::Vector(items) | Form::List(items) => parse_legacy_results(path, items, output),
        _ => Err("test file must return a native Test/run result vector".into()),
    }
}

fn parse_legacy_results(path: PathBuf, items: Vec<Form>, raw: &str) -> Result<TestSummary, String> {
    let mut passed = 0usize;
    let mut failed = 0usize;
    for item in items {
        let Form::Map(entries) = item else {
            return Err("legacy test result must be a map".into());
        };
        match map_get(&entries, "pass") {
            Some(Form::Bool(true)) => passed += 1,
            Some(Form::Bool(false)) => failed += 1,
            _ => return Err("legacy test result is missing boolean :pass".into()),
        }
    }
    Ok(TestSummary {
        path,
        passed: failed == 0,
        facts: passed + failed,
        checks: passed + failed,
        passed_checks: passed,
        failed_checks: failed,
        errors: 0,
        timeouts: 0,
        raw: raw.to_owned(),
    })
}

fn map_get<'a>(entries: &'a [(Form, Form)], key: &str) -> Option<&'a Form> {
    entries
        .iter()
        .find_map(|(candidate, value)| match candidate {
            Form::Keyword(name) if name == key => Some(value),
            _ => None,
        })
}

fn parse_arguments() -> Result<(PathBuf, Vec<PathBuf>), String> {
    let mut root = env::current_dir().map_err(|error| error.to_string())?;
    let mut paths = Vec::new();
    let mut arguments = env::args().skip(1);
    while let Some(argument) = arguments.next() {
        if argument == "--root" {
            let value = arguments.next().ok_or("--root requires a path")?;
            root = PathBuf::from(value);
        } else if argument == "--help" || argument == "-h" {
            return Err("usage: hara-test [--root ROOT] FILE_OR_DIRECTORY...".into());
        } else {
            paths.push(PathBuf::from(argument));
        }
    }
    if paths.is_empty() {
        let default = root.join("test");
        if default.exists() {
            paths.push(default);
        } else {
            return Err("usage: hara-test [--root ROOT] FILE_OR_DIRECTORY...".into());
        }
    }
    let paths = paths
        .into_iter()
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                root.join(path)
            }
        })
        .collect();
    Ok((root, paths))
}

fn main() {
    let (root, paths) = match parse_arguments() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("hara-test: {error}");
            std::process::exit(2);
        }
    };
    let summaries = match run_paths(&root, &paths) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("hara-test: {error}");
            std::process::exit(2);
        }
    };

    let mut failed = 0usize;
    for summary in &summaries {
        if summary.passed {
            println!(
                "PASS  {} ({} facts, {} checks)",
                summary.path.display(),
                summary.facts,
                summary.checks
            );
        } else {
            failed += 1;
            println!("FAIL  {}", summary.path.display());
            println!("      {}", summary.raw);
        }
    }
    println!(
        "\nHara test summary: {} passed, {} failed, {} total",
        summaries.len() - failed,
        failed,
        summaries.len()
    );
    if failed != 0 {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::run_file;
    use std::path::Path;

    fn repository_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("rust crate must have repository parent")
    }

    #[test]
    fn runs_passing_code_test_summary() {
        let root = repository_root();
        let summary = run_file(
            root,
            &root.join("lib/test-fixtures/std/native/test_runner_pass.hal"),
        )
        .unwrap();
        assert!(summary.passed, "{}", summary.failure_message());
        assert_eq!(summary.facts, 1);
        assert_eq!(summary.checks, 1);
        assert_eq!(summary.passed_checks, 1);
        assert_eq!(summary.failed_checks, 0);
    }

    #[test]
    fn preserves_failing_summary_for_cargo() {
        let root = repository_root();
        let summary = run_file(
            root,
            &root.join("lib/test-fixtures/std/native/test_runner_fail.hal"),
        )
        .unwrap();
        assert!(!summary.passed);
        assert_eq!(summary.facts, 1);
        assert_eq!(summary.checks, 1);
        assert_eq!(summary.passed_checks, 0);
        assert_eq!(summary.failed_checks, 1);
        assert!(summary.failure_message().contains(":failed"));
    }
}
