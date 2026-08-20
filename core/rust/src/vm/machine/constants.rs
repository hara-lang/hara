//! Decoding of typed VM constant-pool operands.

use super::Program;
use crate::core::Value;

/// Reads a string constant (the global-name operands).
pub(super) fn constant_string(program: &Program, index: u32) -> Option<&str> {
    match program.constants.get(index as usize) {
        Some(Value::String(value)) => Some(value.as_str()),
        _ => None,
    }
}

/// Reads a string-vector constant (defstruct field names).
pub(super) fn constant_string_vector(program: &Program, index: u32) -> Option<Vec<String>> {
    match program.constants.get(index as usize) {
        Some(Value::Vector(fields)) => fields
            .iter()
            .map(|field| match field {
                Value::String(field) => Some(field.clone()),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}
