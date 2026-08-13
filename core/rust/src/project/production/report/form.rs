use super::super::source::SourceLocation;
use crate::kernel::Form;
use std::collections::BTreeSet;

pub fn source_form(location: &SourceLocation) -> Form {
    form_map(vec![
        ("path", string_form(&location.path)),
        ("line", number(location.line)),
        ("column", number(location.column)),
        ("end-line", number(location.end_line)),
        ("end-column", number(location.end_column)),
    ])
}

pub fn form_map(values: Vec<(&str, Form)>) -> Form {
    Form::Map(
        values
            .into_iter()
            .map(|(key, value)| (Form::Keyword(key.into()), value))
            .collect(),
    )
}

pub fn keyword(value: &str) -> Form { Form::Keyword(value.into()) }
pub fn symbol(value: &str) -> Form { Form::Symbol(value.into()) }
pub fn string_form(value: &str) -> Form { Form::String(value.into()) }
pub fn number(value: usize) -> Form { Form::Number(i64::try_from(value).unwrap_or(i64::MAX)) }
pub fn symbols_form(values: &BTreeSet<String>) -> Form {
    Form::Vector(values.iter().map(|value| symbol(value)).collect())
}
pub fn strings_form(values: &BTreeSet<String>) -> Form {
    Form::Vector(values.iter().map(|value| string_form(value)).collect())
}
