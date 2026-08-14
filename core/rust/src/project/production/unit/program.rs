use super::{qualify, Effect, UnitAnalysis, UnitKind};
use crate::core::Value;
use crate::vm::{Instruction, Program};

pub(super) fn scan_program(program: &Program, analysis: &mut UnitAnalysis) {
    for prototype in &program.functions {
        for instruction in &prototype.code {
            match instruction {
                Instruction::GetGlobal(index)
                | Instruction::VarGlobal(index)
                | Instruction::SetGlobal(index)
                | Instruction::DeclareGlobal(index)
                | Instruction::DynamicBind(index)
                | Instruction::DynamicUnbind(index) => {
                    if let Some(name) = string_constant(program, *index) {
                        analysis.runtime_edges.insert(name.to_owned());
                        classify_native_edge(name, analysis);
                    }
                }
                Instruction::Primitive { op, .. }
                | Instruction::PrimitiveLocalConst { op, .. }
                | Instruction::PrimitiveValue(op) => {
                    analysis.native_primitives.insert(op.operator().to_owned());
                }
                Instruction::BuiltinValue(index) => {
                    let name = string_constant(program, *index)
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("builtin:{index}"));
                    analysis.native_primitives.insert(name);
                }
                Instruction::HostCall => {
                    analysis
                        .native_primitives
                        .insert("std.native.Host/call".into());
                }
                Instruction::DotCall { method, .. } => {
                    if let Some(name) = string_constant(program, *method) {
                        analysis.native_primitives.insert(format!("dot:{name}"));
                    }
                }
                Instruction::DefStruct { name, .. } | Instruction::DefMutable { name, .. } => {
                    if let Some(name) = string_constant(program, *name) {
                        analysis
                            .native_types
                            .insert(qualify(&analysis.module, name));
                    }
                }
                Instruction::DefProtocol(index) | Instruction::ExtendType(index) => {
                    analysis
                        .native_protocols
                        .insert(format!("declaration:{index}"));
                }
                Instruction::DefMulti(index) | Instruction::DefMethod(index) => {
                    analysis
                        .native_protocols
                        .insert(format!("multimethod:{index}"));
                }
                _ => {}
            }
        }
    }
}

pub(super) fn classify_effect(program: &Program, kind: UnitKind) -> Effect {
    if kind == UnitKind::Registration {
        return Effect::Unknown;
    }
    let Some(entry) = program.functions.first() else {
        return Effect::Unknown;
    };
    let mut unknown = false;
    for instruction in &entry.code {
        match instruction {
            Instruction::SetGlobal(_)
            | Instruction::MutableFieldSet(_)
            | Instruction::DynamicBind(_)
            | Instruction::DynamicUnbind(_)
            | Instruction::HostCall
            | Instruction::DotCall { .. }
            | Instruction::ExtendType(_)
            | Instruction::DefMethod(_) => return Effect::Effectful,
            Instruction::Call { .. }
            | Instruction::CallStatic { .. }
            | Instruction::Await
            | Instruction::Yield => unknown = true,
            _ => {}
        }
    }
    if unknown {
        Effect::Unknown
    } else {
        Effect::Pure
    }
}

fn string_constant(program: &Program, index: u32) -> Option<&str> {
    match program.constants.get(index as usize) {
        Some(Value::String(value)) => Some(value),
        _ => None,
    }
}

fn classify_native_edge(name: &str, analysis: &mut UnitAnalysis) {
    if let Some((namespace, _)) = name.split_once('/') {
        if namespace.starts_with("std.native.") {
            analysis.native_types.insert(namespace.to_owned());
        }
        if namespace.starts_with("std.protocol.") {
            analysis.native_protocols.insert(namespace.to_owned());
        }
    }
}
