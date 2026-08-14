//! Canonical semantic representation of the portable HAL schema grammar.
//!
//! HALC keeps surface schemas as ordinary forms on the wire. This module is
//! the compiler-facing lowering step: it turns those forms into a strict,
//! portable graph without evaluating schema Vars or copying nested definitions.

use super::Form;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaField {
    pub name: Form,
    pub value_type: SchemaType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSchema {
    pub fixed: Vec<SchemaType>,
    pub rest: Option<Box<SchemaType>>,
    pub output: Box<SchemaType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaType {
    Primitive(String),
    Reference(String),
    Union(Vec<SchemaType>),
    Vector(Box<SchemaType>),
    Tuple(Vec<SchemaType>),
    Map(Vec<SchemaField>),
    Function(Vec<FunctionSchema>),
    Enum(Vec<Form>),
    // Retained for artifact compatibility. The portable normalizer no longer
    // produces extensions for unsupported surface heads.
    Extension { head: String, arguments: Vec<Form> },
    Unknown(Form),
}

fn schema_error(code: &str, detail: impl AsRef<str>) -> String {
    format!("{code}: {}", detail.as_ref())
}

fn canonical_primitive(name: &str) -> Option<&'static str> {
    match name {
        "boolean" => Some("bool"),
        "number" => Some("num"),
        "integer" => Some("int"),
        "string" => Some("str"),
        "any" => Some("any"),
        "nil" => Some("nil"),
        "bool" => Some("bool"),
        "num" => Some("num"),
        "int" => Some("int"),
        "float" => Some("float"),
        "decimal" => Some("decimal"),
        "str" => Some("str"),
        "char" => Some("char"),
        "regex" => Some("regex"),
        "keyword" => Some("keyword"),
        "symbol" => Some("symbol"),
        "list" => Some("list"),
        "vector" => Some("vector"),
        "map" => Some("map"),
        "set" => Some("set"),
        "fn" => Some("fn"),
        "atom" => Some("atom"),
        "bytes" => Some("bytes"),
        "promise" => Some("promise"),
        _ => None,
    }
}

pub fn normalize_schema(schema: &Form) -> Result<SchemaType, String> {
    match schema {
        Form::Keyword(name) => canonical_primitive(name)
            .map(|name| SchemaType::Primitive(name.into()))
            .ok_or_else(|| {
                schema_error(
                    "unsupported-primitive",
                    format!("unsupported schema primitive: :{name}"),
                )
            }),
        Form::List(reference)
            if matches!(reference.first(), Some(Form::Symbol(operator)) if operator == "var") =>
        {
            normalize_reference(reference)
        }
        Form::Vector(items) if items.is_empty() => Err(schema_error(
            "empty-schema",
            "schema vector cannot be empty",
        )),
        Form::Vector(items) => normalize_composite(items),
        other => Err(schema_error(
            "unsupported-value",
            format!("unsupported schema value: {other}"),
        )),
    }
}

fn normalize_reference(reference: &[Form]) -> Result<SchemaType, String> {
    if reference.len() != 2 {
        return Err(schema_error(
            "invalid-reference",
            "named schema reference must be (var qualified/Symbol)",
        ));
    }
    match &reference[1] {
        Form::Symbol(name)
            if name
                .split_once('/')
                .is_some_and(|(namespace, local)| !namespace.is_empty() && !local.is_empty()) =>
        {
            Ok(SchemaType::Reference(name.clone()))
        }
        Form::Symbol(name) => Err(schema_error(
            "unqualified-reference",
            format!("named schema reference is not fully qualified: {name}"),
        )),
        _ => Err(schema_error(
            "invalid-reference",
            "named schema reference must target a symbol",
        )),
    }
}

fn normalize_composite(items: &[Form]) -> Result<SchemaType, String> {
    let Form::Keyword(head) = &items[0] else {
        return Err(schema_error(
            "invalid-head",
            "schema vector head must be a keyword",
        ));
    };
    let arguments = &items[1..];
    match head.as_str() {
        "or" => {
            if arguments.is_empty() {
                return Err(schema_error(
                    "empty-union",
                    ":or schema requires at least one member",
                ));
            }
            normalize_union(
                arguments
                    .iter()
                    .map(normalize_schema)
                    .collect::<Result<Vec<_>, _>>()?,
            )
        }
        "maybe" => {
            require_count(head, arguments, 1)?;
            normalize_union(vec![
                normalize_schema(&arguments[0])?,
                SchemaType::Primitive("nil".into()),
            ])
        }
        "vector" => {
            require_count(head, arguments, 1)?;
            Ok(SchemaType::Vector(Box::new(normalize_schema(
                &arguments[0],
            )?)))
        }
        "tuple" => arguments
            .iter()
            .map(normalize_schema)
            .collect::<Result<Vec<_>, _>>()
            .map(SchemaType::Tuple),
        "map" => normalize_map(arguments),
        "fn" => normalize_function(items).map(|arity| SchemaType::Function(vec![arity])),
        "function" => {
            if arguments.is_empty() {
                return Err(schema_error(
                    "empty-function",
                    ":function schema requires at least one :fn schema",
                ));
            }
            arguments
                .iter()
                .map(|argument| {
                    let Form::Vector(function) = argument else {
                        return Err(schema_error(
                            "invalid-function-member",
                            ":function members must be :fn schemas",
                        ));
                    };
                    if !matches!(function.first(), Some(Form::Keyword(head)) if head == "fn") {
                        return Err(schema_error(
                            "invalid-function-member",
                            ":function members must be :fn schemas",
                        ));
                    }
                    normalize_function(function)
                })
                .collect::<Result<Vec<_>, _>>()
                .map(SchemaType::Function)
        }
        "enum" => {
            if arguments.is_empty() {
                return Err(schema_error(
                    "empty-enum",
                    ":enum schema requires at least one value",
                ));
            }
            let mut values = Vec::new();
            for value in arguments {
                if !values.contains(value) {
                    values.push(value.clone());
                }
            }
            Ok(SchemaType::Enum(values))
        }
        _ => Err(schema_error(
            "unsupported-form",
            format!("unsupported schema form: :{head}"),
        )),
    }
}

fn normalize_union(values: Vec<SchemaType>) -> Result<SchemaType, String> {
    let mut members = Vec::new();
    for value in values {
        match value {
            SchemaType::Union(nested) => {
                for member in nested {
                    push_unique(&mut members, member);
                }
            }
            member => push_unique(&mut members, member),
        }
    }
    match members.len() {
        0 => Err(schema_error(
            "empty-union",
            ":or schema requires at least one member",
        )),
        1 => Ok(members.pop().expect("singleton union has one member")),
        _ => Ok(SchemaType::Union(members)),
    }
}

fn normalize_map(arguments: &[Form]) -> Result<SchemaType, String> {
    let mut fields = Vec::with_capacity(arguments.len());
    let mut names = Vec::new();
    for argument in arguments {
        let Form::Vector(pair) = argument else {
            return Err(schema_error(
                "invalid-map-field",
                ":map schema fields must be [name type] pairs",
            ));
        };
        if pair.len() != 2 {
            return Err(schema_error(
                "invalid-map-field",
                ":map schema fields must be [name type] pairs",
            ));
        }
        if names.contains(&pair[0]) {
            return Err(schema_error(
                "duplicate-map-field",
                format!("duplicate :map schema field: {}", pair[0]),
            ));
        }
        names.push(pair[0].clone());
        fields.push(SchemaField {
            name: pair[0].clone(),
            value_type: normalize_schema(&pair[1])?,
        });
    }
    Ok(SchemaType::Map(fields))
}

fn normalize_function(items: &[Form]) -> Result<FunctionSchema, String> {
    if !matches!(items.first(), Some(Form::Keyword(head)) if head == "fn") || items.len() != 3 {
        return Err(schema_error(
            "invalid-arity",
            ":fn schema must be [:fn [inputs ...] output]",
        ));
    }
    let Form::Vector(inputs) = &items[1] else {
        return Err(schema_error(
            "invalid-function-inputs",
            ":fn schema inputs must be a vector",
        ));
    };
    let mut fixed = Vec::new();
    let mut rest = None;
    let mut index = 0;
    while index < inputs.len() {
        if matches!(&inputs[index], Form::Symbol(marker) if marker == "&") {
            if rest.is_some() || index + 2 != inputs.len() {
                return Err(schema_error(
                    "invalid-function-rest",
                    ":fn schema & must precede exactly one rest type",
                ));
            }
            rest = Some(Box::new(normalize_schema(&inputs[index + 1])?));
            index += 2;
        } else {
            fixed.push(normalize_schema(&inputs[index])?);
            index += 1;
        }
    }
    Ok(FunctionSchema {
        fixed,
        rest,
        output: Box::new(normalize_schema(&items[2])?),
    })
}

fn require_count(head: &str, arguments: &[Form], expected: usize) -> Result<(), String> {
    if arguments.len() == expected {
        Ok(())
    } else {
        Err(schema_error(
            "invalid-arity",
            format!(
                ":{head} schema expects {expected} argument{}, got {}",
                if expected == 1 { "" } else { "s" },
                arguments.len()
            ),
        ))
    }
}

fn push_unique(output: &mut Vec<SchemaType>, value: SchemaType) {
    if !output.contains(&value) {
        output.push(value);
    }
}

/// Resolves all reachable named schema references without evaluating code.
/// Recursive edges remain references, preserving a finite normalized graph.
pub fn resolve_schema(
    schema: &SchemaType,
    definitions: &HashMap<String, SchemaType>,
) -> SchemaType {
    resolve_schema_inner(schema, definitions, &HashSet::new())
}

fn resolve_schema_inner(
    schema: &SchemaType,
    definitions: &HashMap<String, SchemaType>,
    visited: &HashSet<String>,
) -> SchemaType {
    match schema {
        SchemaType::Reference(name) => {
            if visited.contains(name) {
                return SchemaType::Reference(name.clone());
            }
            let Some(target) = definitions.get(name) else {
                return SchemaType::Reference(name.clone());
            };
            let mut nested = visited.clone();
            nested.insert(name.clone());
            resolve_schema_inner(target, definitions, &nested)
        }
        SchemaType::Union(types) => {
            let resolved = types
                .iter()
                .map(|value| resolve_schema_inner(value, definitions, visited))
                .collect::<Vec<_>>();
            normalize_union(resolved).unwrap_or_else(|_| schema.clone())
        }
        SchemaType::Vector(item) => SchemaType::Vector(Box::new(resolve_schema_inner(
            item,
            definitions,
            visited,
        ))),
        SchemaType::Tuple(items) => SchemaType::Tuple(
            items
                .iter()
                .map(|value| resolve_schema_inner(value, definitions, visited))
                .collect(),
        ),
        SchemaType::Map(fields) => SchemaType::Map(
            fields
                .iter()
                .map(|field| SchemaField {
                    name: field.name.clone(),
                    value_type: resolve_schema_inner(&field.value_type, definitions, visited),
                })
                .collect(),
        ),
        SchemaType::Function(arities) => SchemaType::Function(
            arities
                .iter()
                .map(|arity| FunctionSchema {
                    fixed: arity
                        .fixed
                        .iter()
                        .map(|value| resolve_schema_inner(value, definitions, visited))
                        .collect(),
                    rest: arity.rest.as_ref().map(|value| {
                        Box::new(resolve_schema_inner(value, definitions, visited))
                    }),
                    output: Box::new(resolve_schema_inner(
                        &arity.output,
                        definitions,
                        visited,
                    )),
                })
                .collect(),
        ),
        _ => schema.clone(),
    }
}

fn primitive_compatible(expected: &str, actual: &str) -> bool {
    expected == actual
        || expected == "any"
        || actual == "any"
        || (expected == "num" && matches!(actual, "int" | "float" | "decimal"))
        || (actual == "num" && matches!(expected, "int" | "float" | "decimal"))
}

/// Returns true when two normalized schemas have an overlapping value domain.
pub fn compatible_schema(expected: &SchemaType, actual: &SchemaType) -> bool {
    if expected == actual {
        return true;
    }
    match (expected, actual) {
        (SchemaType::Unknown(_), _) | (_, SchemaType::Unknown(_)) => true,
        (SchemaType::Reference(_), _) | (_, SchemaType::Reference(_)) => true,
        (SchemaType::Primitive(expected), SchemaType::Primitive(actual)) => {
            primitive_compatible(expected, actual)
        }
        (SchemaType::Union(types), actual) => {
            types.iter().any(|member| compatible_schema(member, actual))
        }
        (expected, SchemaType::Union(types)) => {
            types.iter().any(|member| compatible_schema(expected, member))
        }
        (SchemaType::Vector(expected), SchemaType::Vector(actual)) => {
            compatible_schema(expected, actual)
        }
        (SchemaType::Tuple(expected), SchemaType::Tuple(actual))
            if expected.len() == actual.len() =>
        {
            expected
                .iter()
                .zip(actual)
                .all(|(left, right)| compatible_schema(left, right))
        }
        (SchemaType::Map(expected), SchemaType::Map(actual)) => expected.iter().all(|left| {
            actual
                .iter()
                .find(|right| right.name == left.name)
                .map_or(true, |right| compatible_schema(&left.value_type, &right.value_type))
        }),
        (SchemaType::Enum(expected), SchemaType::Enum(actual)) => {
            expected.iter().any(|value| actual.contains(value))
        }
        _ => false,
    }
}

/// Directional assignment: every value in `actual` must fit `expected`.
pub fn assignable_schema(expected: &SchemaType, actual: &SchemaType) -> bool {
    if expected == actual {
        return true;
    }
    match (expected, actual) {
        (SchemaType::Unknown(_), _) | (_, SchemaType::Unknown(_)) => true,
        (SchemaType::Reference(_), _) | (_, SchemaType::Reference(_)) => true,
        (SchemaType::Primitive(name), _) if name == "any" => true,
        (SchemaType::Union(expected), SchemaType::Union(actual)) => actual.iter().all(|member| {
            expected
                .iter()
                .any(|candidate| assignable_schema(candidate, member))
        }),
        (SchemaType::Union(expected), actual) => expected
            .iter()
            .any(|candidate| assignable_schema(candidate, actual)),
        (expected, SchemaType::Union(actual)) => {
            actual.iter().all(|member| assignable_schema(expected, member))
        }
        (SchemaType::Primitive(expected), SchemaType::Primitive(actual)) => {
            expected == actual
                || (expected == "num" && matches!(actual.as_str(), "int" | "float" | "decimal"))
        }
        (SchemaType::Vector(expected), SchemaType::Vector(actual)) => {
            assignable_schema(expected, actual)
        }
        (SchemaType::Tuple(expected), SchemaType::Tuple(actual))
            if expected.len() == actual.len() =>
        {
            expected
                .iter()
                .zip(actual)
                .all(|(left, right)| assignable_schema(left, right))
        }
        (SchemaType::Map(expected), SchemaType::Map(actual)) => expected.iter().all(|left| {
            actual
                .iter()
                .find(|right| right.name == left.name)
                .is_some_and(|right| assignable_schema(&left.value_type, &right.value_type))
        }),
        (SchemaType::Enum(expected), SchemaType::Enum(actual)) => {
            actual.iter().all(|value| expected.contains(value))
        }
        (SchemaType::Function(expected), SchemaType::Function(actual)) => {
            expected.iter().all(|expected_arity| {
                actual.iter().any(|actual_arity| {
                    expected_arity.fixed.len() == actual_arity.fixed.len()
                        && expected_arity.rest.is_some() == actual_arity.rest.is_some()
                        && actual_arity
                            .fixed
                            .iter()
                            .zip(&expected_arity.fixed)
                            .all(|(actual_input, expected_input)| {
                                assignable_schema(actual_input, expected_input)
                            })
                        && match (&actual_arity.rest, &expected_arity.rest) {
                            (Some(actual_rest), Some(expected_rest)) => {
                                assignable_schema(actual_rest, expected_rest)
                            }
                            (None, None) => true,
                            _ => false,
                        }
                        && assignable_schema(&expected_arity.output, &actual_arity.output)
                })
            })
        }
        _ => false,
    }
}

fn function_arities(schema: &SchemaType) -> Option<&[FunctionSchema]> {
    match schema {
        SchemaType::Function(arities) => Some(arities),
        _ => None,
    }
}

fn arity_matches(arity: &FunctionSchema, argument_count: usize) -> bool {
    if arity.rest.is_some() {
        arity.fixed.len() <= argument_count
    } else {
        arity.fixed.len() == argument_count
    }
}

fn matching_arity(schema: &SchemaType, argument_count: usize) -> Option<&FunctionSchema> {
    function_arities(schema)?
        .iter()
        .find(|arity| arity_matches(arity, argument_count))
}

fn unknown_type() -> SchemaType {
    SchemaType::Unknown(Form::Symbol("?".into()))
}

fn join_types(types: impl IntoIterator<Item = SchemaType>) -> SchemaType {
    let mut members = Vec::new();
    for value in types {
        if matches!(value, SchemaType::Unknown(_)) {
            return unknown_type();
        }
        match value {
            SchemaType::Union(nested) => {
                for member in nested {
                    if matches!(member, SchemaType::Unknown(_)) {
                        return unknown_type();
                    }
                    push_unique(&mut members, member);
                }
            }
            member => push_unique(&mut members, member),
        }
    }
    match members.len() {
        0 => unknown_type(),
        1 => members.pop().expect("singleton join has one member"),
        _ => SchemaType::Union(members),
    }
}

fn infer_literal(form: &Form) -> SchemaType {
    match super::super::core::form_without_metadata(form) {
        Form::Nil => SchemaType::Primitive("nil".into()),
        Form::Bool(_) => SchemaType::Primitive("bool".into()),
        Form::Number(_) | Form::BigInteger(_) => SchemaType::Primitive("int".into()),
        Form::Float(_) => SchemaType::Primitive("float".into()),
        Form::Decimal(_) => SchemaType::Primitive("decimal".into()),
        Form::Character(_) => SchemaType::Primitive("char".into()),
        Form::Regex(_) => SchemaType::Primitive("regex".into()),
        Form::String(_) => SchemaType::Primitive("str".into()),
        Form::Keyword(_) => SchemaType::Primitive("keyword".into()),
        Form::Symbol(_) => SchemaType::Primitive("symbol".into()),
        Form::Vector(values) => {
            SchemaType::Vector(Box::new(join_types(values.iter().map(infer_literal))))
        }
        Form::Map(entries) => SchemaType::Map(
            entries
                .iter()
                .map(|(name, value)| SchemaField {
                    name: name.clone(),
                    value_type: infer_literal(value),
                })
                .collect(),
        ),
        Form::Set(_) => SchemaType::Primitive("set".into()),
        Form::List(_) => SchemaType::Primitive("list".into()),
        Form::Tagged(_, value) => infer_literal(value),
        Form::Metadata(_, value) => infer_literal(value),
    }
}

/// Infers a conservative normalized schema without evaluating the form.
pub fn infer_schema(
    form: &Form,
    environment: &HashMap<String, SchemaType>,
    functions: &HashMap<String, SchemaType>,
) -> SchemaType {
    let mut environment = environment.clone();
    infer_expression(form, &mut environment, functions)
}

fn infer_expression(
    form: &Form,
    environment: &mut HashMap<String, SchemaType>,
    functions: &HashMap<String, SchemaType>,
) -> SchemaType {
    match super::super::core::form_without_metadata(form) {
        Form::Nil => SchemaType::Primitive("nil".into()),
        Form::Bool(_) => SchemaType::Primitive("bool".into()),
        Form::Number(_) | Form::BigInteger(_) => SchemaType::Primitive("int".into()),
        Form::Float(_) => SchemaType::Primitive("float".into()),
        Form::Decimal(_) => SchemaType::Primitive("decimal".into()),
        Form::Character(_) => SchemaType::Primitive("char".into()),
        Form::Regex(_) => SchemaType::Primitive("regex".into()),
        Form::String(_) => SchemaType::Primitive("str".into()),
        Form::Keyword(_) => SchemaType::Primitive("keyword".into()),
        Form::Symbol(name) => environment.get(name).cloned().unwrap_or_else(unknown_type),
        Form::Vector(values) => SchemaType::Vector(Box::new(join_types(
            values
                .iter()
                .map(|value| infer_expression(value, environment, functions)),
        ))),
        Form::Map(entries) => SchemaType::Map(
            entries
                .iter()
                .map(|(name, value)| SchemaField {
                    name: name.clone(),
                    value_type: infer_expression(value, environment, functions),
                })
                .collect(),
        ),
        Form::Set(_) => SchemaType::Primitive("set".into()),
        Form::List(items) if items.is_empty() => SchemaType::Primitive("list".into()),
        Form::List(items) => infer_list(items, environment, functions),
        Form::Tagged(_, value) => infer_expression(value, environment, functions),
        Form::Metadata(_, value) => infer_expression(value, environment, functions),
    }
}

fn infer_list(
    items: &[Form],
    environment: &mut HashMap<String, SchemaType>,
    functions: &HashMap<String, SchemaType>,
) -> SchemaType {
    let Some(Form::Symbol(operator)) = items.first() else {
        return unknown_type();
    };
    match operator.as_str() {
        "quote" if items.len() >= 2 => infer_literal(&items[1]),
        "do" => items[1..]
            .iter()
            .map(|value| infer_expression(value, environment, functions))
            .last()
            .unwrap_or_else(|| SchemaType::Primitive("nil".into())),
        "if" if items.len() >= 3 => {
            let mut branches = items[2..]
                .iter()
                .map(|value| infer_expression(value, environment, functions))
                .collect::<Vec<_>>();
            if branches.len() == 1 {
                branches.push(SchemaType::Primitive("nil".into()));
            }
            join_types(branches)
        }
        "let" | "loop" if items.len() >= 3 => {
            let mut nested = environment.clone();
            if let Form::Vector(bindings) = super::super::core::form_without_metadata(&items[1]) {
                for pair in bindings.chunks(2) {
                    if let [name, value] = pair {
                        if let Some(name) = binding_name(name) {
                            let value_type = infer_expression(value, &mut nested, functions);
                            nested.insert(name.to_owned(), value_type);
                        }
                    }
                }
            }
            items[2..]
                .iter()
                .map(|value| infer_expression(value, &mut nested, functions))
                .last()
                .unwrap_or_else(|| SchemaType::Primitive("nil".into()))
        }
        "+" | "-" | "*" | "%" | "mod" => {
            infer_numeric(&items[1..], environment, functions, false)
        }
        "/" => infer_numeric(&items[1..], environment, functions, true),
        "=" | "not=" | "<" | "<=" | ">" | ">=" | "identical?" | "instance?"
        | "nil?" | "some?" => SchemaType::Primitive("bool".into()),
        "count" => SchemaType::Primitive("int".into()),
        "str" => SchemaType::Primitive("str".into()),
        "keyword" => SchemaType::Primitive("keyword".into()),
        "symbol" => SchemaType::Primitive("symbol".into()),
        "vector" => SchemaType::Vector(Box::new(join_types(
            items[1..]
                .iter()
                .map(|value| infer_expression(value, environment, functions)),
        ))),
        "hash-map" => SchemaType::Map(
            items[1..]
                .chunks(2)
                .filter_map(|pair| match pair {
                    [name, value] => Some(SchemaField {
                        name: name.clone(),
                        value_type: infer_expression(value, environment, functions),
                    }),
                    _ => None,
                })
                .collect(),
        ),
        _ => functions
            .get(operator)
            .and_then(|schema| matching_arity(schema, items.len() - 1))
            .map(|arity| (*arity.output).clone())
            .unwrap_or_else(unknown_type),
    }
}

fn infer_numeric(
    arguments: &[Form],
    environment: &mut HashMap<String, SchemaType>,
    functions: &HashMap<String, SchemaType>,
    divide: bool,
) -> SchemaType {
    if arguments.is_empty() {
        return unknown_type();
    }
    let types = arguments
        .iter()
        .map(|value| infer_expression(value, environment, functions))
        .collect::<Vec<_>>();
    if types
        .iter()
        .any(|value| matches!(value, SchemaType::Unknown(_)))
        || !types.iter().all(|value| {
            assignable_schema(&SchemaType::Primitive("num".into()), value)
        })
    {
        return unknown_type();
    }
    if divide {
        SchemaType::Primitive("num".into())
    } else if types
        .iter()
        .all(|value| value == &SchemaType::Primitive("int".into()))
    {
        SchemaType::Primitive("int".into())
    } else {
        SchemaType::Primitive("num".into())
    }
}

fn binding_name(form: &Form) -> Option<&str> {
    match form {
        Form::Symbol(name) => Some(name),
        Form::Metadata(_, value) => binding_name(value),
        _ => None,
    }
}

fn declared_arity<'a>(
    declared: Option<&'a SchemaType>,
    fixed_count: usize,
    variadic: bool,
) -> Option<&'a FunctionSchema> {
    function_arities(declared?)?.iter().find(|arity| {
        arity.fixed.len() == fixed_count && arity.rest.is_some() == variadic
    })
}

fn infer_function_arity(
    parameters: &[Form],
    body: &[Form],
    declared: Option<&SchemaType>,
    functions: &HashMap<String, SchemaType>,
) -> FunctionSchema {
    let fixed_count = parameters
        .iter()
        .take_while(|form| !matches!(form, Form::Symbol(marker) if marker == "&"))
        .count();
    let variadic = parameters
        .iter()
        .any(|form| matches!(form, Form::Symbol(marker) if marker == "&"));
    let declared_arity = declared_arity(declared, fixed_count, variadic);
    let mut environment = HashMap::new();
    let mut fixed = Vec::new();
    let mut rest = None;
    let mut parameter_index = 0;
    let mut after_rest = false;
    for parameter in parameters {
        if matches!(parameter, Form::Symbol(marker) if marker == "&") {
            after_rest = true;
            continue;
        }
        let Some(parameter_name) = binding_name(parameter) else {
            continue;
        };
        let parameter_type = if after_rest {
            declared_arity
                .and_then(|arity| arity.rest.as_deref())
                .cloned()
                .unwrap_or_else(unknown_type)
        } else {
            declared_arity
                .and_then(|arity| arity.fixed.get(parameter_index))
                .cloned()
                .unwrap_or_else(unknown_type)
        };
        if after_rest {
            environment.insert(
                parameter_name.to_owned(),
                SchemaType::Vector(Box::new(parameter_type.clone())),
            );
            rest = Some(Box::new(parameter_type));
        } else {
            environment.insert(parameter_name.to_owned(), parameter_type.clone());
            fixed.push(parameter_type);
            parameter_index += 1;
        }
    }
    let output = body
        .iter()
        .map(|form| infer_expression(form, &mut environment, functions))
        .last()
        .unwrap_or_else(|| SchemaType::Primitive("nil".into()));
    FunctionSchema {
        fixed,
        rest,
        output: Box::new(output),
    }
}

fn function_environment(
    namespace: &str,
    declarations: &HashMap<String, SchemaType>,
    definitions: &HashMap<String, SchemaType>,
    inferred: &HashMap<String, SchemaType>,
) -> HashMap<String, SchemaType> {
    let mut output = inferred.clone();
    for (name, schema) in declarations {
        output.insert(name.clone(), resolve_schema(schema, definitions));
    }
    let prefix = format!("{namespace}/");
    for (name, schema) in output.clone() {
        if let Some(local) = name.strip_prefix(&prefix) {
            output.insert(local.into(), schema);
        }
    }
    output
}

fn infer_function_pass(
    namespace: &str,
    forms: &[Form],
    declarations: &HashMap<String, SchemaType>,
    definitions: &HashMap<String, SchemaType>,
    seed: &HashMap<String, SchemaType>,
) -> HashMap<String, SchemaType> {
    let mut inferred = seed.clone();
    for form in forms {
        let Form::List(items) = super::super::core::form_without_metadata(form) else {
            continue;
        };
        if !matches!(items.first(), Some(Form::Symbol(operator)) if operator == "defn" || operator == "defn-")
        {
            continue;
        }
        let Some(name) = items.get(1).and_then(binding_name) else {
            continue;
        };
        let qualified = format!("{namespace}/{name}");
        let declared = declarations
            .get(&qualified)
            .map(|schema| resolve_schema(schema, definitions));
        let functions = function_environment(namespace, declarations, definitions, &inferred);
        let parameters_at = items.iter().enumerate().skip(2).find_map(|(index, value)| {
            matches!(
                super::super::core::form_without_metadata(value),
                Form::Vector(_)
            )
            .then_some(index)
        });
        let mut arities = Vec::new();
        if let Some(parameters_at) = parameters_at {
            let Form::Vector(parameters) =
                super::super::core::form_without_metadata(&items[parameters_at])
            else {
                continue;
            };
            arities.push(infer_function_arity(
                parameters,
                &items[parameters_at + 1..],
                declared.as_ref(),
                &functions,
            ));
        } else {
            for clause in items.iter().skip(2) {
                let Form::List(clause) = super::super::core::form_without_metadata(clause) else {
                    continue;
                };
                let Some(Form::Vector(parameters)) = clause.first() else {
                    continue;
                };
                arities.push(infer_function_arity(
                    parameters,
                    &clause[1..],
                    declared.as_ref(),
                    &functions,
                ));
            }
        }
        if !arities.is_empty() {
            inferred.insert(qualified, SchemaType::Function(arities));
        }
    }
    inferred
}

/// Infers conservative function signatures from executable module forms.
/// Declared schemas seed parameter types, but inferred results remain a
/// separate table: annotations are contracts, while these are optimizer facts.
pub fn infer_function_types(
    namespace: &str,
    forms: &[Form],
    declarations: &HashMap<String, SchemaType>,
    definitions: &HashMap<String, SchemaType>,
) -> HashMap<String, SchemaType> {
    let mut current = HashMap::new();
    for _ in 0..=forms.len() {
        let next = infer_function_pass(namespace, forms, declarations, definitions, &current);
        if next == current {
            return next;
        }
        current = next;
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::{parse, parse_forms};

    fn join(values: impl IntoIterator<Item = String>) -> String {
        values.into_iter().collect::<Vec<_>>().join(",")
    }

    fn canonical(schema: &SchemaType) -> String {
        match schema {
            SchemaType::Primitive(name) => format!("primitive(:{name})"),
            SchemaType::Reference(name) => format!("reference({name})"),
            SchemaType::Union(types) => {
                format!("union[{}]", join(types.iter().map(canonical)))
            }
            SchemaType::Vector(item) => format!("vector({})", canonical(item)),
            SchemaType::Tuple(items) => {
                format!("tuple[{}]", join(items.iter().map(canonical)))
            }
            SchemaType::Map(fields) => format!(
                "map[{}]",
                join(fields
                    .iter()
                    .map(|field| format!("{}={}", field.name, canonical(&field.value_type))))
            ),
            SchemaType::Function(arities) => {
                let arities = arities.iter().map(|arity| {
                    format!(
                        "fn(fixed=[{}],rest={},output={})",
                        join(arity.fixed.iter().map(canonical)),
                        arity
                            .rest
                            .as_ref()
                            .map(|value| canonical(value))
                            .unwrap_or_else(|| "none".into()),
                        canonical(&arity.output)
                    )
                });
                let values = arities.collect::<Vec<_>>();
                if values.len() == 1 {
                    values[0].clone()
                } else {
                    format!("function[{}]", values.join(","))
                }
            }
            SchemaType::Enum(values) => {
                format!("enum[{}]", join(values.iter().map(ToString::to_string)))
            }
            SchemaType::Unknown(_) => "unknown".into(),
            SchemaType::Extension { head, .. } => format!("extension({head})"),
        }
    }

    fn error_code(error: &str) -> &str {
        error.split_once(':').map_or(error, |(code, _)| code)
    }

    fn parity_cases() -> Vec<Form> {
        let forms = parse_forms(include_str!(
            "../../../lib/test/std/typed/parity_corpus.hal"
        ))
        .expect("shared std.typed parity corpus parses");
        forms
            .iter()
            .find_map(|form| match form {
                Form::List(items)
                    if matches!(items.first(), Some(Form::Symbol(operator)) if operator == "def")
                        && matches!(items.get(1), Some(Form::Symbol(name)) if name == "+cases+") =>
                {
                    match items.get(2) {
                        Some(Form::Vector(cases)) => Some(cases.clone()),
                        _ => None,
                    }
                }
                _ => None,
            })
            .expect("parity corpus defines +cases+")
    }

    fn string_at(items: &[Form], index: usize) -> &str {
        match &items[index] {
            Form::String(value) => value,
            other => panic!("expected string at {index}, got {other}"),
        }
    }

    #[test]
    fn matches_shared_std_typed_parity_corpus() {
        for case in parity_cases() {
            let Form::Vector(items) = case else {
                panic!("parity case must be a vector")
            };
            let Some(Form::Keyword(operation)) = items.first() else {
                panic!("parity operation must be a keyword")
            };
            let id = items.get(1).map(ToString::to_string).unwrap_or_default();
            match operation.as_str() {
                "normalize" => {
                    let actual = canonical(
                        &normalize_schema(&parse(string_at(&items, 2)).unwrap()).unwrap(),
                    );
                    assert_eq!(actual, string_at(&items, 3), "{id}");
                }
                "error" => {
                    let error = normalize_schema(&parse(string_at(&items, 2)).unwrap())
                        .expect_err("error parity case must fail");
                    let Form::Keyword(expected) = &items[3] else {
                        panic!("error case requires a keyword code")
                    };
                    assert_eq!(error_code(&error), expected, "{id}: {error}");
                }
                "assignable" => {
                    let expected = normalize_schema(&parse(string_at(&items, 2)).unwrap()).unwrap();
                    let actual = normalize_schema(&parse(string_at(&items, 3)).unwrap()).unwrap();
                    let Form::Bool(result) = &items[4] else {
                        panic!("assignable case requires a boolean result")
                    };
                    assert_eq!(assignable_schema(&expected, &actual), *result, "{id}");
                }
                "compatible" => {
                    let expected = normalize_schema(&parse(string_at(&items, 2)).unwrap()).unwrap();
                    let actual = normalize_schema(&parse(string_at(&items, 3)).unwrap()).unwrap();
                    let Form::Bool(result) = &items[4] else {
                        panic!("compatible case requires a boolean result")
                    };
                    assert_eq!(compatible_schema(&expected, &actual), *result, "{id}");
                }
                "infer" => {
                    let inferred = infer_schema(
                        &parse(string_at(&items, 2)).unwrap(),
                        &HashMap::new(),
                        &HashMap::new(),
                    );
                    assert_eq!(canonical(&inferred), string_at(&items, 3), "{id}");
                }
                "infer-call" => {
                    let name = string_at(&items, 3).to_owned();
                    let contract =
                        normalize_schema(&parse(string_at(&items, 4)).unwrap()).unwrap();
                    let inferred = infer_schema(
                        &parse(string_at(&items, 2)).unwrap(),
                        &HashMap::new(),
                        &HashMap::from([(name, contract)]),
                    );
                    assert_eq!(canonical(&inferred), string_at(&items, 5), "{id}");
                }
                other => panic!("unsupported parity operation: {other}"),
            }
        }
    }

    #[test]
    fn normalizes_nested_named_function_schemas() {
        assert_eq!(
            normalize_schema(&parse("[:fn [#'demo/Customer & :int] [:maybe :str]]").unwrap())
                .unwrap(),
            SchemaType::Function(vec![FunctionSchema {
                fixed: vec![SchemaType::Reference("demo/Customer".into())],
                rest: Some(Box::new(SchemaType::Primitive("int".into()))),
                output: Box::new(SchemaType::Union(vec![
                    SchemaType::Primitive("str".into()),
                    SchemaType::Primitive("nil".into()),
                ])),
            }])
        );
    }

    #[test]
    fn resolves_recursive_references_without_expanding_cycles() {
        let node = normalize_schema(&parse("[:map [:next [:maybe #'demo/Node]]]").unwrap())
            .unwrap();
        let resolved = resolve_schema(
            &SchemaType::Reference("demo/Node".into()),
            &HashMap::from([("demo/Node".into(), node)]),
        );
        assert_eq!(
            canonical(&resolved),
            "map[:next=union[reference(demo/Node),primitive(:nil)]]"
        );
    }

    #[test]
    fn separates_compatibility_from_directional_assignment() {
        let number = normalize_schema(&parse(":num").unwrap()).unwrap();
        let integer = normalize_schema(&parse(":int").unwrap()).unwrap();
        assert!(compatible_schema(&number, &integer));
        assert!(assignable_schema(&number, &integer));
        assert!(!assignable_schema(&integer, &number));
    }

    #[test]
    fn infers_body_results_without_replacing_declared_contracts() {
        let forms = parse_forms(
            "(ns demo)\n\
             (def Unary [:fn [:int] :num])\n\
             (defn ^{:schema #'Unary} choose [value]\n\
               (let [next (+ value 1)] (if true next 0)))\n\
             (defn labels [] {:name \"Ada\" :active true})\n\
             (defn select ([value] value) ([left right] right))",
        )
        .unwrap();
        let declarations = HashMap::from([(
            "demo/choose".into(),
            SchemaType::Reference("demo/Unary".into()),
        )]);
        let definitions = HashMap::from([(
            "demo/Unary".into(),
            normalize_schema(&parse("[:fn [:int] :num]").unwrap()).unwrap(),
        )]);
        let inferred = infer_function_types("demo", &forms, &declarations, &definitions);

        assert!(matches!(
            inferred.get("demo/choose"),
            Some(SchemaType::Function(arities))
                if arities[0].fixed == vec![SchemaType::Primitive("int".into())]
                    && *arities[0].output == SchemaType::Primitive("int".into())
        ));
        assert!(matches!(
            inferred.get("demo/labels"),
            Some(SchemaType::Function(arities))
                if matches!(arities[0].output.as_ref(), SchemaType::Map(fields) if fields.len() == 2)
        ));
        assert!(matches!(
            inferred.get("demo/select"),
            Some(SchemaType::Function(arities)) if arities.len() == 2
        ));
    }
}
