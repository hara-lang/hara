use super::Options;
use crate::repl;
#[cfg(feature = "halc-encoder")]
use hara_wasm::kernel::{halc::encode_halc_module, parse_forms};
use hara_wasm::kernel::{parse, Form};
use hara_wasm::native_cli::{install_native_kernel, RuntimeBroker};
use hara_wasm::project;
use hara_wasm::resp::{RespConnection, RespServer, RespValue};
use hara_wasm::Runtime;
use std::fs;
use std::io::{self, BufRead};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

#[cfg(feature = "halc-encoder")]
pub(crate) fn compile_halc(args: &[String]) -> Result<(), String> {
    let source_path = args
        .first()
        .ok_or_else(|| "compile-halc requires SOURCE.hal --output OUTPUT.halc".to_owned())?;
    let output_index = args
        .iter()
        .position(|argument| argument == "--output")
        .ok_or_else(|| "compile-halc requires --output OUTPUT.halc".to_owned())?;
    let output_path = args
        .get(output_index + 1)
        .ok_or_else(|| "compile-halc requires --output OUTPUT.halc".to_owned())?;
    let resource = args
        .iter()
        .position(|argument| argument == "--resource")
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
        .unwrap_or(source_path);
    let source = fs::read_to_string(source_path)
        .map_err(|error| format!("cannot read {source_path}: {error}"))?;
    let forms = parse_forms(&source)?;
    let namespace = forms
        .iter()
        .find_map(|form| match form {
            Form::List(values)
                if matches!(values.first(), Some(Form::Symbol(head)) if head == "ns" || head == "ns+") =>
            {
                match values.get(1) {
                    Some(Form::Symbol(namespace)) => Some(namespace.clone()),
                    _ => None,
                }
            }
            _ => None,
        })
        .ok_or_else(|| format!("{source_path} does not declare an ns or ns+ namespace"))?;
    let artifact = encode_halc_module(&namespace, resource, &source, forms)?;
    if let Some(parent) = std::path::Path::new(output_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    fs::write(output_path, artifact).map_err(|error| format!("cannot write {output_path}: {error}"))
}

fn project_for(options: &Options, args: &[String]) -> Result<project::Project, String> {
    let path = args
        .first()
        .map(PathBuf::from)
        .or_else(|| options.project.clone())
        .unwrap_or_else(|| PathBuf::from("."));
    project::discover(&path)
}

fn eval_runtime(options: &Options) -> Result<Runtime, String> {
    let mut runtime = Runtime::new();
    if options.project.is_some() {
        let project = project_for(options, &[])?;
        project::register_sources(&project, &mut runtime)?;
    }
    if let Some(root) = &options.root {
        runtime.install_native_file_provider(root.to_string_lossy().as_ref());
    }
    if options.native_sockets {
        runtime.install_native_socket_provider();
    }
    if options.allow_process {
        runtime.install_native_process_provider();
    }
    if options.allow_postgres {
        runtime.install_native_module(hara_db_postgres::module())?;
    }
    let broker = RuntimeBroker::start_with(
        options.root.clone().or_else(|| options.project.clone()),
        options.native_sockets,
        options.allow_process,
        options.allow_postgres,
    )?;
    install_native_kernel(&mut runtime, broker);
    Ok(runtime)
}

pub(crate) fn new_project(args: &[String]) -> Result<(), String> {
    let name = args
        .first()
        .ok_or_else(|| "new requires a project name".to_owned())?;
    if args.len() > 1 {
        return Err("new accepts exactly one project name".into());
    }
    let project = project::new_app(&PathBuf::from(name), name)?;
    println!("created {}", project.root.display());
    Ok(())
}

pub(crate) fn check_project(options: &Options, args: &[String]) -> Result<(), String> {
    let project = project_for(options, args)?;
    println!("project check: {} {}", project.id, project.version);
    Ok(())
}

pub(crate) fn edit_dependency(options: &Options, args: &[String], add: bool) -> Result<(), String> {
    let coordinate = args.first().ok_or_else(|| {
        if add {
            "add requires COORDINATE@RANGE".to_owned()
        } else {
            "remove requires COORDINATE".to_owned()
        }
    })?;
    if args.len() > 1 {
        return Err("dependency commands accept one coordinate".into());
    }
    let (coordinate, version) = if add {
        coordinate
            .rsplit_once('@')
            .ok_or_else(|| "add requires COORDINATE@RANGE".to_owned())?
    } else {
        (coordinate.as_str(), "")
    };
    let project = project_for(options, &[])?;
    project::set_dependency(&project, coordinate, if add { Some(version) } else { None })?;
    println!("{} {}", if add { "added" } else { "removed" }, coordinate);
    Ok(())
}

pub(crate) fn sync_project(options: &Options, args: &[String]) -> Result<(), String> {
    let project = project_for(options, &[])?;
    let flags: Vec<_> = args.iter().skip(1).collect();
    let mode = match flags.as_slice() {
        [] if options.offline => project::LockMode::Offline,
        [] => project::LockMode::Default,
        [flag] if (*flag).as_str() == "--offline" => project::LockMode::Offline,
        [flag] if (*flag).as_str() == "--locked" => project::LockMode::Locked,
        [flag] if (*flag).as_str() == "--frozen" => project::LockMode::Frozen,
        _ => return Err("sync accepts at most one of --offline, --locked, or --frozen".into()),
    };
    let lock = project::sync_lock(&project, mode)?;
    println!("project sync: {}", lock.display());
    Ok(())
}

pub(crate) fn run_project(options: &Options) -> Result<(), String> {
    let project = project_for(options, &[])?;
    let path = project::main_file(&project)?;
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let mut runtime = Runtime::new();
    runtime.install_native_file_provider(project.root.to_string_lossy().as_ref());
    project::register_sources(&project, &mut runtime)?;
    if options.native_sockets {
        runtime.install_native_socket_provider();
    }
    if options.allow_process {
        runtime.install_native_process_provider();
    }
    println!("{}", runtime.eval_native(&source)?);
    Ok(())
}

pub(crate) fn test_project(options: &Options, args: &[String]) -> Result<(), String> {
    if args.len() > 1 {
        return Err("test accepts at most one path".into());
    }
    let project = project_for(options, args)?;
    let files = match args.first().map(PathBuf::from) {
        Some(path)
            if path.is_file()
                && path.extension().and_then(|extension| extension.to_str()) == Some("hal") =>
        {
            vec![path]
        }
        Some(path) if path.is_file() => return Err("test file must use the .hal extension".into()),
        Some(path) if path.is_dir() => project::files_in(&path, &[PathBuf::new()])?,
        Some(path) => return Err(format!("test path does not exist: {}", path.display())),
        None => project::files_in(&project.root, &project.test_paths)?,
    };
    if files.is_empty() {
        return Err("project has no .hal files under :project/test-paths".into());
    }
    let mut passed = 0usize;
    let mut failed = 0usize;
    for path in files {
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let mut runtime = Runtime::new();
        runtime.install_native_file_provider(project.root.to_string_lossy().as_ref());
        project::register_sources(&project, &mut runtime)?;
        if options.allow_process {
            runtime.install_native_process_provider();
        }
        let evaluated = runtime.eval_native(&source)?;
        match test_results(&evaluated) {
            Ok((file_passed, file_failed)) => {
                passed += file_passed;
                failed += file_failed;
                println!(
                    "test {}: {} passed, {} failed",
                    path.display(),
                    file_passed,
                    file_failed
                );
                if file_failed > 0 {
                    eprintln!("{evaluated}");
                }
            }
            Err(error) => {
                failed += 1;
                eprintln!("test {}: {error}", path.display());
            }
        }
    }
    println!("test result: {passed} passed, {failed} failed");
    if failed == 0 {
        Ok(())
    } else {
        Err("test failures".into())
    }
}

fn test_results(value: &str) -> Result<(usize, usize), String> {
    let parsed = parse(value)?;
    let result = match parsed {
        Form::String(source) => parse(&source)?,
        value => value,
    };

    match result {
        Form::Map(entries) => code_test_results(&entries),
        Form::Vector(results) | Form::List(results) => native_test_results(results),
        _ => {
            Err("test file must return a code.test/run summary or native test result vector".into())
        }
    }
}

fn code_test_results(entries: &[(Form, Form)]) -> Result<(usize, usize), String> {
    let status = map_get(entries, "status")
        .and_then(keyword)
        .ok_or("code.test/run result is missing :status")?;
    let counts = match map_get(entries, "counts") {
        Some(Form::Map(values)) => values,
        _ => return Err("code.test/run result is missing :counts".into()),
    };
    let passed = map_number(counts, "passed", 0);
    let mut failed = map_number(counts, "failed", 0)
        + map_number(counts, "error", 0)
        + map_number(counts, "timeout", 0);
    if status != "passed" && failed == 0 {
        failed = 1;
    }
    Ok((passed, failed))
}

fn native_test_results(results: Vec<Form>) -> Result<(usize, usize), String> {
    let mut passed = 0;
    let mut failed = 0;
    for result in results {
        let Form::Map(entries) = result else {
            return Err("native test result must be a map".into());
        };
        match map_get(&entries, "pass") {
            Some(Form::Bool(true)) => passed += 1,
            Some(Form::Bool(false)) => failed += 1,
            _ => return Err("native test result is missing boolean :pass".into()),
        }
    }
    Ok((passed, failed))
}

fn map_get<'a>(entries: &'a [(Form, Form)], key: &str) -> Option<&'a Form> {
    entries
        .iter()
        .find_map(|(candidate, value)| match candidate {
            Form::Keyword(name) if name == key => Some(value),
            _ => None,
        })
}

fn map_number(entries: &[(Form, Form)], key: &str, fallback: usize) -> usize {
    match map_get(entries, key) {
        Some(Form::Number(value)) if *value >= 0 => *value as usize,
        _ => fallback,
    }
}

fn keyword(form: &Form) -> Option<&str> {
    match form {
        Form::Keyword(value) => Some(value),
        _ => None,
    }
}

fn hal_string(value: &str) -> String {
    format!("{value:?}")
}

fn workflow_units(
    project: &project::Project,
    paths: &[PathBuf],
    language: Option<&str>,
) -> Result<String, String> {
    let mut units = Vec::new();
    let test_root = project
        .root
        .join(paths.first().cloned().unwrap_or_default());
    for path in project::files_in(&project.root, paths)? {
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let path_text = path.to_string_lossy();
        let mut unit = format!(
            "{{:path {} :source {}}}",
            hal_string(path_text.as_ref()),
            hal_string(&contents)
        );
        if let Some(language) = language {
            let root_text = test_root.to_string_lossy();
            unit = format!(
                "{{:path {} :source {} :language :{} :test-root {}}}",
                hal_string(path_text.as_ref()),
                hal_string(&contents),
                language,
                hal_string(root_text.as_ref())
            );
        }
        units.push(unit);
    }
    Ok(format!("[{}]", units.join(" ")))
}

fn run_workflow(
    options: &Options,
    namespace: &str,
    operation: &str,
    units: String,
) -> Result<(), String> {
    let mut runtime = eval_runtime(options)?;
    let source =
        format!("(require (quote {namespace})) ({namespace}/run :{operation} {{:units {units}}})");
    println!("{}", runtime.eval_native(&source)?);
    Ok(())
}

pub(crate) fn manage_project(options: &Options, args: &[String]) -> Result<(), String> {
    const OPERATIONS: &[&str] = &[
        "analyse",
        "extract",
        "vars",
        "docstrings",
        "transform-code",
        "import",
        "purge",
        "missing",
        "todos",
        "incomplete",
        "incomplete-report",
        "orphaned",
        "scaffold",
        "create-tests",
        "in-order",
        "arrange",
        "factcheck-remove",
        "factcheck-generate",
        "snapto",
        "isolate",
        "locate-code",
        "locate-test",
        "grep",
        "grep-replace",
        "unclean",
        "unclean-findings",
        "unchecked",
        "commented",
        "pedantic",
        "refactor-code",
        "refactor-test",
        "ns-format",
        "ns-rename",
        "find-usages",
        "require-file",
        "heal-code",
    ];
    let operation = args.first().map(String::as_str).unwrap_or("analyse");
    if !OPERATIONS.contains(&operation) {
        return Err(format!("unsupported manage operation {operation:?}"));
    }
    let project = project_for(options, &[])?;
    let parsed = manage_arguments(&args[1..])?;
    let units = manage_units(&project, operation, &parsed.namespaces)?;
    // code.manage receives complete source units from the host and must not
    // eagerly parse every project namespace merely to construct a plan.
    let mut runtime = Runtime::new();
    let source = format!(
        "(require (quote code.manage)) (code.manage/plan :{operation} {{:units {units} :options {}}})",
        parsed.options
    );
    let evaluated = runtime.eval_native(&source)?;
    let plan = parse(&evaluated).map_err(|error| format!("invalid code.manage plan: {error}"))?;
    if parsed.write {
        apply_manage_edits(&project.root, &plan)?;
    }
    println!("{evaluated}");
    Ok(())
}

#[derive(Default)]
struct ManageArguments {
    write: bool,
    namespaces: Vec<String>,
    options: String,
}

fn manage_arguments(args: &[String]) -> Result<ManageArguments, String> {
    let mut write = false;
    let mut namespaces = Vec::new();
    let mut options = Vec::new();
    let mut patterns = Vec::new();
    let mut index = 0usize;
    while index < args.len() {
        let argument = &args[index];
        if argument == "--write" {
            write = true;
            index += 1;
        } else if matches!(
            argument.as_str(),
            "--match" | "--replacement" | "--from" | "--to" | "--needle" | "--form" | "--var"
        ) {
            let value = args
                .get(index + 1)
                .ok_or_else(|| format!("{argument} requires a value"))?;
            options.push(format!(
                ":{} {}",
                argument.trim_start_matches("--"),
                hal_string(value)
            ));
            index += 2;
        } else if argument == "--pattern" {
            let value = args.get(index + 1).ok_or("--pattern requires a value")?;
            patterns.push(hal_string(value));
            index += 2;
        } else if argument.starts_with('-') {
            return Err(format!("unknown manage option {argument}"));
        } else {
            namespaces.push(argument.clone());
            index += 1;
        }
    }
    if !patterns.is_empty() {
        options.push(format!(":patterns [{}]", patterns.join(" ")));
    }
    Ok(ManageArguments {
        write,
        namespaces,
        options: format!("{{{}}}", options.join(" ")),
    })
}

fn namespace_selected(path: &Path, namespaces: &[String]) -> bool {
    namespaces.is_empty()
        || namespaces.iter().any(|namespace| {
            let suffix = namespace.replace('.', "/").replace('-', "_");
            path.to_string_lossy().contains(&suffix)
        })
}

fn source_test_path(project: &project::Project, path: &Path) -> Option<PathBuf> {
    let test_root = project.test_paths.first()?;
    for source_root in &project.source_paths {
        let absolute_root = project.root.join(source_root);
        if let Ok(relative) = path.strip_prefix(&absolute_root) {
            let stem = relative.to_string_lossy().strip_suffix(".hal")?.to_owned();
            return Some(
                project
                    .root
                    .join(test_root)
                    .join(format!("{stem}_test.hal")),
            );
        }
    }
    None
}

fn manage_units(
    project: &project::Project,
    operation: &str,
    namespaces: &[String],
) -> Result<String, String> {
    let test_only = matches!(
        operation,
        "refactor-test" | "isolate" | "factcheck-remove" | "factcheck-generate" | "unchecked"
    );
    let both = matches!(
        operation,
        "ns-rename" | "find-usages" | "grep" | "grep-replace"
    );
    let mut typed_paths = Vec::new();
    if !test_only {
        typed_paths.extend(
            project::files_in(&project.root, &project.source_paths)?
                .into_iter()
                .map(|path| (path, "source")),
        );
    }
    if test_only || both {
        typed_paths.extend(
            project::files_in(&project.root, &project.test_paths)?
                .into_iter()
                .map(|path| (path, "test")),
        );
    }
    let mut units = Vec::new();
    for (path, kind) in typed_paths {
        if !namespace_selected(&path, namespaces) {
            continue;
        }
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let test_path = source_test_path(project, &path)
            .map(|value| format!(" :test-path {}", hal_string(&value.to_string_lossy())))
            .unwrap_or_default();
        units.push(format!(
            "{{:path {} :source {} :type :{kind}{test_path}}}",
            hal_string(&path.to_string_lossy()),
            hal_string(&contents)
        ));
    }
    Ok(format!("[{}]", units.join(" ")))
}

fn manage_map_get<'a>(value: &'a Form, key: &str) -> Option<&'a Form> {
    match value {
        Form::Map(entries) => entries.iter().find_map(|(candidate, value)| {
            matches!(candidate, Form::Keyword(name) if name == key).then_some(value)
        }),
        _ => None,
    }
}

fn manage_string(value: Option<&Form>, label: &str) -> Result<String, String> {
    match value {
        Some(Form::String(value)) => Ok(value.clone()),
        _ => Err(format!("code.manage edit requires string {label}")),
    }
}

fn apply_manage_edits(root: &Path, plan: &Form) -> Result<(), String> {
    let edits = match manage_map_get(plan, "edits") {
        Some(Form::Vector(values)) => values,
        _ => return Err("code.manage plan is missing :edits".into()),
    };
    let canonical_root = fs::canonicalize(root)
        .map_err(|error| format!("cannot resolve project root {}: {error}", root.display()))?;
    let mut validated = Vec::new();
    for edit in edits {
        let path_text = manage_string(manage_map_get(edit, "path"), ":path")?;
        let before = manage_string(manage_map_get(edit, "before"), ":before")?;
        let after = manage_string(manage_map_get(edit, "after"), ":after")?;
        let path = PathBuf::from(path_text);
        let absolute = if path.is_absolute() {
            path
        } else {
            canonical_root.join(path)
        };
        let parent = absolute
            .parent()
            .ok_or_else(|| format!("code.manage edit has no parent: {}", absolute.display()))?;
        let checked_parent = if parent.exists() {
            fs::canonicalize(parent)
                .map_err(|error| format!("cannot resolve {}: {error}", parent.display()))?
        } else {
            let mut ancestor = parent;
            while !ancestor.exists() {
                ancestor = ancestor.parent().ok_or("edit path escapes project root")?;
            }
            fs::canonicalize(ancestor)
                .map_err(|error| format!("cannot resolve {}: {error}", ancestor.display()))?
        };
        if !checked_parent.starts_with(&canonical_root) {
            return Err(format!(
                "code.manage edit escapes project root: {}",
                absolute.display()
            ));
        }
        let current = if absolute.exists() {
            fs::read_to_string(&absolute)
                .map_err(|error| format!("cannot read {}: {error}", absolute.display()))?
        } else {
            String::new()
        };
        if current != before {
            return Err(format!("code.manage stale edit: {}", absolute.display()));
        }
        validated.push((absolute, after));
    }
    for (path, after) in validated {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        }
        fs::write(&path, after)
            .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    }
    Ok(())
}

pub(crate) fn seedgen_project(options: &Options, args: &[String]) -> Result<(), String> {
    let operation = args.first().map(String::as_str).unwrap_or("list");
    if !matches!(operation, "root" | "list" | "incomplete" | "benchadd") {
        return Err("seedgen supports root, list, incomplete, or benchadd".into());
    }
    let language =
        (operation == "benchadd").then(|| args.get(1).map(String::as_str).unwrap_or("js"));
    let project = project_for(options, &[])?;
    let units = workflow_units(&project, &project.test_paths, language)?;
    run_workflow(options, "lang.seedgen", operation, units)
}

pub(crate) fn direct_eval(options: &Options, source: &str) -> Result<(), String> {
    if source.is_empty() {
        return Err("eval requires a Hara expression".into());
    }
    let mut runtime = eval_runtime(options)?;
    println!("{}", runtime.eval_native(source)?);
    Ok(())
}

pub(crate) fn run_file(options: &Options, path: &str) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|error| format!("cannot read {path}: {error}"))?;
    let is_halc = path.ends_with(".halc")
        || path.ends_with(".hir")
        || bytes.starts_with(b"HALC")
        || bytes.starts_with(b"HIR\0");
    let mut runtime = eval_runtime(options)?;
    if is_halc {
        println!("{}", runtime.eval_halc(&bytes)?);
    } else {
        println!(
            "{}",
            runtime.eval_native(
                &String::from_utf8(bytes)
                    .map_err(|error| format!("{path} is not valid UTF-8: {error}"))?
            )?
        );
    }
    Ok(())
}

pub(crate) fn run_headless(options: &Options) -> Result<(), String> {
    if options.offline {
        return Err("--offline cannot be used with headless".into());
    }
    let broker = RuntimeBroker::start_with(
        options.root.clone(),
        options.native_sockets,
        options.allow_process,
        options.allow_postgres,
    )?;
    let server = RespServer::start(&options.host, options.port, broker)?;
    println!("HARA RESP {} · session ROOT", server.endpoint());
    loop {
        std::thread::park();
    }
}

pub(crate) fn run_remote(endpoint: &str) -> Result<(), String> {
    let (host, port) = repl::parse_endpoint(endpoint, "127.0.0.1")?;
    let stream = TcpStream::connect((host.as_str(), port))
        .map_err(|error| format!("remote connect failed: {error}"))?;
    let mut connection = RespConnection::new(stream)?;
    connection.write(&RespValue::array(["HELLO", "4", "CLIENT", "HARA-REMOTE"]))?;
    println!(
        "{}",
        response_text(connection.read()?.ok_or("remote closed")?)
    );
    let mut request = 0_u64;
    for line in io::stdin().lock().lines() {
        let source = line.map_err(|error| format!("stdin: {error}"))?;
        if matches!(source.trim(), "/quit" | ":quit") {
            connection.write(&RespValue::array(["QUIT"]))?;
            break;
        }
        request += 1;
        let id = format!("REMOTE-{request}");
        connection.write(&RespValue::array(["EVAL", &id, source.trim()]))?;
        if let Some(value) = connection.read()? {
            println!("{}", response_text(value));
        }
        let _ = connection.read()?;
    }
    Ok(())
}

fn response_text(value: RespValue) -> String {
    match value {
        RespValue::Array(Some(values)) => values
            .into_iter()
            .map(response_text)
            .collect::<Vec<_>>()
            .join(" "),
        RespValue::Bulk(Some(bytes)) => String::from_utf8_lossy(&bytes).into_owned(),
        RespValue::Simple(value) | RespValue::Error(value) => value,
        RespValue::Integer(value) => value.to_string(),
        RespValue::Bulk(None) | RespValue::Array(None) => "nil".into(),
    }
}

#[cfg(test)]
#[path = "project/tests.rs"]
mod tests;
