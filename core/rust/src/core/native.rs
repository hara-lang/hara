fn os_operation(
    operation: &str,
    forms: &[Form],
    env: &mut HashMap<String, Value>,
) -> Result<Value, String> {
    let operation = operation.strip_prefix("os/").unwrap_or(operation);
    let operation = operation
        .strip_prefix("std.native.OS/")
        .unwrap_or(operation);
    let process_operation = operation.strip_prefix("std.native.Process/");
    let operation = match process_operation.unwrap_or(operation) {
        "instance?" if process_operation.is_some() => "process?",
        "alive?" if process_operation.is_some() => "process-alive?",
        "write" if process_operation.is_some() => "process-write",
        "close-input" if process_operation.is_some() => "process-close-input",
        "stdout" if process_operation.is_some() => "process-stdout",
        "stderr" if process_operation.is_some() => "process-stderr",
        "stdout-stream" if process_operation.is_some() => "process-stdout-stream",
        "stderr-stream" if process_operation.is_some() => "process-stderr-stream",
        "wait" if process_operation.is_some() => "process-wait",
        "kill" if process_operation.is_some() => "process-kill",
        value => value,
    };
    match operation {
        "platform" => {
            if !forms.is_empty() {
                return Err("os/platform expects no arguments".into());
            }
            let platform = if cfg!(target_os = "linux") {
                "linux"
            } else if cfg!(target_os = "macos") {
                "macos"
            } else if cfg!(target_os = "windows") {
                "windows"
            } else {
                "unknown"
            };
            return Ok(Value::Keyword(platform.into()));
        }
        "arch" => {
            if !forms.is_empty() {
                return Err("os/arch expects no arguments".into());
            }
            let arch = match std::env::consts::ARCH {
                "x86_64" => "x86-64",
                value => value,
            };
            return Ok(Value::Keyword(arch.into()));
        }
        "cwd" => {
            if !forms.is_empty() {
                return Err("os/cwd expects no arguments".into());
            }
            return std::env::current_dir()
                .map(|path| Value::String(path.to_string_lossy().into_owned()))
                .map_err(|error| format!("os/cwd failed: {error}"));
        }
        "env" => {
            if !forms.is_empty() {
                return Err("os/env expects no arguments".into());
            }
            return Ok(Value::Map(PMap::from_iter(
                std::env::vars().map(|(key, value)| (Value::String(key), Value::String(value))),
            )));
        }
        "getenv" => {
            if forms.len() != 1 {
                return Err("os/getenv expects a name".into());
            }
            let Value::String(name) = eval(&forms[0], env)? else {
                return Err("os/getenv expects a string".into());
            };
            return Ok(std::env::var(name).map(Value::String).unwrap_or(Value::Nil));
        }
        "process?" => {
            if forms.len() != 1 {
                return Err("os/process? expects one argument".into());
            }
            let value = eval(&forms[0], env)?;
            #[cfg(not(target_arch = "wasm32"))]
            return Ok(Value::Bool(crate::native_process::is_process(&value)));
            #[cfg(target_arch = "wasm32")]
            return Ok(Value::Bool(false));
        }
        _ => {}
    }
    require_process_access(&format!("os/{operation}"))?;
    #[cfg(target_arch = "wasm32")]
    return Err(format!("os/{operation} is unsupported on wasm"));
    #[cfg(not(target_arch = "wasm32"))]
    match operation {
        "spawn" => {
            if !(1..=2).contains(&forms.len()) {
                return Err("os/spawn expects argv and optional options".into());
            }
            let argv = iterator_values(eval(&forms[0], env)?)?
                .into_iter()
                .map(|value| match value {
                    Value::String(value) => Ok(value),
                    _ => Err("os/spawn argv must contain strings".to_owned()),
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut cwd = None;
            let mut environment = Vec::new();
            if forms.len() == 2 {
                let options = eval(&forms[1], env)?;
                for (key, value) in map_entries(&options)
                    .ok_or_else(|| "os/spawn options must be a map".to_owned())?
                {
                    match (key, value) {
                        (Value::Keyword(key), Value::String(value)) if key.as_str() == "cwd" => {
                            cwd = Some(value);
                        }
                        (Value::Keyword(key), value) if key.as_str() == "env" => {
                            for (name, value) in map_entries(&value)
                                .ok_or_else(|| "os/spawn :env must be a map".to_owned())?
                            {
                                let (Value::String(name), Value::String(value)) = (name, value)
                                else {
                                    return Err("os/spawn :env must contain string pairs".into());
                                };
                                environment.push((name, value));
                            }
                        }
                        _ => {}
                    }
                }
            }
            crate::native_process::spawn(&argv, cwd.as_deref(), &environment)
        }
        method @ ("process-alive?"
        | "process-close-input"
        | "process-stdout"
        | "process-stderr"
        | "process-wait"
        | "process-kill") => {
            if forms.len() != 1 {
                return Err(format!("os/{method} expects a process"));
            }
            let process = eval(&forms[0], env)?;
            match method {
                "process-alive?" => crate::native_process::alive(&process).map(Value::Bool),
                "process-close-input" => {
                    crate::native_process::close_input(&process).map(|()| Value::Nil)
                }
                "process-stdout" => {
                    crate::native_process::promise(&process, "stdout").map(Value::Promise)
                }
                "process-stderr" => {
                    crate::native_process::promise(&process, "stderr").map(Value::Promise)
                }
                "process-wait" => {
                    crate::native_process::promise(&process, "wait").map(Value::Promise)
                }
                "process-kill" => crate::native_process::kill(&process).map(|()| process),
                _ => unreachable!(),
            }
        }
        method @ ("process-stdout-stream" | "process-stderr-stream") => {
            if forms.len() != 1 { return Err(format!("os/{method} expects a process")); }
            let process = eval(&forms[0], env)?;
            let kind = if method == "process-stderr-stream" { "stderr" } else { "stdout" };
            let handle = crate::native_process::take_stream(&process, kind)?;
            Ok(host_stream(Rc::new(move || Ok(crate::native_process::stream_promise(handle, kind))), Rc::new(|| Ok(()))))
        }
        "process-write" => {
            if forms.len() != 2 {
                return Err("os/process-write expects a process and bytes".into());
            }
            let process = eval(&forms[0], env)?;
            let bytes = match eval(&forms[1], env)? {
                Value::Bytes(value) => value,
                Value::ByteBuffer(value) => value.borrow().clone(),
                _ => return Err("os/process-write expects bytes".into()),
            };
            crate::native_process::write(&process, &bytes).map(|count| Value::Number(count as i64))
        }
        _ => Err(format!("unknown os operation: {operation}")),
    }
}

fn native_test_events() -> Value {
    Value::Vector(PVector::from_iter([
        Value::Keyword("test/run-started".into()),
        Value::Keyword("test/fact-started".into()),
        Value::Keyword("test/fact-completed".into()),
        Value::Keyword("test/run-completed".into()),
    ]))
}

fn native_test_runner(value: Value) -> Result<Value, String> {
    match value {
        Value::Keyword(runner) if matches!(runner.as_str(), "code.test" | "native") => {
            Ok(Value::Keyword(runner))
        }
        _ => Err("runtime test runner must be :code.test or :native".into()),
    }
}

fn native_test_active_runner() -> Result<Value, String> {
    ACTIVE_TEST_RUNNER.with(|runner| {
        native_test_runner(Value::Keyword(runner.borrow().clone().into()))
    })
}

fn native_test_config(
    runner: Value,
    options: Value,
) -> Result<Value, String> {
    if map_entries(&options).is_none() {
        return Err("std.native.Test/config options must be a map".into());
    }
    if map_value(&options, &Value::Keyword("runner".into())).is_some() {
        return Err("std.native.Test/config runner is owned by the runtime".into());
    }
    Ok(Value::Map(PMap::from_iter([
        (Value::Keyword("runner".into()), runner),
        (Value::Keyword("options".into()), options),
    ])))
}

fn native_test_result(name: Value, actual: Value, expected: Value) -> Value {
    let pass = actual == expected;
    Value::Map(PMap::from_iter([
        (Value::Keyword("name".into()), name),
        (Value::Keyword("pass".into()), Value::Bool(pass)),
        (Value::Keyword("actual".into()), actual),
        (Value::Keyword("expected".into()), expected),
    ]))
}

fn native_test_error(name: Value, expected: Value, error: String) -> Value {
    Value::Map(PMap::from_iter([
        (Value::Keyword("name".into()), name),
        (Value::Keyword("pass".into()), Value::Bool(false)),
        (Value::Keyword("status".into()), Value::Keyword("error".into())),
        (Value::Keyword("expected".into()), expected),
        (Value::Keyword("error".into()), Value::String(error)),
    ]))
}

fn native_test_checked_result(name: Value, metadata: Option<Value>, checked: Value) -> Value {
    let Some(entries) = map_entries(&checked) else {
        return native_test_error(
            name,
            Value::Nil,
            "Test/run check function must return a result map".into(),
        );
    };
    if !matches!(
        map_value(&checked, &Value::Keyword("pass".into())),
        Some(Value::Bool(_))
    ) {
        return native_test_error(
            name,
            Value::Nil,
            "Test/run check result requires boolean :pass".into(),
        );
    }
    let mut result = PMap::from_iter(entries);
    result = result.assoc_value(Value::Keyword("name".into()), name);
    if let Some(metadata) = metadata {
        result = result.assoc_value(Value::Keyword("meta".into()), metadata);
    }
    Value::Map(result)
}

fn native_test_run(cases: Value, check_function: Option<Value>) -> Result<Value, String> {
    let cases = match cases {
        Value::Vector(cases) => cases.iter().cloned().collect::<Vec<_>>(),
        Value::Tuple(cases) => cases.iter().cloned().collect::<Vec<_>>(),
        _ => return Err("std.native.Test/run expects a vector of test cases".into()),
    };
    let state = namespace_registry()?.find_or_create("std.native.Test.state");
    let results_symbol = crate::lang::data::Symbol::parse("results");
    let mut results = match state.resolve(&results_symbol).map(|var| var.deref_value()) {
        Some(Value::Vector(results)) => results.iter().cloned().collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    for (index, case) in cases.iter().enumerate() {
        let fallback_name = Value::String(format!("invalid case {}", index + 1));
        let Some(entries) = map_entries(case) else {
            results.push(native_test_error(
                fallback_name,
                Value::Nil,
                "Test/run case must be a map".into(),
            ));
            continue;
        };
        let _ = entries;
        let name = map_value(case, &Value::Keyword("name".into()))
            .cloned()
            .unwrap_or(fallback_name);
        let expected = map_value(case, &Value::Keyword("expected".into())).cloned();
        let test = map_value(case, &Value::Keyword("test".into())).cloned();
        let metadata = map_value(case, &Value::Keyword("meta".into())).cloned();
        let result = match (test, expected) {
            (Some(test), Some(expected)) => match &check_function {
                Some(check) => match call_value(check.clone(), vec![test, expected]) {
                    Ok(checked) => native_test_checked_result(name, metadata, checked),
                    Err(error) => {
                        let failed = native_test_error(name.clone(), Value::Nil, error);
                        native_test_checked_result(name, metadata, failed)
                    }
                },
                None => match call_value(test, Vec::new()).and_then(native_test_await) {
                    Ok(actual) => native_test_result(name, actual, expected),
                    Err(error) => native_test_error(name, expected, error),
                },
            },
            (None, expected) => native_test_error(
                name,
                expected.unwrap_or(Value::Nil),
                "Test/run case requires :test".into(),
            ),
            (Some(_), None) => native_test_error(
                name,
                Value::Nil,
                "Test/run case requires :expected".into(),
            ),
        };
        results.push(result);
    }
    let output = Value::Vector(PVector::from_iter(results));
    state.intern("results", output.clone());
    Ok(output)
}

fn native_test_await(value: Value) -> Result<Value, String> {
    match value {
        Value::Promise(promise) => match promise.wait_state() {
            PromiseState::Fulfilled(value) => Ok(value),
            PromiseState::Rejected(error) => Err(promise_rejection_error(error)),
            PromiseState::Pending => Err("asynchronous test did not settle".into()),
        },
        value => Ok(value),
    }
}

fn native_test_operation(
    operation: &str,
    forms: &[Form],
    env: &mut HashMap<String, Value>,
) -> Result<Value, String> {
    let operation = operation
        .strip_prefix("std.native.Test/")
        .unwrap_or(operation);
    match operation {
        "events" => {
            if !forms.is_empty() {
                return Err("std.native.Test/events expects no arguments".into());
            }
            Ok(native_test_events())
        }
        "catalog" => {
            if !forms.is_empty() {
                return Err("std.native.Test/catalog expects no arguments".into());
            }
            Ok(Value::Map(PMap::from_iter([
                (
                    Value::Keyword("runners".into()),
                    Value::Vector(PVector::from_iter([
                        Value::Keyword("code.test".into()),
                        Value::Keyword("native".into()),
                    ])),
                ),
                (
                    Value::Keyword("default".into()),
                    Value::Keyword("code.test".into()),
                ),
                (
                    Value::Keyword("runner".into()),
                    native_test_active_runner()?,
                ),
                (
                    Value::Keyword("context".into()),
                    Value::Keyword("test".into()),
                ),
                (Value::Keyword("events".into()), native_test_events()),
            ])))
        }
        "config" => {
            if forms.len() > 1 {
                return Err("std.native.Test/config expects optional options".into());
            }
            let options = if forms.is_empty() {
                Value::Map(PMap::new())
            } else {
                eval(&forms[0], env)?
            };
            native_test_config(native_test_active_runner()?, options)
        }
        "context" => {
            if forms.len() > 1 {
                return Err("std.native.Test/context expects an optional config".into());
            }
            let config = if forms.is_empty() {
                native_test_config(native_test_active_runner()?, Value::Map(PMap::new()))?
            } else {
                let value = eval(&forms[0], env)?;
                let Some(runner) = map_value(&value, &Value::Keyword("runner".into())).cloned()
                else {
                    return Err("std.native.Test/context expects a Test/config map".into());
                };
                let runner = native_test_runner(runner)?;
                if runner != native_test_active_runner()? {
                    return Err("std.native.Test/context config runner does not match the runtime".into());
                }
                value
            };
            Ok(Value::Pointer(PPointer::new(
                "test".into(),
                PMap::from_iter([
                    (Value::Keyword("id".into()), Value::Keyword("test".into())),
                    (Value::Keyword("config".into()), config),
                ]),
            )))
        }
        "result" => {
            if forms.len() != 3 {
                return Err("std.native.Test/result expects name, actual, and expected".into());
            }
            let name = eval(&forms[0], env)?;
            let actual = eval(&forms[1], env)?;
            let expected = eval(&forms[2], env)?;
            Ok(native_test_result(name, actual, expected))
        }
        "run" => {
            if forms.is_empty() || forms.len() > 2 {
                return Err(
                    "std.native.Test/run expects cases and an optional check function".into(),
                );
            }
            let cases = eval(&forms[0], env)?;
            let check_function = if forms.len() == 2 {
                Some(eval(&forms[1], env)?)
            } else {
                None
            };
            native_test_run(cases, check_function)
        }
        "passed?" => {
            if forms.len() != 1 {
                return Err("std.native.Test/passed? expects one result".into());
            }
            let result = eval(&forms[0], env)?;
            match map_value(&result, &Value::Keyword("pass".into())) {
                Some(Value::Bool(pass)) => Ok(Value::Bool(*pass)),
                _ => Err("std.native.Test/passed? expects a test result map".into()),
            }
        }
        _ => Err(format!("unknown std.native.Test operation: {operation}")),
    }
}
fn native_regex_operation(
    operation: &str,
    forms: &[Form],
    env: &mut HashMap<String, Value>,
) -> Result<Value, String> {
    let operation = operation
        .strip_prefix("std.native.RegExp/")
        .unwrap_or(operation);
    match operation {
        "instance?" => {
            if forms.len() != 1 {
                return Err("std.native.RegExp/instance? expects one value".into());
            }
            Ok(Value::Bool(matches!(eval(&forms[0], env)?, Value::Regex(_))))
        }
        "compile" => {
            if forms.len() != 1 {
                return Err("std.native.RegExp/compile expects one string".into());
            }
            let pattern = match eval(&forms[0], env)? {
                Value::String(pattern) => pattern,
                _ => return Err("std.native.RegExp/compile expects one string".into()),
            };
            regex::Regex::new(&pattern)
                .map_err(|error| format!("invalid regexp: {error}"))?;
            Ok(Value::Regex(pattern))
        }
        "pattern" => {
            if forms.len() != 1 {
                return Err("std.native.RegExp/pattern expects one regexp".into());
            }
            match eval(&forms[0], env)? {
                Value::Regex(pattern) => Ok(Value::String(pattern)),
                _ => Err("std.native.RegExp/pattern expects one regexp".into()),
            }
        }
        "find?" => {
            if forms.len() != 2 {
                return Err("std.native.RegExp/find? expects a regexp and string".into());
            }
            let pattern = match eval(&forms[0], env)? {
                Value::Regex(pattern) => pattern,
                _ => return Err("std.native.RegExp/find? expects a regexp and string".into()),
            };
            let input = match eval(&forms[1], env)? {
                Value::String(input) => input,
                _ => return Err("std.native.RegExp/find? expects a regexp and string".into()),
            };
            let regexp = regex::Regex::new(&pattern)
                .map_err(|error| format!("invalid regexp: {error}"))?;
            Ok(Value::Bool(regexp.is_match(&input)))
        }
        "find" => {
            if forms.len() != 2 {
                return Err("std.native.RegExp/find expects a regexp and string".into());
            }
            let pattern = match eval(&forms[0], env)? {
                Value::Regex(pattern) => pattern,
                _ => return Err("std.native.RegExp/find expects a regexp and string".into()),
            };
            let input = match eval(&forms[1], env)? {
                Value::String(input) => input,
                _ => return Err("std.native.RegExp/find expects a regexp and string".into()),
            };
            let regexp = regex::Regex::new(&pattern)
                .map_err(|error| format!("invalid regexp: {error}"))?;
            Ok(regexp
                .find(&input)
                .map(|matched| Value::String(matched.as_str().to_owned()))
                .unwrap_or(Value::Nil))
        }
        "matches" => {
            if forms.len() != 2 {
                return Err("std.native.RegExp/matches expects a regexp and string".into());
            }
            let pattern = match eval(&forms[0], env)? {
                Value::Regex(pattern) => pattern,
                _ => return Err("std.native.RegExp/matches expects a regexp and string".into()),
            };
            let input = match eval(&forms[1], env)? {
                Value::String(input) => input,
                _ => return Err("std.native.RegExp/matches expects a regexp and string".into()),
            };
            let anchored = format!(r"\A(?:{pattern})\z");
            let regexp = regex::Regex::new(&anchored)
                .map_err(|error| format!("invalid regexp: {error}"))?;
            Ok(Value::Bool(regexp.is_match(&input)))
        }
        "replace" => {
            if forms.len() != 3 {
                return Err(
                    "std.native.RegExp/replace expects a regexp, string, and replacement".into(),
                );
            }
            let pattern = match eval(&forms[0], env)? {
                Value::Regex(pattern) => pattern,
                _ => {
                    return Err(
                        "std.native.RegExp/replace expects a regexp, string, and replacement"
                            .into(),
                    )
                }
            };
            let input = match eval(&forms[1], env)? {
                Value::String(input) => input,
                _ => {
                    return Err(
                        "std.native.RegExp/replace expects a regexp, string, and replacement"
                            .into(),
                    )
                }
            };
            let replacement = match eval(&forms[2], env)? {
                Value::String(replacement) => replacement,
                _ => {
                    return Err(
                        "std.native.RegExp/replace expects a regexp, string, and replacement"
                            .into(),
                    )
                }
            };
            let regexp = regex::Regex::new(&pattern)
                .map_err(|error| format!("invalid regexp: {error}"))?;
            Ok(Value::String(
                regexp.replace_all(&input, replacement.as_str()).into_owned(),
            ))
        }
        "split" => {
            if forms.len() != 2 {
                return Err("std.native.RegExp/split expects a regexp and string".into());
            }
            let pattern = match eval(&forms[0], env)? {
                Value::Regex(pattern) => pattern,
                _ => return Err("std.native.RegExp/split expects a regexp and string".into()),
            };
            let input = match eval(&forms[1], env)? {
                Value::String(input) => input,
                _ => return Err("std.native.RegExp/split expects a regexp and string".into()),
            };
            let regexp = regex::Regex::new(&pattern)
                .map_err(|error| format!("invalid regexp: {error}"))?;
            Ok(Value::Vector(PVector::from_iter(
                regexp
                    .split(&input)
                    .map(|part| Value::String(part.to_owned())),
            )))
        }
        _ => Err(format!("unknown std.native.RegExp operation: {operation}")),
    }
}

fn file_error(operation: &str, error: FileError) -> String {
    format!("{operation} failed: file/{}", error.code())
}

fn socket_error(operation: &str, error: SocketError) -> String {
    format!("{operation} failed: socket/{}", error.code())
}

fn active_file_provider() -> Option<Rc<dyn FileProvider>> {
    ACTIVE_FILE_PROVIDER.with(|active| active.borrow().clone())
}

fn rejected_file_effect(
    operation: &str,
    path: &str,
    target: Option<&str>,
    error: FileError,
) -> Value {
    let promise = Promise::new();
    promise.reject_value(crate::file::file_error_value(
        operation, path, target, &error,
    ));
    Value::Promise(promise)
}

fn file_effect(
    operation: &str,
    path: &str,
    target: Option<&str>,
    invoke: impl FnOnce(&dyn FileProvider) -> Result<Promise, FileError>,
) -> Value {
    let Some(provider) = active_file_provider() else {
        return rejected_file_effect(operation, path, target, FileError::Denied);
    };
    match invoke(provider.as_ref()) {
        Ok(promise) => Value::Promise(promise),
        Err(error) => rejected_file_effect(operation, path, target, error),
    }
}

fn file_option(options: &Value, name: &str) -> Option<Value> {
    let key = Value::Keyword(name.into());
    map_entries(options)?
        .into_iter()
        .find_map(|(candidate, value)| (candidate == key).then_some(value))
}

fn file_options_value(value: Value, operation: &str) -> Result<Value, String> {
    match value {
        Value::Nil => Ok(Value::Map(PMap::new())),
        value if map_entries(&value).is_some() => Ok(value),
        _ => Err(format!("{operation} options must be a map")),
    }
}

fn file_bool_option(
    options: &Value,
    name: &str,
    default: bool,
    operation: &str,
) -> Result<bool, String> {
    match file_option(options, name) {
        None => Ok(default),
        Some(Value::Bool(value)) => Ok(value),
        Some(_) => Err(format!("{operation} :{name} must be boolean")),
    }
}

fn file_string_option(
    options: &Value,
    name: &str,
    default: &str,
    operation: &str,
) -> Result<String, String> {
    match file_option(options, name) {
        None => Ok(default.into()),
        Some(Value::String(value)) => Ok(value),
        Some(_) => Err(format!("{operation} :{name} must be a string")),
    }
}

fn file_write_options(options: &Value) -> Result<WriteOptions, String> {
    let mode = match file_option(options, "mode") {
        None => WriteMode::Create,
        Some(Value::Keyword(value)) if value.as_str() == "create" => WriteMode::Create,
        Some(Value::Keyword(value)) if value.as_str() == "replace" => WriteMode::Replace,
        Some(Value::Keyword(value)) if value.as_str() == "append" => WriteMode::Append,
        Some(_) => return Err("file/write :mode must be :create, :replace, or :append".into()),
    };
    Ok(WriteOptions {
        mode,
        parents: file_bool_option(options, "parents?", false, "file/write")?,
    })
}

fn file_mkdir_options(options: &Value) -> Result<MkdirOptions, String> {
    Ok(MkdirOptions {
        parents: file_bool_option(options, "parents?", true, "file/mkdir")?,
        exists_ok: file_bool_option(options, "exists-ok?", true, "file/mkdir")?,
    })
}

fn file_delete_options(options: &Value) -> Result<DeleteOptions, String> {
    Ok(DeleteOptions {
        missing_ok: file_bool_option(options, "missing-ok?", false, "file/delete")?,
    })
}

fn file_copy_options(options: &Value) -> Result<CopyOptions, String> {
    Ok(CopyOptions {
        replace: file_bool_option(options, "replace?", false, "file/copy")?,
        parents: file_bool_option(options, "parents?", false, "file/copy")?,
        preserve_modified: file_bool_option(options, "preserve-modified?", false, "file/copy")?,
    })
}

fn file_move_options(options: &Value) -> Result<MoveOptions, String> {
    Ok(MoveOptions {
        replace: file_bool_option(options, "replace?", false, "file/move")?,
        parents: file_bool_option(options, "parents?", false, "file/move")?,
        atomic: file_bool_option(options, "atomic?", false, "file/move")?,
    })
}

fn file_operation(
    operation: &str,
    forms: &[Form],
    env: &mut HashMap<String, Value>,
) -> Result<Value, String> {
    let operation = operation
        .strip_prefix("std.native.File/")
        .map(|method| format!("file/{method}"))
        .unwrap_or_else(|| operation.to_owned());
    let operation = operation.as_str();
    match operation {
        "file/parent" => {
            if forms.len() != 1 {
                return Err("file/parent expects a path".into());
            }
            let Value::String(path) = eval(&forms[0], env)? else {
                return Err("file/parent expects a path".into());
            };
            crate::file::logical_parent(&path)
                .map(|parent| parent.map(Value::String).unwrap_or(Value::Nil))
                .map_err(|error| file_error(operation, error))
        }
        "file/join" | "file/resolve" => {
            if forms.len() != 2 {
                return Err(format!("{operation} expects a base and path"));
            }
            let Value::String(base) = eval(&forms[0], env)? else {
                return Err(format!("{operation} expects a base and path"));
            };
            let Value::String(path) = eval(&forms[1], env)? else {
                return Err(format!("{operation} expects a base and path"));
            };
            let result = if operation == "file/join" {
                crate::file::logical_join(&base, &path)
            } else {
                crate::file::logical_resolve(&base, &path)
            };
            result
                .map(Value::String)
                .map_err(|error| file_error(operation, error))
        }
        "file/read" | "file/exists?" | "file/stat" | "file/entries" | "file/list" | "file/walk" => {
            if forms.len() != 1 {
                return Err(format!("{operation} expects a path"));
            }
            let Value::String(path) = eval(&forms[0], env)? else {
                return Err(format!("{operation} expects a path"));
            };
            Ok(file_effect(
                operation,
                &path,
                None,
                |provider| match operation {
                    "file/read" => provider.read(&path),
                    "file/exists?" => provider.exists(&path),
                    "file/stat" => provider.stat(&path),
                    "file/entries" => provider.entries(&path),
                    "file/list" => provider.list(&path),
                    "file/walk" => provider.walk(&path),
                    _ => unreachable!(),
                },
            ))
        }
        "file/write" => {
            if !(2..=3).contains(&forms.len()) {
                return Err("file/write expects a path, bytes, and optional options".into());
            }
            let Value::String(path) = eval(&forms[0], env)? else {
                return Err("file/write expects a path and bytes".into());
            };
            let bytes = match eval(&forms[1], env)? {
                Value::Bytes(value) => value,
                Value::ByteBuffer(value) => value.borrow().clone(),
                _ => return Err("file/write expects a path and bytes".into()),
            };
            let options = if forms.len() == 3 {
                file_options_value(eval(&forms[2], env)?, operation)?
            } else {
                Value::Map(PMap::new())
            };
            let options = file_write_options(&options)?;
            Ok(file_effect(operation, &path, None, |provider| {
                provider.write_with_options(&path, bytes, options)
            }))
        }
        "file/mkdir" => {
            if !(1..=2).contains(&forms.len()) {
                return Err("file/mkdir expects a path and optional options".into());
            }
            let Value::String(path) = eval(&forms[0], env)? else {
                return Err("file/mkdir expects a path".into());
            };
            let options = if forms.len() == 2 {
                file_options_value(eval(&forms[1], env)?, operation)?
            } else {
                Value::Map(PMap::new())
            };
            let options = file_mkdir_options(&options)?;
            Ok(file_effect(operation, &path, None, |provider| {
                provider.mkdir_with_options(&path, options)
            }))
        }
        "file/delete" => {
            if !(1..=2).contains(&forms.len()) {
                return Err("file/delete expects a path and optional options".into());
            }
            let Value::String(path) = eval(&forms[0], env)? else {
                return Err("file/delete expects a path".into());
            };
            let options = if forms.len() == 2 {
                file_options_value(eval(&forms[1], env)?, operation)?
            } else {
                Value::Map(PMap::new())
            };
            let options = file_delete_options(&options)?;
            Ok(file_effect(operation, &path, None, |provider| {
                provider.delete_with_options(&path, options)
            }))
        }
        "file/copy" | "file/move" => {
            if !(2..=3).contains(&forms.len()) {
                return Err(format!(
                    "{operation} expects source, target, and optional options"
                ));
            }
            let Value::String(source) = eval(&forms[0], env)? else {
                return Err(format!("{operation} expects source and target paths"));
            };
            let Value::String(target) = eval(&forms[1], env)? else {
                return Err(format!("{operation} expects source and target paths"));
            };
            let options = if forms.len() == 3 {
                file_options_value(eval(&forms[2], env)?, operation)?
            } else {
                Value::Map(PMap::new())
            };
            Ok(if operation == "file/copy" {
                let options = file_copy_options(&options)?;
                file_effect(operation, &source, Some(&target), |provider| {
                    provider.copy(&source, &target, options)
                })
            } else {
                let options = file_move_options(&options)?;
                file_effect(operation, &source, Some(&target), |provider| {
                    provider.move_entry(&source, &target, options)
                })
            })
        }
        "file/temp-file" | "file/temp-directory" => {
            if !(1..=2).contains(&forms.len()) {
                return Err(format!("{operation} expects a parent and optional options"));
            }
            let Value::String(parent) = eval(&forms[0], env)? else {
                return Err(format!("{operation} expects a parent path"));
            };
            let options = if forms.len() == 2 {
                file_options_value(eval(&forms[1], env)?, operation)?
            } else {
                Value::Map(PMap::new())
            };
            Ok(if operation == "file/temp-file" {
                let options = TempFileOptions {
                    prefix: file_string_option(&options, "prefix", "tmp", operation)?,
                    suffix: file_string_option(&options, "suffix", "", operation)?,
                };
                file_effect(operation, &parent, None, |provider| {
                    provider.temp_file(&parent, options)
                })
            } else {
                let options = TempDirectoryOptions {
                    prefix: file_string_option(&options, "prefix", "tmp", operation)?,
                };
                file_effect(operation, &parent, None, |provider| {
                    provider.temp_directory(&parent, options)
                })
            })
        }
        _ => Err(format!("unknown std.native.File operation: {operation}")),
    }
}

fn socket_operation(
    operation: &str,
    forms: &[Form],
    env: &mut HashMap<String, Value>,
) -> Result<Value, String> {
    let operation = operation
        .strip_prefix("std.native.Socket/")
        .unwrap_or(operation);
    match operation {
        "receive-stream" | "socket/receive-stream" => {
            if forms.len() != 1 { return Err(format!("Socket/{operation} expects a socket connection")); }
            let socket = socket_handle(&eval(&forms[0], env)?, &format!("Socket/{operation}"))?;
            let events = socket_provider(operation)?.events(socket).map_err(|e| socket_error(operation, e))?;
            Ok(host_stream(Rc::new(move || socket_receive_promise(events)), Rc::new(|| Ok(()))))
        }
        "socket/connect" => {
            if forms.len() != 4 {
                return Err("socket/connect expects a host, port, options, and callback".into());
            }
            let host = match eval(&forms[0], env)? {
                Value::String(value) => value,
                _ => {
                    return Err("socket/connect expects a host, port, options, and callback".into())
                }
            };
            let port_value = eval(&forms[1], env)?;
            let port = value_u16_integer(&port_value, "socket/connect", false)?;
            let _options = eval(&forms[2], env)?;
            let callback = match eval(&forms[3], env)? {
                Value::Function(value) => value,
                _ => return Err("socket/connect expects a callback".into()),
            };
            let callback = Rc::new(move |event| {
                let arguments = match event {
                    SocketEvent::Connected(handle) => {
                        vec![Value::Nil, Value::Number(handle as i64)]
                    }
                    SocketEvent::Failed(_, error) => vec![Value::String(error), Value::Nil],
                    SocketEvent::Data(_, _) | SocketEvent::Closed(_) => return,
                };
                let _ = call_function(&callback, arguments);
            });
            socket_provider(operation)?
                .connect(&host, port, callback)
                .map(|handle| Value::Number(handle as i64))
                .map_err(|error| socket_error(operation, error))
        }
        "socket/listen" => {
            if forms.len() != 4 {
                return Err("socket/listen expects a host, port, options, and callback".into());
            }
            let host = match eval(&forms[0], env)? {
                Value::String(value) => value,
                _ => return Err("socket/listen expects a host string".into()),
            };
            let port_value = eval(&forms[1], env)?;
            let port = value_u16_integer(&port_value, "socket/listen", true)?;
            let _options = eval(&forms[2], env)?;
            let callback = match eval(&forms[3], env)? {
                Value::Function(value) => value,
                _ => return Err("socket/listen expects a callback".into()),
            };
            let callback = Rc::new(move |event| {
                let _ = call_function(&callback, vec![socket_server_event_value(event)]);
            });
            socket_provider(operation)?
                .listen(&host, port, callback)
                .map(|handle| Value::Number(handle as i64))
                .map_err(|error| socket_error(operation, error))
        }
        "socket/endpoint" => {
            if forms.len() != 1 {
                return Err("socket/endpoint expects a server".into());
            }
            let server = socket_handle(&eval(&forms[0], env)?, "socket/endpoint")?;
            socket_provider(operation)?
                .endpoint(server)
                .map(|(host, port)| {
                    Value::Map(PMap::from_iter([
                        (Value::Keyword("host".into()), Value::String(host)),
                        (Value::Keyword("port".into()), Value::Number(port as i64)),
                    ]))
                })
                .map_err(|error| socket_error(operation, error))
        }
        "socket/events" => {
            if forms.len() != 2 {
                return Err("socket/events expects a socket handle and options".into());
            }
            let handle = socket_handle(&eval(&forms[0], env)?, "socket/events")?;
            let _options = eval(&forms[1], env)?;
            socket_provider(operation)?
                .events(handle)
                .map(|stream| Value::Number(stream as i64))
                .map_err(|error| socket_error(operation, error))
        }
        "socket/next" => {
            if forms.len() != 1 {
                return Err("socket/next expects a socket stream".into());
            }
            let stream = socket_handle(&eval(&forms[0], env)?, "socket/next")?;
            socket_provider(operation)?
                .next(stream)
                .map(Value::Promise)
                .map_err(|error| socket_error(operation, error))
        }
        "socket/send" => {
            if forms.len() != 2 {
                return Err("socket/send expects a socket connection and bytes".into());
            }
            let socket_value = eval(&forms[0], env)?;
            let socket = socket_handle(&socket_value, "socket/send")?;
            let bytes = match eval(&forms[1], env)? {
                Value::Bytes(value) => value,
                Value::ByteBuffer(value) => value.borrow().clone(),
                _ => return Err("socket/send expects a socket connection and bytes".into()),
            };
            socket_provider(operation)?
                .send(socket, &bytes)
                .map(|count| Value::Number(count as i64))
                .map_err(|error| socket_error(operation, error))
        }
        "socket/close" => {
            if forms.len() != 1 {
                return Err("socket/close expects a socket connection".into());
            }
            let socket_value = eval(&forms[0], env)?;
            let socket = socket_handle(&socket_value, "socket/close")?;
            socket_provider(operation)?
                .close(socket)
                .map(|()| Value::Nil)
                .map_err(|error| socket_error(operation, error))
        }
        _ => Err(format!("unknown std.native.Socket operation: {operation}")),
    }
}

fn socket_receive_promise(stream: SocketHandle) -> Result<Promise, String> {
    let source = socket_provider("Socket/receive-stream")?.next(stream).map_err(|e| socket_error("Socket/receive-stream", e))?;
    let output = Promise::new();
    let settled = output.clone();
    source.on_settle(Rc::new(move |result| match result {
        PromiseState::Rejected(error) => { settled.reject_rejection(error); }
        PromiseState::Pending => {}
        PromiseState::Fulfilled(event) => {
            let entries = map_entries(&event).unwrap_or_default();
            let kind = entries.iter().find_map(|(k, v)| if matches!(k, Value::Keyword(key) if key.as_str() == "type") { Some(v.clone()) } else { None });
            match kind {
                Some(Value::Keyword(kind)) if kind.as_str() == "data" => {
                    let bytes = entries.into_iter().find_map(|(k, v)| if matches!(k, Value::Keyword(key) if key.as_str() == "bytes") { Some(v) } else { None }).unwrap_or(Value::Nil);
                    settled.resolve(bytes);
                }
                Some(Value::Keyword(kind)) if kind.as_str() == "close" => { settled.resolve(Value::Nil); }
                Some(Value::Keyword(kind)) if kind.as_str() == "error" => { settled.reject("socket receive failed"); }
                _ => { settled.reject("Socket/receive-stream received an invalid event"); }
            }
        }
    }));
    let poll = source.clone(); output.set_poller(Rc::new(move || { poll.state(); }));
    let wait = source.clone(); output.set_waiter(Rc::new(move || { wait.wait_state(); }));
    Ok(output)
}

fn socket_handle(value: &Value, operation: &str) -> Result<SocketHandle, String> {
    value_u64_integer(value, operation)
        .map(|value| value as SocketHandle)
        .map_err(|_| format!("{operation} expects a socket handle"))
}

fn native_host_operation(
    operation: &str,
    forms: &[Form],
    env: &mut HashMap<String, Value>,
) -> Result<Value, String> {
    let method = operation
        .strip_prefix("std.native.Host/")
        .unwrap_or(operation);
    let (service, target, arguments) = match method {
        "call" => {
            if forms.len() != 3 {
                return Err(
                    "std.native.Host/call expects service, method, and an argument vector".into(),
                );
            }
            let service = match eval(&forms[0], env)? {
                Value::String(value) => value,
                _ => return Err("std.native.Host/call service must be a string".into()),
            };
            let target = match eval(&forms[1], env)? {
                Value::String(value) => value,
                _ => return Err("std.native.Host/call method must be a string".into()),
            };
            let arguments = match eval(&forms[2], env)? {
                Value::Vector(values) => values.iter().cloned().collect(),
                Value::Tuple(values) => values.iter().cloned().collect(),
                _ => return Err("std.native.Host/call arguments must be a vector".into()),
            };
            (service, target, arguments)
        }
        "describe" | "capabilities" => {
            if !forms.is_empty() {
                return Err(format!("std.native.Host/{method} expects no arguments"));
            }
            ("host".into(), method.into(), Vec::new())
        }
        "capability?" => {
            if forms.len() != 1 {
                return Err("std.native.Host/capability? expects one capability".into());
            }
            (
                "host".into(),
                "capability?".into(),
                vec![eval(&forms[0], env)?],
            )
        }
        _ => return Err(format!("unknown std.native.Host method: {method}")),
    };
    HOST_CALL_HANDLER.with(|active| {
        let Some(handler) = active.borrow().as_ref().cloned() else {
            let promise = Promise::new();
            promise.reject_value(host_error(
                "host/unavailable",
                "Host capability provider is unavailable",
            ));
            return Ok(Value::Promise(promise));
        };
        handler(service, target, arguments)
    })
}

fn namespace_identifier(value: Value, operation: &str) -> Result<String, String> {
    match value {
        Value::Symbol(name) if name.get_namespace().is_none() => Ok(name.as_str().to_owned()),
        Value::String(name) => Ok(name),
        Value::Namespace(namespace) => Ok(namespace.name().as_str().to_owned()),
        _ => Err(format!(
            "{operation} expects an unqualified namespace symbol, string, or Namespace"
        )),
    }
}

fn namespace_descriptor(registry: &NamespaceRegistry<Value>, name: &str) -> Value {
    let state = registry
        .load_state(name)
        .or_else(|| registry.find(name).map(|_| NamespaceLoadState::Loaded))
        .map(NamespaceLoadState::as_str)
        .unwrap_or("unknown");
    let package = package_catalog().coordinate_for_namespace(name);
    let origin = if name.starts_with("std.native") {
        "embedded"
    } else if package.is_some() {
        "package"
    } else if registry.find(name).is_some() {
        "runtime"
    } else {
        "registered"
    };
    let mut fields = vec![
        (
            Value::Keyword("namespace/name".into()),
            Value::Symbol(Symbol::parse(name)),
        ),
        (
            Value::Keyword("namespace/state".into()),
            Value::Keyword(state.into()),
        ),
        (
            Value::Keyword("namespace/revision".into()),
            Value::Number(registry.module_revision(name) as i64),
        ),
        (
            Value::Keyword("namespace/origin".into()),
            Value::Keyword(origin.into()),
        ),
    ];
    if let Some(package) = package {
        fields.push((
            Value::Keyword("namespace/package".into()),
            Value::String(package),
        ));
    }
    Value::OrderedMap(Box::new(POrderedMap::from_iter(fields)))
}

fn native_runtime_operation(
    operation: &str,
    forms: &[Form],
    env: &mut HashMap<String, Value>,
) -> Result<Value, String> {
    let method = operation
        .strip_prefix("std.native.Runtime/")
        .unwrap_or(operation);
    let registry = namespace_registry()?;
    match method {
        "current" => {
            if !forms.is_empty() {
                return Err("std.native.Runtime/current expects no arguments".into());
            }
            Ok(Value::Symbol(registry.current().name().clone()))
        }
        "snapshot" => {
            if !forms.is_empty() {
                return Err("std.native.Runtime/snapshot expects no arguments".into());
            }
            let namespaces = registry
                .known_names()
                .into_iter()
                .map(|name| namespace_descriptor(&registry, name.as_str()))
                .collect::<Vec<_>>();
            Ok(Value::OrderedMap(Box::new(POrderedMap::from_iter([
                (
                    Value::Keyword("env/current".into()),
                    Value::Symbol(registry.current().name().clone()),
                ),
                (
                    Value::Keyword("env/namespaces".into()),
                    Value::Vector(PVector::from(namespaces)),
                ),
            ]))))
        }
        "namespaces" => {
            if !forms.is_empty() {
                return Err("std.native.Runtime/namespaces expects no arguments".into());
            }
            Ok(Value::Vector(PVector::from(
                registry
                    .known_names()
                    .into_iter()
                    .map(|name| namespace_descriptor(&registry, name.as_str()))
                    .collect::<Vec<_>>(),
            )))
        }
        "namespace" => {
            if forms.len() != 1 {
                return Err("std.native.Runtime/namespace expects one namespace".into());
            }
            let name = namespace_identifier(eval(&forms[0], env)?, operation)?;
            if registry.load_state(&name).is_none() && registry.find(&name).is_none() {
                Ok(Value::Nil)
            } else {
                Ok(namespace_descriptor(&registry, &name))
            }
        }
        "module" => {
            if forms.len() != 1 {
                return Err("std.native.Runtime/module expects one module path".into());
            }
            let requested = match eval(&forms[0], env)? {
                Value::String(path) => path,
                Value::Symbol(name) => name.as_str().to_owned(),
                _ => return Err("std.native.Runtime/module expects a path string or namespace symbol".into()),
            };
            let source = requested.strip_prefix("classpath:").unwrap_or(&requested);
            let namespace = if source.ends_with(".hal") || source.ends_with(".hrl") {
                source
                    .trim_end_matches(".hal")
                    .trim_end_matches(".hrl")
                    .trim_start_matches("./")
                    .replace('/', ".")
            } else {
                source.to_owned()
            };
            let revision = registry.module_revision(&namespace);
            if revision == 0
                && registry.load_state(&namespace).is_none()
                && registry.find(&namespace).is_none()
            {
                return Ok(Value::Nil);
            }
            let dependencies = registry
                .module_dependencies(&namespace)
                .into_iter()
                .map(|dependency| {
                    Value::String(format!("{}.hal", dependency.as_str().replace('.', "/")))
                })
                .collect::<Vec<_>>();
            Ok(Value::OrderedMap(Box::new(POrderedMap::from_iter([
                (Value::Keyword("module/path".into()), Value::String(requested)),
                (
                    Value::Keyword("module/namespace".into()),
                    Value::Symbol(Symbol::parse(&namespace)),
                ),
                (
                    Value::Keyword("module/revision".into()),
                    Value::Number(revision as i64),
                ),
                (
                    Value::Keyword("module/dependencies".into()),
                    Value::Vector(PVector::from(dependencies)),
                ),
            ]))))
        }
        "vars" => {
            if forms.len() > 1 {
                return Err("std.native.Runtime/vars expects zero or one namespace".into());
            }
            let name = if forms.is_empty() {
                registry.current().name().as_str().to_owned()
            } else {
                namespace_identifier(eval(&forms[0], env)?, operation)?
            };
            let namespace = registry
                .find(&name)
                .ok_or_else(|| format!("namespace/not-found: {name}"))?;
            let mut mappings = namespace.mappings();
            mappings.retain(|(_, var)| var.symbol().get_namespace() == Some(name.as_str()));
            mappings.sort_by(|(left, _), (right, _)| left.as_str().cmp(right.as_str()));
            Ok(Value::OrderedMap(Box::new(POrderedMap::from_iter(
                mappings.into_iter().map(|(symbol, var)| {
                    (
                        Value::Symbol(Symbol::create(None, symbol.as_str())),
                        Value::Var(var),
                    )
                }),
            ))))
        }
        "resolve" => {
            if forms.len() != 1 {
                return Err("std.native.Runtime/resolve expects one symbol".into());
            }
            let Value::Symbol(symbol) = eval(&forms[0], env)? else {
                return Err("std.native.Runtime/resolve expects a symbol".into());
            };
            // Deliberately bypass force_lazy_alias: Runtime inspection must never
            // load source or invoke a package provider.
            Ok(registry.resolve(&symbol).map(Value::Var).unwrap_or(Value::Nil))
        }
        "eval" => {
            if forms.len() != 1 {
                return Err("std.native.Runtime/eval expects one form".into());
            }
            eval_value(eval(&forms[0], env)?, env)
        }
        "alias-state" | "intern-var" | "eval-in" => {
            let legacy = match method {
                "alias-state" => "ns-alias-state",
                "intern-var" => "intern-var",
                _ => "eval-in-ns",
            };
            let mut delegated = Vec::with_capacity(forms.len() + 1);
            delegated.push(Form::Symbol(legacy.to_owned()));
            delegated.extend_from_slice(forms);
            eval(&Form::List(delegated), env)
        }
        _ => Err(format!("unknown std.native.Runtime method: {method}")),
    }
}

fn eval_value(value: Value, env: &mut HashMap<String, Value>) -> Result<Value, String> {
    eval(&value_to_form(&value)?, env)
}

fn native_package_operation(
    operation: &str,
    forms: &[Form],
    env: &mut HashMap<String, Value>,
) -> Result<Value, String> {
    let method = operation
        .strip_prefix("std.native.Package/")
        .unwrap_or(operation);
    let expected = match method {
        "catalog" => 0..=0,
        "find" | "ensure" | "load" | "state" => 1..=1,
        "unload" => 1..=2,
        _ => return Err(format!("unknown std.native.Package method: {method}")),
    };
    if !expected.contains(&forms.len()) {
        return Err(format!(
            "std.native.Package/{method} expects {} arguments",
            expected.start()
        ));
    }
    let arguments = forms
        .iter()
        .map(|form| eval(form, env))
        .collect::<Result<Vec<_>, _>>()?;
    let catalog = package_catalog();
    if method == "catalog" {
        return Ok(catalog.catalog_value());
    }
    let target = match arguments.first() {
        Some(Value::Symbol(value)) => value.as_str().to_owned(),
        Some(Value::String(value)) => value.clone(),
        Some(Value::Keyword(value)) => value.as_str().to_owned(),
        Some(value @ Value::OrderedMap(_)) if method == "ensure" || method == "unload" =>
            package_descriptor_coordinate(value).ok_or_else(|| {
                format!("std.native.Package/{method} descriptor requires :package/coordinate")
            })?,
        _ => return Err(format!("std.native.Package/{method} expects a namespace, coordinate, or exact descriptor")),
    };
    let found = catalog.find(&target);
    if method == "find" {
        return Ok(found.map(|(_, value)| value).unwrap_or(Value::Nil));
    }
    let Some((coordinate, descriptor)) = found else {
        if method == "state" {
            return Ok(Value::Nil);
        }
        return Err(format!("package/not-locked: {target}"));
    };
    if method == "state" {
        return Ok(Value::Keyword(
            catalog.state(&coordinate).unwrap_or_else(|| "available".into()).into(),
        ));
    }
    if method == "load" {
        if catalog.coordinate_for_namespace(&target).as_deref() != Some(&coordinate) {
            return Err("std.native.Package/load expects a locked namespace".into());
        }
        if catalog.state(&coordinate).as_deref() != Some("ready") {
            return Err(format!("package/not-ready: {coordinate}; call Package/ensure first"));
        }
        let registry = namespace_registry()?;
        require_namespace(&registry, env, &target)?;
        return Ok(Value::Symbol(Symbol::parse(&target)));
    }
    if method == "ensure" {
        if catalog.state(&coordinate).as_deref() == Some("ready") {
            let promise = Promise::new();
            promise.resolve(descriptor);
            return Ok(Value::Promise(promise));
        }
        if let Some(pending) = catalog.pending(&coordinate) {
            return Ok(Value::Promise(pending));
        }
    } else if catalog.state(&coordinate).as_deref() == Some("available") {
        let promise = Promise::new();
        promise.resolve(Value::Vector(PVector::new()));
        return Ok(Value::Promise(promise));
    } else if catalog.pending(&coordinate).is_some() {
        return Err(format!("package/busy: {coordinate}"));
    }
    if method == "unload" {
        if let Some(options) = arguments.get(1) {
            if map_entries(options).is_none() {
                return Err("std.native.Package/unload options must be a map".into());
            }
            if let Some(value) = map_value(options, &Value::Keyword("cascade".into())) {
                if !matches!(value, Value::Bool(_)) {
                    return Err("std.native.Package/unload :cascade must be boolean".into());
                }
            }
        }
    }
    let previous_state = catalog.state(&coordinate).unwrap_or_else(|| "available".into());
    catalog.set_state(&coordinate, if method == "ensure" { "ensuring" } else { "unloading" });
    HOST_CALL_HANDLER.with(|active| {
        let Some(handler) = active.borrow().as_ref().cloned() else {
            let promise = Promise::new();
            promise.reject_value(host_error(
                "package/unsupported",
                "Package capability provider is unavailable",
            ));
            catalog.set_state(&coordinate, if method == "ensure" { "failed" } else { &previous_state });
            return Ok(Value::Promise(promise));
        };
        let mut provider_arguments = vec![descriptor];
        provider_arguments.extend(arguments.iter().skip(1).cloned());
        let result = handler("package".into(), method.into(), provider_arguments);
        if let Ok(Value::Promise(promise)) = &result {
            let state = catalog.clone();
            let coordinate = coordinate.clone();
            let operation = method.to_owned();
            let rollback = previous_state.clone();
            state.set_pending(&coordinate, Some(promise.clone()));
            promise.on_settle(Rc::new(move |settlement| {
                let next = match (&operation[..], settlement) {
                    ("ensure", PromiseState::Fulfilled(_)) => "ready",
                    ("ensure", _) => "failed",
                    ("unload", PromiseState::Fulfilled(_)) => "available",
                    ("unload", _) => rollback.as_str(),
                    _ => rollback.as_str(),
                };
                state.set_state(&coordinate, next);
                state.set_pending(&coordinate, None);
            }));
        } else if result.is_ok() {
            catalog.set_state(&coordinate, if method == "ensure" { "ready" } else { "available" });
        } else {
            catalog.set_state(&coordinate, if method == "ensure" { "failed" } else { &previous_state });
        }
        result
    })
}

/// Invokes the active host capability provider with already-evaluated VM
/// values. This is the bytecode boundary for `std.native.Host/call`; the VM
/// remains unaware of timers, sockets, or any other concrete host operation.
pub fn call_host_value(service: Value, target: Value, arguments: Value) -> Result<Value, String> {
    let service = match service {
        Value::String(value) => value,
        _ => return Err("std.native.Host/call service must be a string".into()),
    };
    let target = match target {
        Value::String(value) => value,
        _ => return Err("std.native.Host/call method must be a string".into()),
    };
    let arguments = match arguments {
        Value::Vector(values) => values.iter().cloned().collect(),
        Value::Tuple(values) => values.iter().cloned().collect(),
        _ => return Err("std.native.Host/call arguments must be a vector".into()),
    };
    HOST_CALL_HANDLER.with(|active| {
        let Some(handler) = active.borrow().as_ref().cloned() else {
            let promise = Promise::new();
            promise.reject_value(host_error(
                "host/unavailable",
                "Host capability provider is unavailable",
            ));
            return Ok(Value::Promise(promise));
        };
        handler(service, target, arguments)
    })
}

fn host_error(code: &str, message: &str) -> Value {
    Value::ExceptionInfo(Rc::new(ExceptionInfo {
        message: message.into(),
        data: Box::new(Value::Map(
            vec![(
                Value::Keyword("error/code".into()),
                Value::Keyword(code.into()),
            )]
            .into_iter()
            .collect(),
        )),
        cause: None,
    }))
}
/// Installs the explicit host-call boundary for one evaluation.
pub fn with_host_calls<R>(
    handler: Rc<dyn Fn(String, String, Vec<Value>) -> Result<Value, String>>,
    operation: impl FnOnce() -> R,
) -> R {
    HOST_CALL_HANDLER.with(|active| {
        let previous = active.replace(Some(handler));
        let result = operation();
        active.replace(previous);
        result
    })
}

/// Runs an evaluation with a source provider used to satisfy `require` loads.
pub(crate) fn with_namespace_source<R>(
    provider: Rc<dyn Fn(&str) -> Option<NamespaceResource>>,
    action: impl FnOnce() -> R,
) -> R {
    NAMESPACE_SOURCE_PROVIDER.with(|active| {
        let previous = active.borrow_mut().replace(provider);
        let result = action();
        *active.borrow_mut() = previous;
        result
    })
}
