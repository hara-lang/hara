use super::plan::BuildPlan;
use super::source::{Diagnostic, SourceLocation};
use super::unit::{Effect, UnitAnalysis, UnitKind};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleAnalysis {
    pub name: String,
    pub path: String,
    pub digest: String,
    pub input_bytes: usize,
    pub dependencies: Vec<String>,
    pub unit_ids: Vec<String>,
    pub standard_library: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionReason {
    pub unit_id: String,
    pub subject: Option<String>,
    pub code: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis {
    pub modules: Vec<ModuleAnalysis>,
    pub units: Vec<UnitAnalysis>,
    pub runtime_roots: BTreeSet<String>,
    pub runtime_closure: BTreeSet<String>,
    pub compile_time_roots: BTreeSet<String>,
    pub compile_time_closure: BTreeSet<String>,
    pub retained_unit_ids: BTreeSet<String>,
    pub removed_unit_ids: BTreeSet<String>,
    pub retained_vars: BTreeSet<String>,
    pub removed_vars: BTreeSet<String>,
    pub retained_namespaces: BTreeSet<String>,
    pub removed_namespaces: BTreeSet<String>,
    pub reasons: Vec<RetentionReason>,
    pub diagnostics: Vec<Diagnostic>,
    pub native_primitives: BTreeSet<String>,
    pub native_types: BTreeSet<String>,
    pub native_protocols: BTreeSet<String>,
    pub input_bytes: usize,
    pub input_digest: String,
}

#[derive(Debug, Clone)]
pub struct AnalysisOutput {
    pub analysis: Analysis,
    pub report_path: std::path::PathBuf,
    pub report_source: String,
}

pub fn finish_analysis(
    plan: &BuildPlan,
    modules: Vec<ModuleAnalysis>,
    units: Vec<UnitAnalysis>,
    input_bytes: usize,
    input_digest: String,
) -> Analysis {
    let providers = provider_index(&units);
    let namespace_units = namespace_index(&units);
    let unit_positions = units
        .iter()
        .enumerate()
        .map(|(index, unit)| (unit.id.clone(), index))
        .collect::<BTreeMap<_, _>>();

    let mut runtime_roots = plan
        .entrypoints
        .iter()
        .chain(plan.keep_vars.iter())
        .cloned()
        .collect::<BTreeSet<_>>();
    for namespace in &plan.keep_namespaces {
        if let Some(indices) = namespace_units.get(namespace) {
            for index in indices {
                runtime_roots.extend(units[*index].provides.iter().cloned());
            }
        }
    }

    let mut reasons = Vec::new();
    let mut runtime_unit_ids = BTreeSet::new();
    let mut runtime_closure = runtime_roots.clone();
    let mut queue = VecDeque::new();
    for root in &runtime_roots {
        if let Some(unit_id) = providers.get(root) {
            retain(
                unit_id,
                Some(root.clone()),
                if plan.entrypoints.contains(root) {
                    "entrypoint"
                } else {
                    "keep-var"
                },
                "declared production root",
                &mut runtime_unit_ids,
                &mut reasons,
                &mut queue,
            );
        }
    }
    for namespace in &plan.keep_namespaces {
        if let Some(indices) = namespace_units.get(namespace) {
            for index in indices {
                retain(
                    &units[*index].id,
                    None,
                    "keep-namespace",
                    &format!("namespace {namespace} is explicitly retained"),
                    &mut runtime_unit_ids,
                    &mut reasons,
                    &mut queue,
                );
            }
        }
    }
    for unit in &units {
        if unit.kind == UnitKind::Registration {
            retain(
                &unit.id,
                unit.provides.iter().next().cloned(),
                "registration",
                "registration forms are conservatively retained",
                &mut runtime_unit_ids,
                &mut reasons,
                &mut queue,
            );
        } else if unit.kind == UnitKind::Initializer && unit.effect != Effect::Pure {
            retain(
                &unit.id,
                None,
                "unknown-top-level-effect",
                "top-level initializer is not proven pure",
                &mut runtime_unit_ids,
                &mut reasons,
                &mut queue,
            );
        }
    }

    while let Some(unit_id) = queue.pop_front() {
        let Some(index) = unit_positions.get(&unit_id).copied() else {
            continue;
        };
        let unit = &units[index];
        for edge in &unit.runtime_edges {
            runtime_closure.insert(edge.clone());
            if let Some(target) = providers.get(edge) {
                retain(
                    target,
                    Some(edge.clone()),
                    "runtime-dependency",
                    &format!("referenced by {}", unit.id),
                    &mut runtime_unit_ids,
                    &mut reasons,
                    &mut queue,
                );
            }
        }
        for namespace in &unit.namespace_edges {
            if let Some(indices) = namespace_units.get(namespace) {
                for target in indices {
                    let target = &units[*target];
                    retain(
                        &target.id,
                        None,
                        "dynamic-namespace",
                        &format!("namespace {namespace} is loaded by {}", unit.id),
                        &mut runtime_unit_ids,
                        &mut reasons,
                        &mut queue,
                    );
                }
            }
        }
    }

    let mut compile_time_roots = BTreeSet::new();
    for unit_id in &runtime_unit_ids {
        if let Some(index) = unit_positions.get(unit_id) {
            compile_time_roots.extend(units[*index].compile_time_edges.iter().cloned());
        }
    }
    let mut compile_time_closure = compile_time_roots.clone();
    let mut compile_unit_ids = BTreeSet::new();
    let mut compile_queue = VecDeque::new();
    for root in &compile_time_roots {
        if let Some(unit_id) = providers.get(root) {
            retain(
                unit_id,
                Some(root.clone()),
                "compile-time-macro",
                "macro is required while producing retained definitions",
                &mut compile_unit_ids,
                &mut reasons,
                &mut compile_queue,
            );
        }
    }
    while let Some(unit_id) = compile_queue.pop_front() {
        let Some(index) = unit_positions.get(&unit_id).copied() else {
            continue;
        };
        let unit = &units[index];
        for edge in unit
            .compile_time_edges
            .iter()
            .chain(unit.runtime_edges.iter())
        {
            compile_time_closure.insert(edge.clone());
            if let Some(target) = providers.get(edge) {
                retain(
                    target,
                    Some(edge.clone()),
                    "compile-time-dependency",
                    &format!("required by compile-time unit {}", unit.id),
                    &mut compile_unit_ids,
                    &mut reasons,
                    &mut compile_queue,
                );
            }
        }
    }

    let retained_unit_ids = runtime_unit_ids
        .union(&compile_unit_ids)
        .cloned()
        .collect::<BTreeSet<_>>();
    let all_unit_ids = units.iter().map(|unit| unit.id.clone()).collect::<BTreeSet<_>>();
    let removed_unit_ids = all_unit_ids
        .difference(&retained_unit_ids)
        .cloned()
        .collect::<BTreeSet<_>>();

    let mut retained_vars = BTreeSet::new();
    let mut removed_vars = BTreeSet::new();
    let mut retained_namespaces = BTreeSet::new();
    let mut native_primitives = BTreeSet::new();
    let mut native_types = BTreeSet::new();
    let mut native_protocols = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for unit in &units {
        if retained_unit_ids.contains(&unit.id) {
            retained_vars.extend(unit.provides.iter().cloned());
            retained_namespaces.insert(unit.module.clone());
            native_primitives.extend(unit.native_primitives.iter().cloned());
            native_types.extend(unit.native_types.iter().cloned());
            native_protocols.extend(unit.native_protocols.iter().cloned());
            diagnostics.extend(unit.diagnostics.iter().cloned());
        } else {
            removed_vars.extend(unit.provides.iter().cloned());
        }
    }
    removed_vars = removed_vars
        .difference(&retained_vars)
        .cloned()
        .collect::<BTreeSet<_>>();
    let all_namespaces = modules
        .iter()
        .map(|module| module.name.clone())
        .collect::<BTreeSet<_>>();
    let removed_namespaces = all_namespaces
        .difference(&retained_namespaces)
        .cloned()
        .collect::<BTreeSet<_>>();

    for entrypoint in &plan.entrypoints {
        if !providers.contains_key(entrypoint) {
            diagnostics.push(Diagnostic {
                code: "production/missing-entrypoint".into(),
                operation: "entrypoint".into(),
                module: entrypoint
                    .split_once('/')
                    .map(|(namespace, _)| namespace)
                    .unwrap_or("unknown")
                    .into(),
                location: SourceLocation {
                    path: "project.edn".into(),
                    line: 1,
                    column: 1,
                    end_line: 1,
                    end_column: 1,
                },
                message: format!("production entrypoint has no analyzed provider: {entrypoint}"),
            });
        }
    }
    diagnostics.sort_by(|left, right| {
        (
            left.location.path.as_str(),
            left.location.line,
            left.location.column,
            left.code.as_str(),
        )
            .cmp(&(
                right.location.path.as_str(),
                right.location.line,
                right.location.column,
                right.code.as_str(),
            ))
    });
    diagnostics.dedup();
    reasons.sort_by(|left, right| {
        (
            left.unit_id.as_str(),
            left.code.as_str(),
            left.subject.as_deref().unwrap_or(""),
        )
            .cmp(&(
                right.unit_id.as_str(),
                right.code.as_str(),
                right.subject.as_deref().unwrap_or(""),
            ))
    });
    reasons.dedup();

    Analysis {
        modules,
        units,
        runtime_roots,
        runtime_closure,
        compile_time_roots,
        compile_time_closure,
        retained_unit_ids,
        removed_unit_ids,
        retained_vars,
        removed_vars,
        retained_namespaces,
        removed_namespaces,
        reasons,
        diagnostics,
        native_primitives,
        native_types,
        native_protocols,
        input_bytes,
        input_digest,
    }
}

fn provider_index(units: &[UnitAnalysis]) -> BTreeMap<String, String> {
    let mut output = BTreeMap::new();
    for unit in units {
        for provided in &unit.provides {
            output.entry(provided.clone()).or_insert_with(|| unit.id.clone());
        }
    }
    output
}

fn namespace_index(units: &[UnitAnalysis]) -> BTreeMap<String, Vec<usize>> {
    let mut output = BTreeMap::<String, Vec<usize>>::new();
    for (index, unit) in units.iter().enumerate() {
        output.entry(unit.module.clone()).or_default().push(index);
    }
    output
}

#[allow(clippy::too_many_arguments)]
fn retain(
    unit_id: &str,
    subject: Option<String>,
    code: &str,
    detail: &str,
    retained: &mut BTreeSet<String>,
    reasons: &mut Vec<RetentionReason>,
    queue: &mut VecDeque<String>,
) {
    reasons.push(RetentionReason {
        unit_id: unit_id.into(),
        subject,
        code: code.into(),
        detail: detail.into(),
    });
    if retained.insert(unit_id.into()) {
        queue.push_back(unit_id.into());
    }
}
