//! Emit deterministic API documentation data from native Hara source forms.

use hara_wasm::kernel::{read_forms, Form};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
struct Definition {
    name: String,
    kind: String,
    doc: String,
    signature: String,
    line: usize,
}

#[derive(Debug)]
struct Namespace {
    name: String,
    source: String,
    definitions: Vec<Definition>,
    examples: Vec<String>,
}

fn symbol(form: &Form) -> Option<&str> {
    match form {
        Form::Symbol(value) => Some(value),
        Form::Metadata(_, value) => symbol(value),
        _ => None,
    }
}

fn metadata_is_private(form: &Form) -> bool {
    match form {
        Form::Metadata(meta, _) => match meta.as_ref() {
            Form::Keyword(value) => value == "private",
            Form::Map(entries) => entries.iter().any(|(key, value)| {
                matches!(key, Form::Keyword(key) if key == "private")
                    && !matches!(value, Form::Bool(false) | Form::Nil)
            }),
            _ => false,
        },
        _ => false,
    }
}

fn definition(form: &Form, line: usize) -> Option<Definition> {
    let Form::List(items) = form else { return None };
    let kind = symbol(items.first()?)?;
    if !matches!(
        kind,
        "def" | "defn" | "defmacro" | "defprotocol" | "deftype" | "defstruct"
    ) {
        return None;
    }
    let name_form = items.get(1)?;
    let name = symbol(name_form)?.to_owned();
    if name.starts_with('-') || kind.ends_with('-') || metadata_is_private(name_form) {
        return None;
    }
    let mut cursor = 2;
    let doc = match items.get(cursor) {
        Some(Form::String(value)) => {
            cursor += 1;
            value.clone()
        }
        _ => String::new(),
    };
    while matches!(items.get(cursor), Some(Form::Map(_))) {
        cursor += 1;
    }
    let signature = match items.get(cursor) {
        Some(Form::Vector(_)) => items[cursor].to_string(),
        Some(Form::List(arities)) => arities.iter()
            .filter(|arity| matches!(arity, Form::List(parts) if matches!(parts.first(), Some(Form::Vector(_)))))
            .map(|arity| match arity { Form::List(parts) => parts[0].to_string(), _ => String::new() })
            .collect::<Vec<_>>().join(" "),
        _ => String::new(),
    };
    Some(Definition {
        name,
        kind: kind.to_owned(),
        doc,
        signature,
        line,
    })
}

fn namespace_builtins(form: &Form, line: usize) -> Vec<Definition> {
    let Form::List(namespace) = form else {
        return Vec::new();
    };
    if namespace.first().and_then(symbol) != Some("ns") {
        return Vec::new();
    }
    namespace
        .iter()
        .skip(2)
        .find_map(|clause| {
            let Form::List(items) = clause else {
                return None;
            };
            if !matches!(items.first(), Some(Form::Keyword(key)) if key == "config") {
                return None;
            }
            let Form::Map(config) = items.get(1)? else {
                return None;
            };
            let builtins = config.iter().find_map(|(key, value)| {
                matches!(key, Form::Keyword(key) if key == "builtins").then_some(value)
            })?;
            let Form::Vector(names) = builtins else {
                return None;
            };
            Some(
                names
                    .iter()
                    .filter_map(symbol)
                    .map(|name| Definition {
                        name: name.to_owned(),
                        kind: "builtin".into(),
                        doc: "Native constructor activated by this namespace.".into(),
                        signature: String::new(),
                        line,
                    })
                    .collect(),
            )
        })
        .unwrap_or_default()
}

fn namespace_name(path: &Path, source_root: &Path) -> String {
    path.strip_prefix(source_root)
        .unwrap_or(path)
        .with_extension("")
        .components()
        .map(|part| part.as_os_str().to_string_lossy().replace('_', "-"))
        .collect::<Vec<_>>()
        .join(".")
}

fn json(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn api_sources(root: &Path) -> Vec<PathBuf> {
    let base = root.join("std");
    let mut files = Vec::new();
    fn visit(path: &Path, files: &mut Vec<PathBuf>) {
        if path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    visit(&entry.path(), files);
                }
            }
        } else if path.extension().and_then(|v| v.to_str()) == Some("hal") {
            files.push(path.to_owned());
        }
    }
    visit(&base.join("foundation"), &mut files);
    files.push(base.join("foundation.hal"));
    files.push(base.join("lib/collection.hal"));
    files.sort();
    files
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        return Err("usage: hara-api-doc SOURCE_ROOT TEST_ROOT".into());
    }
    let source_root = Path::new(&args[1]);
    let test_root = Path::new(&args[2]);
    let mut namespaces = Vec::new();
    for path in api_sources(source_root) {
        let source = fs::read_to_string(&path)?;
        let name = namespace_name(&path, source_root);
        let forms = read_forms(&source)?;
        let mut definitions = forms
            .iter()
            .filter_map(|spanned| definition(&spanned.form, spanned.span.start.line))
            .collect::<Vec<_>>();
        definitions.extend(
            forms
                .iter()
                .flat_map(|spanned| namespace_builtins(&spanned.form, spanned.span.start.line)),
        );
        definitions.sort_by_key(|definition| definition.line);
        let relative = path
            .strip_prefix(source_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        let test_path = test_root.join(relative.replace(".hal", "_test.hal"));
        let examples = if test_path.exists() {
            read_forms(&fs::read_to_string(test_path)?)?
                .into_iter()
                .filter_map(|spanned| {
                    let Form::List(items) = &spanned.form else {
                        return None;
                    };
                    (items.first().and_then(symbol) == Some("fact"))
                        .then(|| spanned.form.to_string())
                })
                .collect()
        } else {
            Vec::new()
        };
        namespaces.push(Namespace {
            name,
            source: relative,
            definitions,
            examples,
        });
    }
    print!("{{\"schemaVersion\":1,\"namespaces\":[");
    for (ni, ns) in namespaces.iter().enumerate() {
        if ni > 0 {
            print!(",");
        }
        print!(
            "{{\"name\":{},\"source\":{},\"definitions\":[",
            json(&ns.name),
            json(&ns.source)
        );
        for (di, def) in ns.definitions.iter().enumerate() {
            if di > 0 {
                print!(",");
            }
            print!(
                "{{\"name\":{},\"kind\":{},\"doc\":{},\"signature\":{},\"line\":{}}}",
                json(&def.name),
                json(&def.kind),
                json(&def.doc),
                json(&def.signature),
                def.line
            );
        }
        print!("],\"examples\":[");
        for (ei, example) in ns.examples.iter().enumerate() {
            if ei > 0 {
                print!(",");
            }
            print!("{}", json(example));
        }
        print!("]}}");
    }
    println!("]}}");
    Ok(())
}
