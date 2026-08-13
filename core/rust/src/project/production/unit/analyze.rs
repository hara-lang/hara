use super::super::graph::{Effect, UnitAnalysis, UnitKind};
use super::dynamic::{canonical_symbol, collect_resolved_symbols, scan_dynamic_access};
use super::program::{classify_effect, scan_program};
use super::provides::{list_head, provided_vars, unit_kind, without_metadata};
use super::super::plan::BuildPlan;
use super::super::source::{Diagnostic, SourceLocation};
use crate::core::{self, Value};
use crate::kernel::Form;
use crate::vm::Program;
use crate::Runtime;
use std::collections::BTreeSet;
use std::rc::Rc;

pub struct UnitSeed {
    pub id: String,
    pub module: String,
    pub index: usize,
    pub form: Form,
    pub compile_time_edges: BTreeSet<String>,
    pub location: SourceLocation,
}

pub struct CompiledUnit {
    pub analysis: UnitAnalysis,
    pub program: Option<Rc<Program>>,
}

pub fn expand_top_level(
    runtime: &Runtime,
    module: &str,
    index: usize,
    form: &Form,
    location: SourceLocation,
) -> Result<Vec<UnitSeed>, String> {
    let mut compile_time_edges = BTreeSet::new();
    let expanded = expand_form(runtime, module, form, &mut compile_time_edges)?;
    let mut forms = Vec::new();
    flatten_top_level(expanded, &mut forms);
    Ok(forms
        .into_iter()
        .enumerate()
        .map(|(subindex, form)| UnitSeed {
            id: format!("{module}:{index:05}:{subindex:03}"),
            module: module.into(),
            index: index * 1000 + subindex,
            form,
            compile_time_edges: compile_time_edges.clone(),
            location: location.clone(),
        })
        .collect())
}

pub fn analyze_unit(runtime: &Runtime, seed: UnitSeed, plan: &BuildPlan) -> CompiledUnit {
    let kind = unit_kind(&seed.form);
    let provides = provided_vars(&seed.form, &seed.module);
    let mut analysis = UnitAnalysis {
        id: seed.id,
        module: seed.module.clone(),
        index: seed.index,
        kind,
        effect: Effect::Unknown,
        location: seed.location.clone(),
        provides,
        runtime_edges: BTreeSet::new(),
        compile_time_edges: seed.compile_time_edges,
        namespace_edges: BTreeSet::new(),
        native_primitives: BTreeSet::new(),
        native_types: BTreeSet::new(),
        native_protocols: BTreeSet::new(),
        diagnostics: Vec::new(),
    };
    scan_dynamic_access(runtime, &seed.module, &seed.form, plan, &mut analysis);
    if kind == UnitKind::Registration {
        collect_resolved_symbols(
            runtime,
            &seed.module,
            &seed.form,
            &mut analysis.runtime_edges,
        );
    }
    let program = match runtime.compile_bytecode(&seed.form.to_string()) {
        Ok(program) => {
            scan_program(&program, &mut analysis);
            analysis.effect = classify_effect(&program, kind);
            Some(program)
        }
        Err(error) => {
            analysis.effect = Effect::Unknown;
            analysis.diagnostics.push(Diagnostic {
                code: "production/unit-compile-failed".into(),
                operation: "compile".into(),
                module: seed.module,
                location: seed.location,
                message: error,
            });
            None
        }
    };
    CompiledUnit { analysis, program }
}

pub fn execute_compile_time_unit(
    runtime: &mut Runtime,
    compiled: &CompiledUnit,
) -> Result<(), String> {
    let eligible = match compiled.analysis.kind {
        UnitKind::Macro | UnitKind::Registration => {
            compiled.analysis.effect != Effect::Effectful
        }
        UnitKind::Definition => compiled.analysis.effect == Effect::Pure,
        UnitKind::Initializer => false,
    };
    if !eligible {
        return Ok(());
    }
    let Some(program) = &compiled.program else {
        return Ok(());
    };
    runtime
        .execute_compiled_bytecode_registry_value(program.clone())
        .map(|_| ())
}


fn expand_form(
    runtime: &Runtime,
    module: &str,
    form: &Form,
    compile_time_edges: &mut BTreeSet<String>,
) -> Result<Form, String> {
    let stripped = without_metadata(form);
    if matches!(
        stripped,
        Form::List(values)
            if matches!(values.first(), Some(Form::Symbol(head)) if head == "quote" || head == "syntax-quote")
    ) {
        return Ok(form.clone());
    }
    if matches!(stripped, Form::List(_)) {
        let expanded = macroexpand(runtime, form)?;
        if expanded != *form {
            if let Some(head) = list_head(stripped) {
                compile_time_edges.insert(canonical_symbol(runtime, module, head));
            }
            return expand_form(runtime, module, &expanded, compile_time_edges);
        }
    }
    Ok(match form {
        Form::Metadata(metadata, value) => Form::Metadata(
            metadata.clone(),
            Box::new(expand_form(runtime, module, value, compile_time_edges)?),
        ),
        Form::Tagged(tag, value) => Form::Tagged(
            tag.clone(),
            Box::new(expand_form(runtime, module, value, compile_time_edges)?),
        ),
        Form::List(values) => Form::List(
            values
                .iter()
                .map(|value| expand_form(runtime, module, value, compile_time_edges))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Form::Vector(values) => Form::Vector(
            values
                .iter()
                .map(|value| expand_form(runtime, module, value, compile_time_edges))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Form::Set(values) => Form::Set(
            values
                .iter()
                .map(|value| expand_form(runtime, module, value, compile_time_edges))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Form::Map(entries) => Form::Map(
            entries
                .iter()
                .map(|(key, value)| {
                    Ok((
                        expand_form(runtime, module, key, compile_time_edges)?,
                        expand_form(runtime, module, value, compile_time_edges)?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?,
        ),
        value => value.clone(),
    })
}

fn macroexpand(runtime: &Runtime, form: &Form) -> Result<Form, String> {
    core::with_macros(runtime.macros.clone(), || {
        core::with_namespace_registry(&runtime.namespace_registry, || {
            core::with_protocols(&runtime.protocols, || core::vm_macroexpand(form))
        })
    })
}

fn flatten_top_level(form: Form, output: &mut Vec<Form>) {
    let stripped = without_metadata(&form);
    if let Form::List(values) = stripped {
        if matches!(values.first(), Some(Form::Symbol(head)) if head == "do") {
            for value in values.iter().skip(1) {
                flatten_top_level(value.clone(), output);
            }
            return;
        }
    }
    output.push(form);
}

