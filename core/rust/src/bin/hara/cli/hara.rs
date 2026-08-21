use super::Options;
use hara_wasm::cli_app;
use hara_wasm::kernel::{parse, Form};
use hara_wasm::native_cli::{install_native_kernel, RuntimeBroker};
use hara_wasm::project;
#[cfg(feature = "bytecode-vm")]
use hara_wasm::task::production;
use hara_wasm::wasm_binding;
use hara_wasm::Runtime;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

pub(super) fn run(options: &Options, argv: &[String]) -> Result<(), String> {
    run_hara(options, argv)
}

fn run_hara(options: &Options, argv: &[String]) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|error| format!("cannot read cwd: {error}"))?;
    let root = capability_root(options, &cwd);
    let mut runtime = Runtime::new();
    runtime.install_native_file_provider(root.to_string_lossy().as_ref());
    for path in [options.lite_project.as_deref(), options.project.as_deref()]
        .into_iter()
        .flatten()
    {
        let current_project = project::discover(path)?;
        project::register_sources(&current_project, &mut runtime)?;
    }
    cli_app::install_embedded_cli_sources(&mut runtime);
    let process_allowed = options.allow_process
        || argv
            .first()
            .is_some_and(|value| matches!(value.as_str(), "id" | "identity"));
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
        options.allow_postgres,
    )?;
    for path in [options.lite_project.as_deref(), options.project.as_deref()]
        .into_iter()
        .flatten()
    {
        let current_project = project::discover(path)?;
        for (namespace, source) in project::source_resources(&current_project)? {
            broker.register_resource(&namespace, &source)?;
        }
    }
    install_native_kernel(&mut runtime, broker);
    let full_argv = launcher_argv(options, argv);
    let capabilities = capability_edn(options, process_allowed);
    let project = options
        .project
        .as_ref()
        .map(|path| format!("{:?}", path.to_string_lossy()))
        .unwrap_or_else(|| "nil".into());
    let source = format!(
        "(do (require [tool.cli.main :as main] [tool.cli.handlers :as handlers] [tool.cli.model :as model]) \
         (try (main/run (std.native.Edn/read {:?}) {:?} \
          {{:runtime/id :native :runtime/cwd {:?} :runtime/project {} :runtime/capabilities {}}} \
          (handlers/registry)) \
          (catch Throwable error \
           (let [data (ex-data error) message (or (:error data) (ex-message error) (str error))] \
            (model/failure :tool.cli.outcome/usage-error :tool.cli/command-error message)))))",
        cli_app::MANIFEST_SOURCE,
        full_argv,
        cwd.to_string_lossy(),
        project,
        capabilities,
    );
    match runtime.eval_native_traced(&source) {
        Ok(rendered) => render_result(&rendered, options),
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
    selected
        .canonicalize()
        .unwrap_or_else(|_| selected.to_path_buf())
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

fn capability_edn(options: &Options, process_allowed: bool) -> String {
    let mut values = vec![":file"];
    if process_allowed {
        values.push(":process");
    }
    if options.native_sockets {
        values.push(":net");
    }
    if options.allow_postgres {
        values.push(":db/postgres");
    }
    format!("[{}]", values.join(" "))
}

fn render_result(source: &str, options: &Options) -> Result<(), String> {
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
            let Form::Map(message) = message else {
                continue;
            };
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
    if exit != 0 {
        std::process::exit(exit)
    }
    if let Some(Form::Map(data)) = keyword_value(entries, "result/data") {
        if let Some(Form::Keyword(action)) = keyword_value(data, "host/action") {
            let arguments = match keyword_value(data, "host/arguments") {
                Some(Form::Vector(values)) => values
                    .iter()
                    .map(|value| match value {
                        Form::String(value) => Ok(value.clone()),
                        _ => Err("host action arguments must be strings".to_owned()),
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                _ => Vec::new(),
            };
            return execute_host_action(action, &arguments, options);
        }
    }
    Ok(())
}

fn execute_host_action(
    action: &str,
    arguments: &[String],
    options: &Options,
) -> Result<(), String> {
    match action {
        "stdin" => {
            let mut source = String::new();
            io::stdin()
                .read_to_string(&mut source)
                .map_err(|error| format!("stdin: {error}"))?;
            super::project::direct_eval(options, &source)
        }
        "repl" => crate::repl::run_repl(options, options.offline),
        "server" => super::project::run_headless(options),
        "remote" => super::project::run_remote(
            arguments
                .first()
                .ok_or_else(|| "remote requires HOST:PORT".to_owned())?,
        ),
        "extension-inspect" => execute_extension_inspect(options, arguments),
        "extension-bind" => execute_extension_bind(options, arguments),
        "compile-halc" => {
            #[cfg(feature = "halc-encoder")]
            {
                super::project::compile_halc(arguments)
            }
            #[cfg(not(feature = "halc-encoder"))]
            {
                let _ = arguments;
                Err("compile-halc requires the halc-encoder feature".into())
            }
        }
        "production-analyze" => {
            #[cfg(feature = "bytecode-vm")]
            {
                analyze_production(options, arguments)
            }
            #[cfg(not(feature = "bytecode-vm"))]
            {
                let _ = (options, arguments);
                Err("production analysis requires the bytecode-vm feature".into())
            }
        }
        value => Err(format!("unknown Hara host action: {value}")),
    }
}

fn execute_extension_inspect(options: &Options, arguments: &[String]) -> Result<(), String> {
    let (module, output, namespace) = match arguments {
        [module, output] => (module.as_str(), output.as_str(), None),
        [module, output, namespace] => {
            (module.as_str(), output.as_str(), Some(namespace.as_str()))
        }
        _ => return Err("extension inspect host action requires MODULE OUTPUT [NAMESPACE]".into()),
    };
    let module = authoring_path(options, module, "module")?;
    let output = authoring_path(options, output, "output")?;
    let artifact = wasm_binding::write_interface_skeleton(&module, &output, namespace)?;
    println!(
        "Wrote unresolved interface for {} ({}) to {}",
        artifact.namespace,
        artifact.module,
        output.display()
    );
    println!("{}", artifact.inspection_source);
    Ok(())
}

fn execute_extension_bind(options: &Options, arguments: &[String]) -> Result<(), String> {
    let [interface, module, output] = arguments else {
        return Err("extension bind host action requires INTERFACE MODULE OUTPUT".into());
    };
    let interface = authoring_path(options, interface, "interface")?;
    let module = authoring_path(options, module, "module")?;
    let output = authoring_path(options, output, "output")?;
    let package = wasm_binding::bind_package(&interface, &module, &output)?;
    println!(
        "Bound {} as a direct core.v1 extension at {}",
        package.namespace,
        package.root.display()
    );
    println!("module: {}", package.module_digest);
    println!("interface: {}", package.interface_digest);
    println!("bindings: {}", package.binding_digest);
    Ok(())
}

fn authoring_path(options: &Options, value: &str, label: &str) -> Result<PathBuf, String> {
    let relative = Path::new(value);
    let unsafe_path = value.is_empty()
        || value.contains('\\')
        || value.contains(':')
        || value.bytes().any(|byte| byte == 0)
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)));
    if unsafe_path {
        return Err(format!(
            "extension {label} must be a safe relative path within the selected project root"
        ));
    }
    let cwd = std::env::current_dir().map_err(|error| format!("cannot read cwd: {error}"))?;
    Ok(capability_root(options, &cwd).join(relative))
}

#[cfg(feature = "bytecode-vm")]
fn analyze_production(options: &Options, arguments: &[String]) -> Result<(), String> {
    if arguments.len() != 1 {
        return Err("project production build requires one serialized build plan".into());
    }
    let start = options
        .project
        .clone()
        .or_else(|| options.root.clone())
        .unwrap_or_else(|| PathBuf::from("."));
    let project = project::discover(&start)?;
    let output = production::build_and_write(&project, &arguments[0])?;
    if output.analysis.succeeded() {
        let bundle = output
            .bundle_path
            .as_ref()
            .ok_or("production build succeeded without a bundle path")?;
        println!("production bundle: {}", bundle.display());
        println!("shake report: {}", output.report_path.display());
        Ok(())
    } else {
        Err(format!(
            "production build failed with {} diagnostic(s); report: {}",
            output.analysis.diagnostics.len(),
            output.report_path.display()
        ))
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
