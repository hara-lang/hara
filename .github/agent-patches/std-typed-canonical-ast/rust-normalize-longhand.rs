fn longhand_value<'a>(entries: &'a [(Form, Form)], name: &str) -> Option<&'a Form> {
    entries.iter().find_map(|(key, value)| {
        matches!(key, Form::Keyword(key) if key == name).then_some(value)
    })
}

fn longhand_children(entries: &[(Form, Form)]) -> Result<&[Form], String> {
    match longhand_value(entries, "children") {
        Some(Form::Vector(values)) => Ok(values),
        None => Ok(&[]),
        _ => Err("schema :children must be a vector".into()),
    }
}

fn longhand_sequence<'a>(
    entries: &'a [(Form, Form)],
    name: &str,
    fallback: &'a [Form],
) -> Result<&'a [Form], String> {
    match longhand_value(entries, name) {
        Some(Form::Vector(values)) => Ok(values),
        Some(_) => Err(format!("schema :{name} must be a vector")),
        None => Ok(fallback),
    }
}

fn normalize_reference_name(value: &Form) -> Result<SchemaType, String> {
    match value {
        Form::Symbol(name) if name.contains('/') => Ok(SchemaType::Reference(name.clone())),
        Form::Symbol(name) => Err(format!(
            "named schema reference is not fully qualified: {name}"
        )),
        _ => Err("named schema reference must target a symbol".into()),
    }
}

fn normalize_union_forms(values: &[Form]) -> Result<SchemaType, String> {
    if values.is_empty() {
        return Err(":or schema requires at least one member".into());
    }
    let mut members = Vec::new();
    for value in values {
        let normalized = normalize_schema(value)?;
        match normalized {
            SchemaType::Union(nested) => {
                for member in nested {
                    push_unique(&mut members, member);
                }
            }
            member => push_unique(&mut members, member),
        }
    }
    Ok(if members.len() == 1 {
        members.pop().unwrap()
    } else {
        SchemaType::Union(members)
    })
}

fn normalize_longhand_field(field: &Form) -> Result<SchemaField, String> {
    let Form::Map(entries) = field else {
        return Err("map schema fields must be {:name name :type schema} maps".into());
    };
    let name = longhand_value(entries, "name")
        .ok_or_else(|| "map schema field requires :name".to_string())?;
    let value_type = longhand_value(entries, "type")
        .ok_or_else(|| "map schema field requires :type".to_string())?;
    Ok(SchemaField {
        name: name.clone(),
        value_type: normalize_schema(value_type)?,
    })
}

fn normalize_function_inputs(
    inputs: &Form,
) -> Result<(Vec<SchemaType>, Option<Box<SchemaType>>), String> {
    match inputs {
        Form::Map(entries) => {
            let fixed = match longhand_value(entries, "fixed") {
                Some(Form::Vector(values)) => values
                    .iter()
                    .map(normalize_schema)
                    .collect::<Result<Vec<_>, _>>()?,
                None => Vec::new(),
                _ => return Err("function schema :fixed must be a vector".into()),
            };
            let rest = match longhand_value(entries, "rest") {
                None | Some(Form::Nil) => None,
                Some(value) => Some(Box::new(normalize_schema(value)?)),
            };
            Ok((fixed, rest))
        }
        Form::Vector(values) => {
            let mut fixed = Vec::new();
            let mut rest = None;
            let mut index = 0;
            while index < values.len() {
                if matches!(&values[index], Form::Symbol(marker) if marker == "&") {
                    if rest.is_some() || index + 2 != values.len() {
                        return Err(":fn schema & must precede exactly one rest type".into());
                    }
                    rest = Some(Box::new(normalize_schema(&values[index + 1])?));
                    index += 2;
                } else {
                    fixed.push(normalize_schema(&values[index])?);
                    index += 1;
                }
            }
            Ok((fixed, rest))
        }
        _ => Err("function schema :inputs must be a vector or map".into()),
    }
}

fn normalize_longhand_function(entries: &[(Form, Form)]) -> Result<FunctionSchema, String> {
    let inputs = longhand_value(entries, "inputs")
        .ok_or_else(|| "function schema requires :inputs".to_string())?;
    let output = longhand_value(entries, "output")
        .ok_or_else(|| "function schema requires :output".to_string())?;
    let (fixed, rest) = normalize_function_inputs(inputs)?;
    Ok(FunctionSchema {
        fixed,
        rest,
        output: Box::new(normalize_schema(output)?),
    })
}

fn normalize_longhand_functions(values: &[Form]) -> Result<SchemaType, String> {
    if values.is_empty() {
        return Err(":function schema requires at least one :fn schema".into());
    }
    let mut arities = Vec::new();
    for value in values {
        match value {
            Form::Map(entries) if longhand_value(entries, "kind").is_none() => {
                arities.push(normalize_longhand_function(entries)?);
            }
            _ => match normalize_schema(value)? {
                SchemaType::Function(nested) => arities.extend(nested),
                _ => return Err(":function members must be :fn schemas".into()),
            },
        }
    }
    Ok(SchemaType::Function(arities))
}

fn normalize_longhand(entries: &[(Form, Form)]) -> Result<SchemaType, String> {
    let Some(Form::Keyword(kind)) = longhand_value(entries, "kind") else {
        return Ok(SchemaType::Unknown(Form::Map(entries.to_vec())));
    };
    let children = longhand_children(entries)?;
    match kind.as_str() {
        "primitive" => {
            let value = longhand_value(entries, "name").or_else(|| children.first());
            match value {
                Some(Form::Keyword(name)) => Ok(SchemaType::Primitive(name.clone())),
                _ => Err("primitive schema requires one keyword name".into()),
            }
        }
        "reference" => {
            let value = longhand_value(entries, "name").or_else(|| children.first());
            value
                .ok_or_else(|| "reference schema requires :name".to_string())
                .and_then(normalize_reference_name)
        }
        "union" | "or" => {
            normalize_union_forms(longhand_sequence(entries, "types", children)?)
        }
        "vector" => {
            let value = longhand_value(entries, "item").or_else(|| children.first());
            value
                .ok_or_else(|| "vector schema requires :item".to_string())
                .and_then(normalize_schema)
                .map(|value| SchemaType::Vector(Box::new(value)))
        }
        "tuple" => longhand_sequence(entries, "items", children)?
            .iter()
            .map(normalize_schema)
            .collect::<Result<Vec<_>, _>>()
            .map(SchemaType::Tuple),
        "map" => {
            if longhand_value(entries, "fields").is_some() {
                longhand_sequence(entries, "fields", &[])?
                    .iter()
                    .map(normalize_longhand_field)
                    .collect::<Result<Vec<_>, _>>()
                    .map(SchemaType::Map)
            } else {
                children
                    .iter()
                    .map(|child| match child {
                        Form::Vector(pair) if pair.len() == 2 => Ok(SchemaField {
                            name: pair[0].clone(),
                            value_type: normalize_schema(&pair[1])?,
                        }),
                        _ => Err("map schema children must be [name schema] pairs".into()),
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(SchemaType::Map)
            }
        }
        "fn" => normalize_longhand_function(entries).map(|arity| SchemaType::Function(vec![arity])),
        "function" => normalize_longhand_functions(longhand_sequence(
            entries,
            "arities",
            children,
        )?),
        "enum" => Ok(SchemaType::Enum(
            longhand_sequence(entries, "values", children)?.to_vec(),
        )),
        "extension" => {
            let head = longhand_value(entries, "head")
                .or_else(|| longhand_value(entries, "name"))
                .ok_or_else(|| "extension schema requires :head".to_string())?;
            let Form::Keyword(head) = head else {
                return Err("extension schema :head must be a keyword".into());
            };
            Ok(SchemaType::Extension {
                head: head.clone(),
                arguments: longhand_sequence(entries, "arguments", children)?.to_vec(),
            })
        }
        "unknown" => Ok(SchemaType::Unknown(
            longhand_value(entries, "surface")
                .or_else(|| children.first())
                .cloned()
                .unwrap_or_else(|| Form::Map(entries.to_vec())),
        )),
        _ => Err(format!("unsupported longhand schema kind: {kind}")),
    }
}
