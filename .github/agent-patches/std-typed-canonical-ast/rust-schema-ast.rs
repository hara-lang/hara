fn schema_kind(schema: &crate::kernel::SchemaType) -> &'static str {
    use crate::kernel::SchemaType::*;
    match schema {
        Primitive(_) => "primitive",
        Reference(_) => "reference",
        Union(_) => "union",
        Vector(_) => "vector",
        Tuple(_) => "tuple",
        Map(_) => "map",
        Function(arities) if arities.len() == 1 => "fn",
        Function(_) => "function",
        Enum(_) => "enum",
        Extension { .. } => "extension",
        Unknown(_) => "unknown",
    }
}

fn schema_ast_map(entries: Vec<(&str, Form)>) -> Form {
    Form::Map(
        entries
            .into_iter()
            .map(|(key, value)| (Form::Keyword(key.into()), value))
            .collect(),
    )
}

fn schema_function_ast(arity: &crate::kernel::FunctionSchema) -> Form {
    schema_ast_map(vec![
        ("kind", Form::Keyword("fn".into())),
        (
            "inputs",
            schema_ast_map(vec![
                (
                    "fixed",
                    Form::Vector(arity.fixed.iter().map(schema_ast_form).collect()),
                ),
                (
                    "rest",
                    arity
                        .rest
                        .as_deref()
                        .map(schema_ast_form)
                        .unwrap_or(Form::Nil),
                ),
            ]),
        ),
        ("output", schema_ast_form(&arity.output)),
    ])
}

fn schema_ast_form(schema: &crate::kernel::SchemaType) -> Form {
    use crate::kernel::SchemaType::*;
    match schema {
        Primitive(name) => schema_ast_map(vec![
            ("kind", Form::Keyword("primitive".into())),
            ("name", Form::Keyword(name.clone())),
        ]),
        Reference(name) => schema_ast_map(vec![
            ("kind", Form::Keyword("reference".into())),
            ("name", Form::Symbol(name.clone())),
        ]),
        Union(values) => schema_ast_map(vec![
            ("kind", Form::Keyword("union".into())),
            (
                "types",
                Form::Vector(values.iter().map(schema_ast_form).collect()),
            ),
        ]),
        Vector(value) => schema_ast_map(vec![
            ("kind", Form::Keyword("vector".into())),
            ("item", schema_ast_form(value)),
        ]),
        Tuple(values) => schema_ast_map(vec![
            ("kind", Form::Keyword("tuple".into())),
            (
                "items",
                Form::Vector(values.iter().map(schema_ast_form).collect()),
            ),
        ]),
        Map(fields) => schema_ast_map(vec![
            ("kind", Form::Keyword("map".into())),
            (
                "fields",
                Form::Vector(
                    fields
                        .iter()
                        .map(|field| {
                            schema_ast_map(vec![
                                ("name", field.name.clone()),
                                ("type", schema_ast_form(&field.value_type)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]),
        Function(arities) if arities.len() == 1 => schema_function_ast(&arities[0]),
        Function(arities) => schema_ast_map(vec![
            ("kind", Form::Keyword("function".into())),
            (
                "arities",
                Form::Vector(arities.iter().map(schema_function_ast).collect()),
            ),
        ]),
        Enum(values) => schema_ast_map(vec![
            ("kind", Form::Keyword("enum".into())),
            ("values", Form::Vector(values.clone())),
        ]),
        Extension { head, arguments } => {
            let surface = Form::Vector(
                std::iter::once(Form::Keyword(head.clone()))
                    .chain(arguments.iter().cloned())
                    .collect(),
            );
            schema_ast_map(vec![
                ("kind", Form::Keyword("extension".into())),
                ("head", Form::Keyword(head.clone())),
                ("arguments", Form::Vector(arguments.clone())),
                ("surface", surface),
            ])
        }
        Unknown(value) => schema_ast_map(vec![
            ("kind", Form::Keyword("unknown".into())),
            ("surface", value.clone()),
        ]),
    }
}
