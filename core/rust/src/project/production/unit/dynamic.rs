use super::super::plan::BuildPlan;
use super::super::source::Diagnostic;
use super::{qualify, without_metadata, UnitAnalysis};
use crate::kernel::Form;
use crate::lang::data::Symbol;
use crate::lang::protocol::INamespaced;
use crate::Runtime;
use std::collections::BTreeSet;

pub(super) fn collect_resolved_symbols(
    runtime: &Runtime,
    module: &str,
    form: &Form,
    output: &mut BTreeSet<String>,
) {
    match form {
        Form::Symbol(name) => {
            if let Some(resolved) = resolve_existing_symbol(runtime, module, name) {
                output.insert(resolved);
            }
        }
        Form::List(values) => {
            if matches!(values.first(), Some(Form::Symbol(head)) if head == "quote") {
                return;
            }
            for value in values {
                collect_resolved_symbols(runtime, module, value, output);
            }
        }
        Form::Vector(values) | Form::Set(values) => {
            for value in values {
                collect_resolved_symbols(runtime, module, value, output);
            }
        }
        Form::Map(entries) => {
            for (key, value) in entries {
                collect_resolved_symbols(runtime, module, key, output);
                collect_resolved_symbols(runtime, module, value, output);
            }
        }
        Form::Metadata(_, value) | Form::Tagged(_, value) => {
            collect_resolved_symbols(runtime, module, value, output);
        }
        _ => {}
    }
}

pub(super) fn scan_dynamic_access(
    runtime: &Runtime,
    module: &str,
    form: &Form,
    plan: &BuildPlan,
    analysis: &mut UnitAnalysis,
) {
    let Form::List(values) = without_metadata(form) else {
        scan_dynamic_children(runtime, module, form, plan, analysis);
        return;
    };
    let Some(Form::Symbol(operator)) = values.first() else {
        scan_dynamic_children(runtime, module, form, plan, analysis);
        return;
    };
    let operation = operator.rsplit('/').next().unwrap_or(operator);
    match operation {
        "resolve" | "var" => {
            if let Some(name) = values.get(1).and_then(literal_symbol) {
                analysis
                    .runtime_edges
                    .insert(canonical_symbol(runtime, module, name));
            } else if plan.keep_vars.is_empty() && plan.keep_namespaces.is_empty() {
                push_dynamic_diagnostic(analysis, "unbounded-dynamic-var", operation);
            }
        }
        "require" => {
            if let Some(namespace) = values.get(1).and_then(literal_symbol) {
                analysis.namespace_edges.insert(namespace.to_owned());
            } else if plan.keep_namespaces.is_empty() {
                push_dynamic_diagnostic(analysis, "unbounded-dynamic-namespace", operation);
            }
        }
        "load-string" => {
            if let Some(Form::String(source)) = values.get(1).map(without_metadata) {
                match crate::kernel::parse_forms(source) {
                    Ok(forms) => {
                        for loaded in forms {
                            collect_code_symbols(runtime, module, &loaded, analysis);
                        }
                    }
                    Err(error) => analysis.diagnostics.push(Diagnostic {
                        code: "production/invalid-constant-load-string".into(),
                        operation: operation.into(),
                        module: module.into(),
                        location: analysis.location.clone(),
                        message: error,
                    }),
                }
            } else if plan.keep_vars.is_empty() && plan.keep_namespaces.is_empty() {
                push_dynamic_diagnostic(analysis, "unbounded-generated-source", operation);
            }
        }
        "eval" => {
            if let Some(code) = values.get(1).and_then(quoted_value) {
                collect_code_symbols(runtime, module, code, analysis);
            } else if plan.keep_vars.is_empty() && plan.keep_namespaces.is_empty() {
                push_dynamic_diagnostic(analysis, "unbounded-eval", operation);
            }
        }
        "eval-in-ns" => {
            let target = values.get(1).and_then(literal_symbol);
            let code = values.get(2).and_then(quoted_value);
            match (target, code) {
                (Some(target), Some(code)) => {
                    analysis.namespace_edges.insert(target.to_owned());
                    collect_code_symbols(runtime, target, code, analysis);
                }
                _ if plan.keep_vars.is_empty() && plan.keep_namespaces.is_empty() => {
                    push_dynamic_diagnostic(analysis, "unbounded-eval-in-ns", operation);
                }
                _ => {}
            }
        }
        _ => {}
    }
    if operation != "quote" && operation != "syntax-quote" {
        for value in values.iter().skip(1) {
            scan_dynamic_access(runtime, module, value, plan, analysis);
        }
    }
}

fn scan_dynamic_children(
    runtime: &Runtime,
    module: &str,
    form: &Form,
    plan: &BuildPlan,
    analysis: &mut UnitAnalysis,
) {
    match form {
        Form::List(values) | Form::Vector(values) | Form::Set(values) => {
            for value in values {
                scan_dynamic_access(runtime, module, value, plan, analysis);
            }
        }
        Form::Map(entries) => {
            for (key, value) in entries {
                scan_dynamic_access(runtime, module, key, plan, analysis);
                scan_dynamic_access(runtime, module, value, plan, analysis);
            }
        }
        Form::Metadata(_, value) | Form::Tagged(_, value) => {
            scan_dynamic_access(runtime, module, value, plan, analysis);
        }
        _ => {}
    }
}

fn collect_code_symbols(
    runtime: &Runtime,
    module: &str,
    form: &Form,
    analysis: &mut UnitAnalysis,
) {
    match form {
        Form::Symbol(name) => {
            if let Some(resolved) = resolve_existing_symbol(runtime, module, name) {
                analysis.runtime_edges.insert(resolved);
            }
        }
        Form::List(values) => {
            for value in values {
                collect_code_symbols(runtime, module, value, analysis);
            }
        }
        Form::Vector(values) | Form::Set(values) => {
            for value in values {
                collect_code_symbols(runtime, module, value, analysis);
            }
        }
        Form::Map(entries) => {
            for (key, value) in entries {
                collect_code_symbols(runtime, module, key, analysis);
                collect_code_symbols(runtime, module, value, analysis);
            }
        }
        Form::Metadata(_, value) | Form::Tagged(_, value) => {
            collect_code_symbols(runtime, module, value, analysis);
        }
        _ => {}
    }
}

fn push_dynamic_diagnostic(analysis: &mut UnitAnalysis, code: &str, operation: &str) {
    analysis.diagnostics.push(Diagnostic {
        code: format!("production/{code}"),
        operation: operation.into(),
        module: analysis.module.clone(),
        location: analysis.location.clone(),
        message: format!(
            "reachable non-literal {operation} is not bounded by :build/keep-vars or :build/keep-namespaces"
        ),
    });
}

pub(super) fn canonical_symbol(runtime: &Runtime, module: &str, name: &str) -> String {
    resolve_existing_symbol(runtime, module, name).unwrap_or_else(|| {
        if name.contains('/') {
            name.into()
        } else {
            qualify(module, name)
        }
    })
}

fn resolve_existing_symbol(runtime: &Runtime, module: &str, name: &str) -> Option<String> {
    let namespace = runtime.namespace_registry.find(module)?;
    let symbol = Symbol::parse(name);
    let resolved = if name.contains('/') {
        runtime
            .namespace_registry
            .resolve(&symbol)
            .or_else(|| namespace.resolve(&symbol))
    } else {
        namespace.resolve(&symbol)
    }?;
    Some(resolved.symbol().as_str().to_owned())
}

fn literal_symbol(form: &Form) -> Option<&str> {
    match without_metadata(form) {
        Form::Symbol(value) => Some(value),
        Form::List(values)
            if values.len() == 2
                && matches!(values.first(), Some(Form::Symbol(head)) if head == "quote") =>
        {
            match without_metadata(&values[1]) {
                Form::Symbol(value) => Some(value),
                _ => None,
            }
        }
        _ => None,
    }
}

fn quoted_value(form: &Form) -> Option<&Form> {
    match without_metadata(form) {
        Form::List(values)
            if values.len() == 2
                && matches!(values.first(), Some(Form::Symbol(head)) if head == "quote") =>
        {
            values.get(1)
        }
        _ => None,
    }
}
