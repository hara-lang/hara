use super::Options;
use hara_wasm::cli_app;
use hara_wasm::kernel::{parse, Form};
use hara_wasm::native_cli::{install_native_kernel, RuntimeBroker};
use hara_wasm::Runtime;
use std::path::{Path, PathBuf};

const PORTED_HANDLERS: &[&str] = &[
    "hara.cli.handler/eval",
    "hara.cli.handler/run-file",
    "hara.cli.handler/project-run",
    "hara.cli.handler/manage",
    "hara.cli.handler/project-test",
    "hara.cli.handler/project-new",
    "hara.cli.handler/project-check",
    "hara.cli.handler/project-add",
    "hara.cli.handler/project-remove",
    "hara.cli.handler/project-sync",
    "hara.cli.handler/project-update",
    "hara.cli.handler/spec",
    "hara.cli.handler/seedgen",
    "hara.cli.handler/asset",
    "hara.cli.handler/extension",
    "hara.cli.handler/identity",
    "hara.cli.handler/package",
];

pub(super) fn run_if_ported(options: &Options, argv: &[String]) -> Option<Result<(), String>> {
    let resolved = cli_app::router().resolve(argv)?;
    if !PORTED_HANDLERS.contains(&resolved.route.handler.as_str()) {
        return None;
    }
    Some(run(options, argv, &resolved.route.handler))
}

fn run(options: &Options, argv: &[String], handler: &str) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|error| format!("cannot read cwd: {error}"))?;
    let root = capability_root(options, &cwd);
    let mut runtime = Runtime::new();
    runtime.install_native_file_provider(root.to_string_lossy().as_ref());
    let process_allowed = options.allow_process || handler == "hara.cli.handler/identity";
    if process_allowed {
        runtime.install_native_process_provider();
    }
    if options.native_sockets {
        runtime.install_native_socket_provider();
    }
    let broker = RuntimeBroker::start_with(
        Some(root.clone()),
        options.native_sockets,
        process_allowed,
    )?;
    install_native_kernel(&mut runtime, broker);
    let full_argv = launcher_argv(options, argv);
    let capabilities = capability_edn(options, handler);
    let project = options
        .project
        .as_ref()
        .map(|path| format!("{:?}", path.to_string_lossy()))
        .unwrap_or_else(|| "nil".into());
    let source = format!(
        "(do (require [std.foundation.edn :as edn] [hara.cli.main :as main] [hara.cli.handlers :as handlers] [hara.cli.model :as model]) \
         (try (main/run (edn/read {:?}) {:?} \
          {{:runtime/id :native :runtime/cwd {:?} :runtime/project {} :runtime/capabilities {}}} \
          (handlers/registry)) \
          (catch Throwable error \
           (let [data (ex-data error) message (or (:error data) (ex-message error) (str error))] \
            (model/failure :hara.cli.outcome/usage-error :hara.cli/command-error message)))))",
        cli_app::MANIFEST_SOURCE,
        full_argv,
        cwd.to_string_lossy(),
        project,
        capabilities,
    );
    match runtime.eval_native_traced(&source) {
        Ok(rendered) => render_result(&rendered),
        Err(error) => render_runtime_error(&error),
    }
}

fn capability_root(options: &Options, cwd: &Path) -> PathBuf {
    let selected = options
        .project
        .as_deref()
        .or(options.root.as_deref())
        .unwrap_or(cwd);
    let selected = if selected.is_file() {
        selected.parent().unwrap_or(selected)
    } else {
        selected
    };
    selected.canonicalize().unwrap_or_else(|_| selected.to_path_buf())
}

fn launcher_argv(options: &Options, argv: &[String]) -> Vec<String> {
    let mut output = Vec::new();
    if let Some(path) = &options.project {
        output.extend(["--project".into(), path.to_string_lossy().into_owned()]);
    }
    if let Some(path) = &options.root {
        output.extend(["--root".into(), path.to_string_lossy().into_owned()]);
    }
    if options.allow_file {
        output.push("--allow-file".into());
    }
    if options.allow_process {
        output.push("--allow-process".into());
    }
    if options.native_sockets {
        output.push("--allow-net".into());
    }
    if options.offline {
        output.push("--offline".into());
    }
    output.extend(argv.iter().cloned());
    output
}

fn capability_edn(options: &Options, handler: &str) -> String {
    let mut values = vec![":file"];
    if options.allow_process || handler == "hara.cli.handler/identity" {
        values.push(":process");
    }
    if options.native_sockets {
        values.push(":net");
    }
    format!("[{}]", values.join(" "))
}

fn render_result(source: &str) -> Result<(), String> {
    let form = parse(source).map_err(|error| format!("invalid Hara CLI result: {error}"))?;
    let entries = match &form {
        Form::Map(entries) => entries,
        _ => return Err(format!("invalid Hara CLI result: {source}")),
    };
    let exit = match keyword_value(entries, "result/exit") {
        Some(Form::Number(value)) => *value as i32,
        _ => return Err("Hara CLI result is missing :result/exit".into()),
    };
    if let Some(Form::Vector(messages)) = keyword_value(entries, "result/messages") {
        for message in messages {
            let Form::Map(message) = message else { continue };
            let stream = match keyword_value(message, "message/stream") {
                Some(Form::Keyword(value)) => value.as_str(),
                _ => "stdout",
            };
            let Some(Form::String(text)) = keyword_value(message, "message/text") else {
                continue;
            };
            if stream == "stderr" {
                eprint!("{text}");
            } else {
                print!("{text}");
            }
        }
    }
    if exit == 0 {
        Ok(())
    } else {
        std::process::exit(exit)
    }
}

fn render_runtime_error(error: &str) -> Result<(), String> {
    let message = error.split("\n[hara stack]").next().unwrap_or(error);
    eprintln!("{message}");
    let exit = if message.starts_with("unknown ")
        || message.starts_with("usage:")
        || message.starts_with("unavailable:")
        || message.contains(" requires ")
        || message.contains("cannot read")
        || message.contains("not found")
        || message.contains("No such file")
    {
        2
    } else {
        1
    };
    std::process::exit(exit)
}

fn keyword_value<'a>(entries: &'a [(Form, Form)], key: &str) -> Option<&'a Form> {
    entries.iter().find_map(|(candidate, value)| {
        matches!(candidate, Form::Keyword(name) if name == key).then_some(value)
    })
}
