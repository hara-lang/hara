fn value_to_metadata(value: &Value) -> Result<MetadataValue, String> {
    match value {
        Value::Nil => Ok(MetadataValue::Nil),
        Value::Bool(value) => Ok(MetadataValue::Boolean(*value)),
        Value::Number(value) => Ok(MetadataValue::Number(*value)),
        Value::Float(value) => Ok(MetadataValue::Float(*value)),
        Value::BigInteger(value) => Ok(MetadataValue::BigInteger(value.clone())),
        Value::Decimal(value) => Ok(MetadataValue::Decimal(value.clone())),
        Value::Character(value) => Ok(MetadataValue::Character(*value)),
        Value::Regex(value) => Ok(MetadataValue::Regex(value.clone())),
        Value::Tagged(value) => Ok(MetadataValue::Tagged(
            value.tag().get_name().into(),
            Box::new(value_to_metadata(value.form())?),
        )),
        Value::String(value) => Ok(MetadataValue::String(value.clone())),
        Value::Keyword(value) => Ok(MetadataValue::Keyword(value.clone())),
        Value::Symbol(value) => Ok(MetadataValue::Symbol(value.clone())),
        Value::Tuple(values) => Ok(MetadataValue::Vector(
            values
                .iter()
                .map(value_to_metadata)
                .collect::<Result<_, _>>()?,
        )),
        Value::Vector(values) => Ok(MetadataValue::Vector(
            values
                .iter()
                .map(value_to_metadata)
                .collect::<Result<_, _>>()?,
        )),
        Value::Queue(values) => Ok(MetadataValue::List(
            values
                .iter()
                .map(value_to_metadata)
                .collect::<Result<_, _>>()?,
        )),
        Value::Deque(values) => Ok(MetadataValue::List(
            values
                .iter()
                .map(value_to_metadata)
                .collect::<Result<_, _>>()?,
        )),
        Value::List(values) => Ok(MetadataValue::List(
            values
                .iter()
                .map(value_to_metadata)
                .collect::<Result<_, _>>()?,
        )),
        value @ (Value::Set(_) | Value::OrderedSet(_) | Value::SortedSet(_)) => {
            Ok(MetadataValue::Set(
                set_items(value)
                    .unwrap()
                    .into_iter()
                    .map(value_to_metadata)
                    .collect::<Result<_, _>>()?,
            ))
        }
        value @ (Value::Map(_) | Value::OrderedMap(_) | Value::SortedMap(_) | Value::Trie(_)) => {
            Ok(MetadataValue::Map(
                map_entries(value)
                    .unwrap()
                    .iter()
                    .map(|(key, value)| Ok((value_to_metadata(key)?, value_to_metadata(value)?)))
                    .collect::<Result<_, String>>()?,
            ))
        }
        _ => Err("value cannot be stored in runtime-neutral metadata".into()),
    }
}

fn metadata_to_value(value: &MetadataValue) -> Result<Value, String> {
    match value {
        MetadataValue::Nil => Ok(Value::Nil),
        MetadataValue::Boolean(value) => Ok(Value::Bool(*value)),
        MetadataValue::Number(value) => Ok(Value::Number(*value)),
        MetadataValue::Float(value) => Ok(Value::Float(*value)),
        MetadataValue::BigInteger(value) => Ok(Value::BigInteger(value.clone())),
        MetadataValue::Decimal(value) => Ok(Value::Decimal(value.clone())),
        MetadataValue::Character(value) => Ok(Value::Character(*value)),
        MetadataValue::Regex(value) => Ok(Value::Regex(value.clone())),
        MetadataValue::Tagged(tag, value) => Ok(Value::Tagged(Box::new(PTaggedLiteral::new(
            Symbol::parse(tag),
            metadata_to_value(value)?,
        )))),
        MetadataValue::String(value) => Ok(Value::String(value.clone())),
        MetadataValue::Keyword(value) => Ok(Value::Keyword(value.clone())),
        MetadataValue::Symbol(value) => Ok(Value::Symbol(value.clone())),
        MetadataValue::Vector(values) => Ok(Value::Vector(
            values
                .iter()
                .map(metadata_to_value)
                .collect::<Result<_, _>>()?,
        )),
        MetadataValue::List(values) => Ok(Value::List(
            values
                .iter()
                .map(metadata_to_value)
                .collect::<Result<_, _>>()?,
        )),
        MetadataValue::Set(values) => Ok(Value::Set(
            values
                .iter()
                .map(metadata_to_value)
                .collect::<Result<Vec<_>, _>>()?
                .into(),
        )),
        MetadataValue::Map(values) => Ok(Value::Map(
            values
                .iter()
                .map(|(key, value)| Ok((metadata_to_value(key)?, metadata_to_value(value)?)))
                .collect::<Result<Vec<_>, String>>()?
                .into_iter()
                .collect(),
        )),
    }
}

fn value_metadata(value: &Value) -> Option<Rc<Metadata>> {
    match value {
        Value::Symbol(value) => value.meta().cloned(),
        Value::Pointer(value) => value.meta().cloned(),
        Value::Tuple(value) => value.meta().cloned(),
        Value::Vector(value) => value.meta().cloned(),
        Value::List(value) => value.meta().cloned(),
        Value::Cons(value) => value.meta().cloned(),
        Value::Queue(value) => value.meta().cloned(),
        Value::Deque(value) => value.meta().cloned(),
        Value::Map(value) => value.meta().cloned(),
        Value::OrderedMap(value) => value.meta().cloned(),
        Value::SortedMap(value) => value.meta().cloned(),
        Value::Trie(value) => value.meta().cloned(),
        Value::PriorityMap(value) => value.meta().cloned(),
        Value::Set(value) => value.meta().cloned(),
        Value::OrderedSet(value) => value.meta().cloned(),
        Value::SortedSet(value) => value.meta().cloned(),
        Value::Seq(value) => value.meta().cloned(),
        Value::Var(value) => value.hara_metadata(),
        Value::Function(value) => value.metadata.clone(),
        Value::Struct(value) => value.metadata.clone(),
        Value::Mutable(value) => value.metadata.clone(),
        Value::NativeType(value) => value.metadata.clone(),
        _ => None,
    }
}

fn protocol_meta(arguments: &[Value]) -> Result<Value, String> {
    if arguments.len() != 1 {
        return Err("IObjType/meta expects one argument".into());
    }
    match value_metadata(&arguments[0]) {
        None => Ok(Value::Nil),
        Some(metadata) => metadata_to_value(&MetadataValue::Map(metadata.entries().to_vec())),
    }
}

fn protocol_with_meta(arguments: &[Value]) -> Result<Value, String> {
    if arguments.len() != 2 {
        return Err("IObjType/with-meta expects a value and metadata map".into());
    }
    let metadata = match &arguments[1] {
        Value::Nil => None,
        value => {
            let MetadataValue::Map(entries) = value_to_metadata(value)? else {
                return Err("IObjType/with-meta expects a metadata map or nil".into());
            };
            Some(Metadata::new(entries))
        }
    };
    attach_optional_metadata(arguments[0].clone(), metadata)
}

fn protocol_count(arguments: &[Value]) -> Result<Value, String> {
    if arguments.len() == 1 {
        collection_count(&arguments[0])
            .map_err(|error| format!("protocol/unsupported-receiver: {error}"))
    } else {
        Err("ICount/count expects one argument".into())
    }
}

fn protocol_nth(arguments: &[Value]) -> Result<Value, String> {
    if arguments.len() != 2 {
        return Err("INth/nth expects a collection and index".into());
    }
    if let Value::Bytes(bytes) = &arguments[0] {
        let index = value_index(&arguments[1])?;
        return bytes
            .get(index)
            .map(|byte| Value::Number(*byte as i8 as i64))
            .ok_or_else(|| "nth index out of bounds".into());
    }
    if let Value::ByteBuffer(bytes) = &arguments[0] {
        let index = value_index(&arguments[1])?;
        return bytes
            .borrow()
            .get(index)
            .map(|byte| Value::Number(*byte as i8 as i64))
            .ok_or_else(|| "nth index out of bounds".into());
    }
    collection_nth(&arguments[0], &arguments[1])
}

fn namespaced_parts(value: &Value) -> Option<(String, Option<String>)> {
    match value {
        Value::Keyword(value) => Some((
            value.get_name().to_owned(),
            value.get_namespace().map(str::to_owned),
        )),
        Value::Symbol(value) => Some((
            value.get_name().to_owned(),
            value.get_namespace().map(str::to_owned),
        )),
        Value::Var(value) => Some((
            value.get_name().to_owned(),
            value.get_namespace().map(str::to_owned),
        )),
        Value::NativeType(value) => value
            .name
            .rsplit_once('.')
            .map(|(namespace, name)| (name.to_owned(), Some(namespace.to_owned()))),
        _ => None,
    }
}

fn protocol_namespaced_name(arguments: &[Value]) -> Result<Value, String> {
    if arguments.len() != 1 {
        return Err("INamespaced/name expects one value".into());
    }
    namespaced_parts(&arguments[0])
        .map(|(name, _)| Value::String(name))
        .ok_or_else(|| "INamespaced/name has no implementation for this value".into())
}

fn protocol_namespaced_namespace(arguments: &[Value]) -> Result<Value, String> {
    if arguments.len() != 1 {
        return Err("INamespaced/namespace expects one value".into());
    }
    namespaced_parts(&arguments[0])
        .map(|(_, namespace)| namespace.map(Value::String).unwrap_or(Value::Nil))
        .ok_or_else(|| "INamespaced/namespace has no implementation for this value".into())
}

fn protocol_string_like_to_string(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::String(value)] => Ok(Value::String(value.clone())),
        [Value::Keyword(value)] => Ok(Value::String(value.as_str().into())),
        [Value::Symbol(value)] => Ok(Value::String(value.as_str().into())),
        [Value::Bytes(value)] => String::from_utf8(value.clone())
            .map(Value::String)
            .map_err(|error| format!("IStringLike/to-string expects UTF-8 bytes: {error}")),
        [_] => Err("IStringLike/to-string expects a string-like value".into()),
        _ => Err("IStringLike/to-string expects one argument".into()),
    }
}

fn protocol_string_like_from_string(arguments: &[Value]) -> Result<Value, String> {
    let [sample, Value::String(text)] = arguments else {
        return Err("IStringLike/from-string expects a sample and string".into());
    };
    match sample {
        Value::String(_) => Ok(Value::String(text.clone())),
        Value::Keyword(_) => Keyword::parse(text).map(Value::Keyword),
        Value::Symbol(_) => Ok(Value::Symbol(Symbol::parse(text))),
        Value::Bytes(_) => Ok(Value::Bytes(text.as_bytes().to_vec())),
        _ => Err("IStringLike/from-string expects a string-like sample".into()),
    }
}

fn protocol_lookup(arguments: &[Value]) -> Result<Value, String> {
    if arguments.len() == 2 || arguments.len() == 3 {
        collection_get(
            &arguments[0],
            &arguments[1],
            arguments.get(2).cloned().unwrap_or(Value::Nil),
        )
    } else {
        Err("ILookup/lookup expects a collection, key, and optional default".into())
    }
}

fn protocol_pointer_context(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Pointer(pointer)] => Ok(Value::Keyword(pointer.context().clone())),
        _ => Err("IPointer/ptr-context expects one pointer".into()),
    }
}

fn pointer_default(pointer: &PPointer) -> Result<Value, String> {
    let resolver = vm_resolve_global("std.context.space/space:rt-current")?.deref_value();
    call_value(resolver, vec![Value::Keyword(pointer.context().clone())])
        .map_err(|error| format!("pointer/runtime-unavailable: {error}"))
}

fn pointer_context_call(
    pointer: &PPointer,
    runtime: Value,
    operation: &str,
    arguments: &[Value],
) -> Result<Value, String> {
    let mut call = vec![
        runtime,
        Value::Keyword(Keyword::from(operation)),
        Value::Pointer(pointer.clone()),
    ];
    call.extend_from_slice(arguments);
    protocol_call("std.protocol.icontext/IContext", "call", &call)
}

fn protocol_apply_default(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Pointer(pointer)] => pointer_default(pointer),
        _ => Err("IApplicable/apply-default expects one pointer".into()),
    }
}

fn linear_arguments(value: &Value) -> Result<Vec<Value>, String> {
    match value {
        Value::Vector(values) => Ok(values.iter().cloned().collect()),
        Value::Tuple(values) => Ok(values.iter().cloned().collect()),
        Value::List(values) => Ok(values.iter().cloned().collect()),
        _ => Err("pointer invocation arguments must be sequential".into()),
    }
}

fn protocol_apply_in(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Pointer(pointer), runtime, values] => pointer_context_call(
            pointer,
            runtime.clone(),
            "pointer/invoke",
            &linear_arguments(values)?,
        ),
        _ => Err("IApplicable/apply-in expects a pointer, runtime, and arguments".into()),
    }
}

fn protocol_transform_in(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Pointer(_), _, values] => Ok(values.clone()),
        _ => Err("IApplicable/transform-in expects a pointer, runtime, and arguments".into()),
    }
}

fn protocol_transform_out(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Pointer(_), _, _, value] => Ok(value.clone()),
        _ => {
            Err("IApplicable/transform-out expects a pointer, runtime, arguments, and value".into())
        }
    }
}

fn protocol_invoke_in(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Pointer(pointer), runtime, rest @ ..] => {
            pointer_context_call(pointer, runtime.clone(), "pointer/invoke", rest)
        }
        _ => Err("IInvokeIn/invoke-in expects a pointer and runtime".into()),
    }
}

fn protocol_assoc(arguments: &[Value]) -> Result<Value, String> {
    if arguments.len() == 3 {
        collection_assoc(&arguments[0], &arguments[1], arguments[2].clone())
    } else {
        Err("IAssoc/assoc expects a collection, key, and value".into())
    }
}

fn protocol_dissoc(arguments: &[Value]) -> Result<Value, String> {
    if arguments.len() == 2 {
        collection_dissoc(&arguments[0], &[arguments[1].clone()])
    } else {
        Err("IDissoc/dissoc expects a collection and key".into())
    }
}

fn pair_parts(value: &Value) -> Option<(Value, Value)> {
    match value {
        Value::Tuple(values) if values.len() == 2 => Some((
            values.get(0).unwrap().clone(),
            values.get(1).unwrap().clone(),
        )),
        Value::Vector(values) if values.len() == 2 => Some((
            values.get(0).unwrap().clone(),
            values.get(1).unwrap().clone(),
        )),
        Value::List(values) if values.len() == 2 => Some((
            values.get(0).unwrap().clone(),
            values.get(1).unwrap().clone(),
        )),
        _ => None,
    }
}

fn pair_value(key: Value, value: Value) -> Value {
    Value::Tuple(Box::new(PTuple::Tup2([key, value])))
}

fn indexed_find(value: Option<&Value>, index: usize) -> Result<Value, String> {
    Ok(value
        .map(|value| pair_value(Value::Number(index as i64), value.clone()))
        .unwrap_or(Value::Nil))
}

fn protocol_find(arguments: &[Value]) -> Result<Value, String> {
    if arguments.len() != 2 {
        return Err("IFind/find expects a collection and key".into());
    }
    let collection = &arguments[0];
    let key = &arguments[1];
    match collection {
        Value::Extension(receiver) => {
            extension_protocol_call(receiver, "std.protocol.ifind/IFind", "find", arguments)
        }
        value @ (Value::Map(_)
        | Value::OrderedMap(_)
        | Value::SortedMap(_)
        | Value::Trie(_)
        | Value::PriorityMap(_)) => Ok(map_entries(value)
            .unwrap()
            .into_iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(candidate, value)| pair_value(candidate, value))
            .unwrap_or(Value::Nil)),
        Value::Pointer(pointer) => Ok(pointer
            .fields()
            .iter()
            .find(|(candidate, _)| *candidate == key)
            .map(|(candidate, value)| pair_value(candidate.clone(), value.clone()))
            .unwrap_or(Value::Nil)),
        Value::Object(values) => {
            let key = match key {
                Value::String(value) => value.as_str(),
                Value::Keyword(value) => value.as_str(),
                _ => return Err("IFind/find object expects a string or keyword key".into()),
            };
            Ok(values
                .borrow()
                .iter()
                .find(|(candidate, _)| candidate == key)
                .map(|(candidate, value)| {
                    pair_value(Value::String(candidate.clone()), value.clone())
                })
                .unwrap_or(Value::Nil))
        }
        Value::Struct(value) => Ok(named_field_name(key)
            .and_then(|name| value.get(name).cloned().map(|item| (name, item)))
            .map(|(name, item)| pair_value(named_field_key(name), item))
            .unwrap_or(Value::Nil)),
        Value::Mutable(value) => Ok(named_field_name(key)
            .and_then(|name| value.get(name).map(|item| (name, item)))
            .map(|(name, item)| pair_value(named_field_key(name), item))
            .unwrap_or(Value::Nil)),
        Value::MutableCollection(collection) => {
            let borrowed = collection.borrow();
            let mutable = borrowed
                .as_ref()
                .ok_or_else(|| "mutable collection used after to-persistent".to_string())?;
            match mutable {
                MutableCollection::Map(values) => Ok(values
                    .find_entry(key)
                    .map(|(candidate, value)| pair_value(candidate.clone(), value.clone()))
                    .unwrap_or(Value::Nil)),
                MutableCollection::OrderedMap(values) => Ok(values
                    .find_entry(key)
                    .map(|(candidate, value)| pair_value(candidate.clone(), value.clone()))
                    .unwrap_or(Value::Nil)),
                MutableCollection::SortedMap(values) => Ok(values
                    .find_entry(key)
                    .map(|(candidate, value)| pair_value(candidate.clone(), value.clone()))
                    .unwrap_or(Value::Nil)),
                MutableCollection::Trie(values) => {
                    let key = marker_key(key, "IFind/find trie")?;
                    Ok(values
                        .get(&key)
                        .map(|value| pair_value(Value::String(key), value.clone()))
                        .unwrap_or(Value::Nil))
                }
                MutableCollection::Set(values) => {
                    Ok(values.get(key).cloned().unwrap_or(Value::Nil))
                }
                MutableCollection::OrderedSet(values) => {
                    Ok(values.get(key).cloned().unwrap_or(Value::Nil))
                }
                MutableCollection::SortedSet(values) => {
                    Ok(values.get(key).cloned().unwrap_or(Value::Nil))
                }
                MutableCollection::List(values) => {
                    let index = value_index(key)?;
                    indexed_find(values.get(index), index)
                }
                MutableCollection::Queue(values) => {
                    let index = value_index(key)?;
                    indexed_find(values.get(index), index)
                }
                MutableCollection::Vector(values) => {
                    let index = value_index(key)?;
                    indexed_find(values.get(index), index)
                }
            }
        }
        value @ (Value::Set(_) | Value::OrderedSet(_) | Value::SortedSet(_)) => {
            Ok(set_find(value, key).unwrap_or(Value::Nil))
        }
        Value::Tuple(values) => indexed_find(values.get(value_index(key)?), value_index(key)?),
        Value::Vector(values) => indexed_find(values.get(value_index(key)?), value_index(key)?),
        Value::List(values) => indexed_find(values.get(value_index(key)?), value_index(key)?),
        Value::Cons(values) => {
            let index = value_index(key)?;
            indexed_find(values.iter().nth(index).as_ref(), index)
        }
        Value::Queue(values) => indexed_find(values.get(value_index(key)?), value_index(key)?),
        Value::Deque(values) => indexed_find(values.get(value_index(key)?), value_index(key)?),
        _ => Err("IFind/find has no implementation for this value".into()),
    }
}

fn protocol_iter(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Extension(receiver)] => {
            extension_protocol_call(receiver, "std.protocol.iiter/IIter", "iter", arguments)
        }
        [value]
            if matches!(
                value,
                Value::Iterator(_)
                    | Value::Nil
                    | Value::String(_)
                    | Value::Bytes(_)
                    | Value::ByteBuffer(_)
                    | Value::Array(_)
                    | Value::Object(_)
                    | Value::Struct(_)
                    | Value::Mutable(_)
                    | Value::Map(_)
                    | Value::OrderedMap(_)
                    | Value::SortedMap(_)
                    | Value::Trie(_)
                    | Value::PriorityMap(_)
                    | Value::Pointer(_)
                    | Value::Set(_)
                    | Value::OrderedSet(_)
                    | Value::SortedSet(_)
                    | Value::List(_)
                    | Value::Cons(_)
                    | Value::Queue(_)
                    | Value::Deque(_)
                    | Value::Tuple(_)
                    | Value::Vector(_)
            ) =>
        {
            make_iterator(value.clone())
        }
        _ => Err("IIter/iter expects one value".into()),
    }
}

fn protocol_deref(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Atom(atom)] => Ok(atom.deref_value()),
        [Value::Var(var)] => Ok(var.deref_value()),
        [Value::Promise(promise)] => promise_value_result(promise),
        [Value::Result(result)] => result.deref_value(),
        [Value::Pointer(pointer)] => {
            pointer_context_call(pointer, pointer_default(pointer)?, "pointer/deref", &[])
        }
        [Value::Schema(schema)] => {
            form_to_value(&crate::lang::protocol::IDeref::deref(&schema.ast))
        }
        _ => Err("IDeref/deref has no implementation for this value".into()),
    }
}

fn protocol_deref_timeout(arguments: &[Value]) -> Result<Value, String> {
    let [target, milliseconds, timeout] = arguments else {
        return Err("IDerefTimeout/deref-timeout expects three arguments".into());
    };
    let milliseconds = value_u64_integer(milliseconds, "IDerefTimeout/deref-timeout")
        .map_err(|_| "IDerefTimeout/deref-timeout expects non-negative milliseconds".to_string())?;
    match target {
        Value::Promise(promise) => {
            match promise.wait_state_timeout(std::time::Duration::from_millis(milliseconds)) {
                PromiseState::Fulfilled(value) => Ok(value),
                PromiseState::Rejected(error) => Err(promise_rejection_error(error)),
                PromiseState::Pending => Ok(timeout.clone()),
            }
        }
        Value::Atom(atom) => Ok(atom.deref_value()),
        Value::Var(var) => Ok(var.deref_value()),
        _ => Err(
            "IDerefTimeout/deref-timeout expects a dereferenceable value, milliseconds, and timeout value"
                .into(),
        ),
    }
}

fn protocol_reset(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Atom(atom), value] => atom.reset(value.clone()),
        _ => Err("IReset/reset expects an atom and value".into()),
    }
}

fn protocol_cas(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Atom(atom), old_value, new_value] => Ok(Value::Bool(
            atom.compare_and_set(old_value, new_value.clone())?,
        )),
        _ => Err("ICas/cas expects an atom, old value, and new value".into()),
    }
}

const REDUCED_TAG_NAMESPACE: &str = "hara.internal";
const REDUCED_TAG_NAME: &str = "reduced";

fn reduced_value(value: Value) -> Value {
    Value::Tagged(Box::new(PTaggedLiteral::new(
        Symbol::create(Some(REDUCED_TAG_NAMESPACE), REDUCED_TAG_NAME),
        value,
    )))
}

fn reduced_value_ref(value: &Value) -> Option<&Value> {
    match value {
        Value::Tagged(tagged)
            if tagged.tag().get_namespace() == Some(REDUCED_TAG_NAMESPACE)
                && tagged.tag().get_name() == REDUCED_TAG_NAME =>
        {
            Some(tagged.form())
        }
        _ => None,
    }
}

fn is_reduced_value(value: &Value) -> bool {
    reduced_value_ref(value).is_some()
}

fn unreduced_value(value: Value) -> Value {
    match value {
        Value::Tagged(tagged)
            if tagged.tag().get_namespace() == Some(REDUCED_TAG_NAMESPACE)
                && tagged.tag().get_name() == REDUCED_TAG_NAME =>
        {
            tagged.into_form()
        }
        value => value,
    }
}

fn reduce_iterator(
    function: &Rc<Function>,
    initial: Option<Value>,
    source: Value,
    operation: &str,
) -> Result<Value, String> {
    let iterator = make_iterator(source)?;
    let result = (|| {
        let mut accumulator = initial;
        while let Some(value) = iterator_try_next(&iterator)? {
            let next = match accumulator {
                Some(current) => call_function(function, vec![current, value])?,
                None => value,
            };
            if is_reduced_value(&next) {
                return Ok(unreduced_value(next));
            }
            accumulator = Some(next);
        }
        accumulator.ok_or_else(|| format!("{operation} cannot reduce an empty value without init"))
    })();
    let close = iterator_close(&iterator);
    match result {
        Err(error) => {
            let _ = close;
            Err(error)
        }
        Ok(value) => {
            close?;
            Ok(value)
        }
    }
}

fn schema_kind(schema: &crate::kernel::SchemaType) -> &'static str {
    use crate::kernel::SchemaType::*;
    match schema {
        Primitive(_) => "primitive",
        Reference(_) => "reference",
        Union(_) => "union",
        Vector(_) => "vector",
        Set(_) => "set",
        Tuple(_) => "tuple",
        Map(_) => "map",
        WithProperties { schema, .. } => schema_kind(schema),
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
        Set(value) => schema_ast_map(vec![
            ("kind", Form::Keyword("set".into())),
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
                            let mut entries = vec![("name", field.name.clone())];
                            if let Some(properties) = &field.properties {
                                entries.push(("properties", properties.clone()));
                            }
                            entries.push(("type", schema_ast_form(&field.value_type)));
                            schema_ast_map(entries)
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
        WithProperties { schema, properties } => {
            let Form::Map(mut entries) = schema_ast_form(schema) else {
                unreachable!("canonical schema AST must be a map");
            };
            entries.push((Form::Keyword("properties".into()), properties.clone()));
            Form::Map(entries)
        }
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

fn compile_schema_value(value: &Value, origin: Option<KernelVar<Value>>) -> Result<Value, String> {
    if let Value::Schema(schema) = value {
        return Ok(Value::Schema(schema.clone()));
    }
    if let Value::Var(var) = value {
        return compile_schema_value(&var.deref_value(), Some(var.clone()));
    }
    let form = value_to_form(value).map_err(|_| "schema expects schema data".to_string())?;
    let ast = crate::kernel::normalize_schema(&form)
        .map_err(|error| format!("invalid schema: {error}"))?;
    if matches!(ast, crate::kernel::SchemaType::Unknown(_)) {
        return Err("schema expects schema data".into());
    }
    Ok(Value::Schema(Rc::new(RuntimeSchema { form, ast, origin })))
}

fn declared_schema_contract(var: &KernelVar<Value>) -> Result<Option<Value>, String> {
    let Some(metadata) = var.hara_metadata() else {
        return Ok(None);
    };
    let Some(raw) = metadata.get_keyword("schema") else {
        return Ok(None);
    };
    let form = metadata_value_to_form(raw);
    if let Form::List(reference) = &form {
        if let [Form::Symbol(operator), Form::Symbol(target)] = reference.as_slice() {
            if operator == "var" {
                let registry = namespace_registry()?;
                let referenced = registry
                    .resolve(&Symbol::parse(target))
                    .ok_or_else(|| format!("schema Var does not exist: {target}"))?;
                return compile_schema_value(&referenced.deref_value(), Some(referenced)).map(Some);
            }
        }
    }
    let value = form_to_value(&form)?;
    compile_schema_value(&value, Some(var.clone())).map(Some)
}

fn refresh_schema_contract(var: &KernelVar<Value>) -> Result<(), String> {
    let contract = declared_schema_contract(var)?;
    var.set_schema_contract(contract);
    Ok(())
}

fn schema_contract(var: &KernelVar<Value>) -> Result<Value, String> {
    Ok(var.schema_contract().unwrap_or(Value::Nil))
}

fn native_schema_operation(
    operation: &str,
    forms: &[Form],
    env: &mut HashMap<String, Value>,
) -> Result<Value, String> {
    let method = operation
        .strip_prefix("std.native.Schema/")
        .ok_or_else(|| format!("invalid Schema operation: {operation}"))?;
    if forms.len() != 1 {
        return Err(format!("Schema/{method} expects one value"));
    }
    let value = eval(&forms[0], env)?;
    match method {
        "instance?" => Ok(Value::Bool(matches!(value, Value::Schema(_)))),
        "kind" => match value {
            Value::Schema(schema) => Ok(Value::Keyword(Keyword::from(schema_kind(&schema.ast)))),
            _ => Err("Schema/kind expects a schema".into()),
        },
        "form" => match value {
            Value::Schema(schema) => form_to_value(&schema.form),
            _ => Err("Schema/form expects a schema".into()),
        },
        "ast" => match value {
            Value::Schema(schema) => form_to_value(&schema_ast_form(&schema.ast)),
            _ => Err("Schema/ast expects a schema".into()),
        },
        "origin" => match value {
            Value::Schema(schema) => {
                Ok(schema.origin.clone().map(Value::Var).unwrap_or(Value::Nil))
            }
            _ => Err("Schema/origin expects a schema".into()),
        },
        _ => Err(format!("unknown Schema operation: {operation}")),
    }
}

fn protocol_reduce(arguments: &[Value]) -> Result<Value, String> {
    let (source, function, accumulator) = match arguments {
        [source, Value::Function(function), initial] => (source, function, Some(initial.clone())),
        [source, Value::Function(function)] => (source, function, None),
        _ => {
            return Err(
                "IReduce/reduce expects a value, function, and optional initial value".into(),
            )
        }
    };
    reduce_iterator(function, accumulator, source.clone(), "IReduce/reduce")
}

fn native_base_values(operation: &str, values: &[Value]) -> Result<Value, String> {
    let operation = operation
        .strip_prefix("std.native.Base/")
        .or_else(|| operation.strip_prefix("Base/"))
        .unwrap_or(operation);
    match operation {
        "list" => Ok(Value::List(values.to_vec().into())),
        "vector" => Ok(Value::Vector(values.to_vec().into())),
        "vec" => match values {
            [value] => Ok(Value::Vector(PVector::from_iter(iterator_values(
                value.clone(),
            )?))),
            _ => Err("Base/vec expects one collection".into()),
        },
        "set" => match values {
            [value] => Ok(Value::Set(
                unique_values(iterator_values(value.clone())?).into(),
            )),
            _ => Err("Base/set expects one collection".into()),
        },
        "pair" => match values {
            [left, right] => Ok(Value::Tuple(Box::new(PTuple::from_values(vec![
                left.clone(),
                right.clone(),
            ])?))),
            _ => Err("Base/pair expects two arguments".into()),
        },
        "tuple" if values.len() <= 8 => Ok(Value::Tuple(Box::new(PTuple::from_values(
            values.to_vec(),
        )?))),
        "tuple" => Err("Base/tuple expects at most 8 arguments".into()),
        "hash-map" if values.len() % 2 == 0 => Ok(Value::Map(PMap::from_iter(
            values
                .chunks_exact(2)
                .map(|pair| (pair[0].clone(), pair[1].clone())),
        ))),
        "hash-map" => Err("Base/hash-map expects an even number of arguments".into()),
        "hash-set" => Ok(Value::Set(values.iter().cloned().collect())),
        "atom" => match values {
            [value] => Ok(Value::Atom(Box::new(RuntimeAtom::new(value.clone(), true)))),
            _ => Err("Base/atom expects one value".into()),
        },
        "pointer" => match values {
            [descriptor] => pointer_from_descriptor(descriptor.clone()),
            _ => Err("Base/pointer expects one descriptor map".into()),
        },
        "symbol" => match values {
            [Value::String(name)] => Ok(Value::Symbol(Symbol::parse(name))),
            [Value::String(namespace), Value::String(name)] => {
                Ok(Value::Symbol(Symbol::create(Some(namespace), name)))
            }
            _ => Err("Base/symbol expects a name or namespace and name".into()),
        },
        "keyword" => match values {
            [Value::String(name)] => Keyword::parse(name)
                .map(Value::Keyword)
                .map_err(|error| format!("Base/keyword failed: {error}")),
            [Value::String(namespace), Value::String(name)] => {
                Keyword::create(Some(namespace), name)
                    .map(Value::Keyword)
                    .map_err(|error| format!("Base/keyword failed: {error}"))
            }
            _ => Err("Base/keyword expects a name or namespace and name".into()),
        },
        "reduced" => match values {
            [value] => Ok(reduced_value(value.clone())),
            _ => Err("Base/reduced expects one value".into()),
        },
        "reduced?" => match values {
            [value] => Ok(Value::Bool(is_reduced_value(value))),
            _ => Err("Base/reduced? expects one value".into()),
        },
        "unreduced" => match values {
            [value] => Ok(unreduced_value(value.clone())),
            _ => Err("Base/unreduced expects one value".into()),
        },
        "satisfies?" => match values {
            [Value::Protocol(protocol), value] => {
                Ok(Value::Bool(protocol_satisfies(protocol, value)))
            }
            _ => Err("Base/satisfies? expects a protocol and value".into()),
        },
        "type" => match values {
            [value] => Ok(Value::Keyword(portable_type_keyword(value)?)),
            _ => Err("Base/type expects one value".into()),
        },
        "instance?" => match values {
            [Value::StructType(_), value] | [Value::MutableType(_), value] => {
                named_instance_of(&values[0], value)
            }
            [Value::NativeType(native), value]
                if native.methods.iter().any(|method| method == "instance?") =>
            {
                Ok(Value::Bool(
                    portable_type_keyword(value)?.as_str() == format!("std.native.{}", native.name),
                ))
            }
            [Value::NativeType(_), _] => {
                Err("Base/instance? descriptor does not define instance?".into())
            }
            _ => Err("Base/instance? expects a type descriptor and value".into()),
        },
        "schema" => match values {
            [value] => compile_schema_value(value, None),
            _ => Err("Base/schema expects one value".into()),
        },
        "schema-of" => match values {
            [Value::Var(var)] => schema_contract(var),
            [value] => Err(format!(
                "Base/schema-of expects a Var, received {}",
                portable_type_name(value)
            )),
            _ => Err("Base/schema-of expects one Var".into()),
        },
        predicate if predicate.ends_with('?') => match values {
            [value] => Ok(Value::Bool(match predicate {
                "nil?" => matches!(value, Value::Nil),
                "not-nil?" => !matches!(value, Value::Nil),
                "boolean?" => matches!(value, Value::Bool(_)),
                "false?" => matches!(value, Value::Bool(false)),
                "true?" => matches!(value, Value::Bool(true)),
                "string?" => matches!(value, Value::String(_)),
                "char?" => matches!(value, Value::Character(_)),
                "number?" => numeric::is_numeric_value(value),
                "integer?" => numeric::is_integer_value(value),
                "decimal?" => matches!(value, Value::Decimal(_)),
                "long?" => numeric::to_i64_exact(value).is_ok(),
                "double?" => matches!(value, Value::Float(_)),
                "keyword?" => matches!(value, Value::Keyword(_)),
                "symbol?" => matches!(value, Value::Symbol(_)),
                "pointer?" => matches!(value, Value::Pointer(_)),
                "atom?" => matches!(value, Value::Atom(_)),
                "fn?" => matches!(value, Value::Function(_)),
                "bytes?" => matches!(value, Value::Bytes(_) | Value::ByteBuffer(_)),
                "array?" => matches!(value, Value::Array(_)),
                "object?" => matches!(value, Value::Object(_)),
                "list?" => matches!(value, Value::List(_)),
                "cons?" => matches!(value, Value::Cons(_)),
                "pair?" => matches!(value, Value::Tuple(tuple) if tuple.len() == 2),
                "vector?" => matches!(value, Value::Vector(_) | Value::Tuple(_)),
                "tuple?" => matches!(value, Value::Tuple(_)),
                "map?" => matches!(
                    value,
                    Value::Map(_)
                        | Value::OrderedMap(_)
                        | Value::SortedMap(_)
                        | Value::Trie(_)
                        | Value::PriorityMap(_)
                ),
                "map-entry?" => pair_parts(value).is_some(),
                "set?" => matches!(
                    value,
                    Value::Set(_) | Value::OrderedSet(_) | Value::SortedSet(_)
                ),
                "sequential?" => matches!(
                    value,
                    Value::List(_)
                        | Value::Cons(_)
                        | Value::Queue(_)
                        | Value::Deque(_)
                        | Value::Vector(_)
                        | Value::Tuple(_)
                ),
                _ => return Err(format!("unknown Base predicate: {predicate}")),
            })),
            _ => Err(format!("Base/{predicate} expects one value")),
        },
        _ => Err(format!("unknown Base operation: {operation}")),
    }
}

fn native_algo_operation(
    operation: &str,
    forms: &[Form],
    env: &mut HashMap<String, Value>,
) -> Result<Value, String> {
    let method = operation
        .strip_prefix("std.native.Algo/")
        .ok_or_else(|| format!("invalid Algo operation: {operation}"))?;
    if let Some(family) = method.strip_suffix('?') {
        if forms.len() != 1 {
            return Err(format!("Algo/{method} expects one value"));
        }
        let value = eval(&forms[0], env)?;
        return Ok(Value::Bool(match family {
            "deque" => matches!(value, Value::Deque(_)),
            "ordered-map" => matches!(value, Value::OrderedMap(_)),
            "ordered-set" => matches!(value, Value::OrderedSet(_)),
            "priority-map" => matches!(value, Value::PriorityMap(_)),
            "queue" => matches!(value, Value::Queue(_)),
            "sorted-map" => matches!(value, Value::SortedMap(_)),
            "sorted-set" => matches!(value, Value::SortedSet(_)),
            "trie" => matches!(value, Value::Trie(_)),
            _ => return Err(format!("unknown Algo predicate: {method}")),
        }));
    }
    match method {
        "deque" | "ordered-map" | "ordered-set" | "priority-map" | "queue" | "sorted-map"
        | "sorted-set" | "trie" => eval_collection_constructor(method, forms, env),
        _ => Err(format!("unknown Algo operation: {operation}")),
    }
}

fn protocol_promise_state(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Promise(promise)] => Ok(promise_state_value(promise)),
        _ => Err("IPromise/state expects a promise".into()),
    }
}

fn protocol_promise_value(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Promise(promise)] => promise_value_result(promise),
        _ => Err("IPromise/value expects a promise".into()),
    }
}

fn protocol_promise_chain(operation: &str, arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Promise(promise), Value::Function(function)] => Ok(Value::Promise(promise_chain(
            promise.clone(),
            operation,
            function.clone(),
        ))),
        _ => Err(format!(
            "IPromise/{operation} expects a promise and function"
        )),
    }
}

fn protocol_promise_cancel(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Promise(promise)] => {
            promise.cancel();
            Ok(Value::Promise(promise.clone()))
        }
        _ => Err("IPromise/cancel expects a promise".into()),
    }
}

fn protocol_coroutine_status(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Coroutine(coroutine)] => Ok(coroutine_status(coroutine)),
        _ => Err("ICoroutine/status expects a coroutine".into()),
    }
}

fn protocol_coroutine_resume(arguments: &[Value]) -> Result<Value, String> {
    let Some(Value::Coroutine(coroutine)) = arguments.first() else {
        return Err("ICoroutine/resume expects a coroutine".into());
    };
    fiber::coroutine::resume_sync(coroutine.clone(), arguments[1..].to_vec())
}

fn protocol_watch_add(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Atom(atom), key, Value::Function(function)] => {
            atom.add_watch(key.clone(), function.clone())?;
            Ok(Value::Atom(atom.clone()))
        }
        _ => Err("IWatch/watch-add expects an atom, key, and function".into()),
    }
}

fn protocol_watch_remove(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Atom(atom), key] => {
            atom.remove_watch(key)?;
            Ok(Value::Atom(atom.clone()))
        }
        _ => Err("IWatch/watch-remove expects an atom and key".into()),
    }
}

fn protocol_watch_list(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::Atom(atom)] => Ok(iterator_from_values(atom.watch_entries()?)),
        _ => Err("IWatch/watch-list expects an atom".into()),
    }
}

fn protocol_empty(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [value] => collection_empty_value(value.clone()),
        _ => Err("IEmpty/empty expects one collection".into()),
    }
}

fn protocol_equality(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [left, right] => Ok(Value::Bool(left == right)),
        _ => Err("IEquality/equality expects two values".into()),
    }
}

fn protocol_display(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [value] => Ok(Value::String(value.display())),
        _ => Err("IDisplay/display expects one value".into()),
    }
}

fn protocol_encode_with(arguments: &[Value]) -> Result<Value, String> {
    let [value, visitor] = arguments else {
        return Err("IEncodable/encode-with expects a value and visitor".into());
    };
    let (method, visitor_arguments) = match value {
        Value::Nil => ("visit-nil", vec![visitor.clone()]),
        Value::Bool(_) => ("visit-boolean", vec![visitor.clone(), value.clone()]),
        Value::Number(_) | Value::Float(_) | Value::BigInteger(_) | Value::Decimal(_) => {
            ("visit-number", vec![visitor.clone(), value.clone()])
        }
        Value::Character(_) => ("visit-character", vec![visitor.clone(), value.clone()]),
        Value::String(_) => ("visit-string", vec![visitor.clone(), value.clone()]),
        Value::Keyword(_) => ("visit-keyword", vec![visitor.clone(), value.clone()]),
        Value::Symbol(_) => ("visit-symbol", vec![visitor.clone(), value.clone()]),
        Value::List(_) | Value::Cons(_) | Value::Queue(_) | Value::Deque(_) => {
            ("visit-seq", vec![visitor.clone(), value.clone()])
        }
        Value::Vector(_) | Value::Tuple(_) => {
            ("visit-vector", vec![visitor.clone(), value.clone()])
        }
        Value::Map(_)
        | Value::OrderedMap(_)
        | Value::SortedMap(_)
        | Value::Trie(_)
        | Value::PriorityMap(_) => ("visit-map", vec![visitor.clone(), value.clone()]),
        Value::Set(_) | Value::OrderedSet(_) | Value::SortedSet(_) => {
            ("visit-set", vec![visitor.clone(), value.clone()])
        }
        Value::Tagged(tagged) => (
            "visit-tagged",
            vec![
                visitor.clone(),
                Value::Symbol(tagged.tag().clone()),
                tagged.form().clone(),
            ],
        ),
        _ => ("visit-unknown", vec![visitor.clone(), value.clone()]),
    };
    protocol_call(
        "std.protocol.iencodevisitor/IEncodeVisitor",
        method,
        &visitor_arguments,
    )
}

fn protocol_hash(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [value] => Ok(Value::Number(value.stable_hash() as i64)),
        _ => Err("IHash/hash expects one value".into()),
    }
}

fn protocol_invoke(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [callable, rest @ ..] => call_value(callable.clone(), rest.to_vec()),
        _ => Err("IFn/invoke expects a callable receiver".into()),
    }
}

fn protocol_pair_key(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [value] => pair_parts(value)
            .map(|(key, _)| key)
            .ok_or_else(|| "IPair/key has no implementation for this value".into()),
        _ => Err("IPair/key expects one pair".into()),
    }
}

fn protocol_pair_value(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [value] => pair_parts(value)
            .map(|(_, value)| value)
            .ok_or_else(|| "IPair/value has no implementation for this value".into()),
        _ => Err("IPair/value expects one pair".into()),
    }
}

fn protocol_peek_first(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [value] if builtin_protocol_satisfies("IPeekFirst", value) => {
            collection_first(value.clone())
        }
        [_] => Err("protocol/unsupported-receiver: IPeekFirst/peek-first".into()),
        _ => Err("IPeekFirst/peek-first expects one collection".into()),
    }
}

fn protocol_peek_last(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [value] if builtin_protocol_satisfies("IPeekLast", value) => collection_last(value.clone()),
        [_] => Err("protocol/unsupported-receiver: IPeekLast/peek-last".into()),
        _ => Err("IPeekLast/peek-last expects one collection".into()),
    }
}

fn protocol_pop_first(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::MutableCollection(collection)] => {
            let mut borrowed = collection.borrow_mut();
            let mutable = borrowed
                .as_mut()
                .ok_or_else(|| "mutable collection used after to-persistent".to_string())?;
            match mutable {
                MutableCollection::List(values) => {
                    values.pop_first();
                }
                MutableCollection::Queue(values) => {
                    values.pop_first();
                }
                _ => return Err("protocol/unsupported-receiver: IPopFirst/pop-first".into()),
            }
            Ok(Value::MutableCollection(collection.clone()))
        }
        [Value::List(values)] => Ok(Value::List(values.pop_first())),
        [Value::Cons(values)] => Ok(Value::List(values.clone().pop_first())),
        [Value::Tuple(values)] => Ok(Value::Tuple(Box::new(values.pop_first()))),
        [Value::Queue(values)] => Ok(Value::Queue(Box::new(values.pop_first()))),
        [Value::Deque(values)] => Ok(Value::Deque(Box::new(values.pop_first()))),
        [Value::PriorityMap(values)] => Ok(Value::PriorityMap(Box::new(values.pop_first()))),
        [value @ Value::Seq(_)] => collection_rest(value.clone()),
        [_] => Err("protocol/unsupported-receiver: IPopFirst/pop-first".into()),
        _ => Err("IPopFirst/pop-first expects one collection".into()),
    }
}

fn protocol_pop_last(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::MutableCollection(collection)] => {
            let mut borrowed = collection.borrow_mut();
            let mutable = borrowed
                .as_mut()
                .ok_or_else(|| "mutable collection used after to-persistent".to_string())?;
            match mutable {
                MutableCollection::List(values) => {
                    values.pop_last();
                }
                MutableCollection::Queue(values) => {
                    values.pop_last();
                }
                MutableCollection::Vector(values) => {
                    values.pop_last();
                }
                _ => return Err("protocol/unsupported-receiver: IPopLast/pop-last".into()),
            }
            Ok(Value::MutableCollection(collection.clone()))
        }
        [Value::List(values)] => Ok(Value::List(values.pop_last())),
        [Value::Tuple(values)] => Ok(Value::Tuple(Box::new(values.pop_last()))),
        [Value::Vector(values)] => Ok(Value::Vector(values.pop_last())),
        [Value::Queue(values)] => Ok(Value::Queue(Box::new(values.pop_last()))),
        [Value::Deque(values)] => Ok(Value::Deque(Box::new(values.pop_last()))),
        [Value::PriorityMap(values)] => Ok(Value::PriorityMap(Box::new(values.pop_last()))),
        [_] => Err("protocol/unsupported-receiver: IPopLast/pop-last".into()),
        _ => Err("IPopLast/pop-last expects one collection".into()),
    }
}

fn protocol_push_first(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::MutableCollection(collection), value] => {
            let mut borrowed = collection.borrow_mut();
            let mutable = borrowed
                .as_mut()
                .ok_or_else(|| "mutable collection used after to-persistent".to_string())?;
            match mutable {
                MutableCollection::List(values) => {
                    values.push_first(value.clone());
                }
                MutableCollection::Queue(values) => {
                    values.push_first(value.clone());
                }
                _ => return Err("protocol/unsupported-receiver: IPushFirst/push-first".into()),
            }
            Ok(Value::MutableCollection(collection.clone()))
        }
        [Value::List(values), value] => Ok(Value::List(values.push_first(value.clone()))),
        [Value::Cons(values), value] => Ok(Value::Cons(Box::new(
            PCons::new(value.clone(), values.to_list()).with_meta(values.meta().cloned()),
        ))),
        [Value::Tuple(values), value] => tuple_push_first(values, value.clone()),
        [Value::Deque(values), value] => {
            Ok(Value::Deque(Box::new(values.push_first(value.clone()))))
        }
        [Value::Queue(values), value] => {
            Ok(Value::Queue(Box::new(values.push_first(value.clone()))))
        }
        [_, _] => Err("protocol/unsupported-receiver: IPushFirst/push-first".into()),
        _ => Err("IPushFirst/push-first expects a collection and value".into()),
    }
}

fn protocol_push_last(arguments: &[Value]) -> Result<Value, String> {
    match arguments {
        [Value::MutableCollection(collection), value] => {
            let mut borrowed = collection.borrow_mut();
            let mutable = borrowed
                .as_mut()
                .ok_or_else(|| "mutable collection used after to-persistent".to_string())?;
            match mutable {
                MutableCollection::List(values) => {
                    values.push_last(value.clone());
                }
                MutableCollection::Queue(values) => {
                    values.push_last(value.clone());
                }
                MutableCollection::Vector(values) => {
                    values.push_last(value.clone());
                }
                _ => return Err("protocol/unsupported-receiver: IPushLast/push-last".into()),
            }
            Ok(Value::MutableCollection(collection.clone()))
        }
        [Value::List(values), value] => Ok(Value::List(values.push_last(value.clone()))),
        [Value::Tuple(values), value] => tuple_push_last(values, value.clone()),
        [Value::Vector(values), value] => Ok(Value::Vector(values.push_last(value.clone()))),
        [Value::Queue(values), value] => {
            Ok(Value::Queue(Box::new(values.push_last(value.clone()))))
        }
        [Value::Deque(values), value] => {
            Ok(Value::Deque(Box::new(values.push_last(value.clone()))))
        }
        [_, _] => Err("protocol/unsupported-receiver: IPushLast/push-last".into()),
        _ => Err("IPushLast/push-last expects a collection and value".into()),
    }
}

fn protocol_cons(arguments: &[Value]) -> Result<Value, String> {
    let [collection, item] = arguments else {
        return Err("ICons/cons expects a collection and value".into());
    };
    match collection {
        Value::Cons(values) => Ok(Value::Cons(Box::new(
            PCons::new(item.clone(), values.iter().collect()).with_meta(values.meta().cloned()),
        ))),
        Value::Tuple(values) => Ok(Value::Cons(Box::new(PCons::new(
            item.clone(),
            values.iter().cloned().collect(),
        )))),
        Value::Vector(values) => Ok(Value::Cons(Box::new(PCons::new(
            item.clone(),
            values.iter().cloned().collect(),
        )))),
        Value::List(values) => Ok(Value::Cons(Box::new(PCons::new(
            item.clone(),
            values.clone(),
        )))),
        Value::Queue(values) => Ok(Value::Cons(Box::new(PCons::new(
            item.clone(),
            values.iter().cloned().collect(),
        )))),
        Value::Deque(values) => Ok(Value::Deque(Box::new(values.push_first(item.clone())))),
        Value::Nil => Ok(Value::Cons(Box::new(PCons::new(
            item.clone(),
            PList::new(),
        )))),
        Value::Seq(_) => iterator_seq(iterator_prepend(item.clone(), collection.clone())?),
        _ => Err("ICons/cons has no implementation for this value".into()),
    }
}

fn tuple_push_last(values: &PTuple<Value>, item: Value) -> Result<Value, String> {
    if values.len() < 8 {
        return Ok(Value::Tuple(Box::new(values.push_last(item)?)));
    }
    Ok(Value::Vector(
        PVector::from_iter(values.iter().cloned().chain(std::iter::once(item)))
            .with_meta(values.meta().cloned()),
    ))
}

fn tuple_push_first(values: &PTuple<Value>, item: Value) -> Result<Value, String> {
    if values.len() < 8 {
        return Ok(Value::Tuple(Box::new(values.push_first(item)?)));
    }
    Ok(Value::Vector(
        PVector::from_iter(std::iter::once(item).chain(values.iter().cloned()))
            .with_meta(values.meta().cloned()),
    ))
}

fn protocol_conj(arguments: &[Value]) -> Result<Value, String> {
    if arguments.len() != 2 {
        return Err("IConj/conj expects a collection and value".into());
    }
    let collection = &arguments[0];
    let item = &arguments[1];
    match collection {
        Value::Extension(receiver) => {
            extension_protocol_call(receiver, "std.protocol.iconj/IConj", "conj", arguments)
        }
        Value::MutableCollection(collection) => {
            let mut borrowed = collection.borrow_mut();
            let mutable = borrowed
                .as_mut()
                .ok_or_else(|| "mutable collection used after to-persistent".to_string())?;
            match mutable {
                MutableCollection::Set(values) => {
                    values.conj(item.clone());
                }
                MutableCollection::OrderedSet(values) => {
                    values.conj(item.clone());
                }
                MutableCollection::SortedSet(values) => {
                    values.conj(item.clone());
                }
                MutableCollection::List(values) => {
                    values.push_first(item.clone());
                }
                MutableCollection::Queue(values) => {
                    values.push_last(item.clone());
                }
                MutableCollection::Vector(values) => {
                    values.push_last(item.clone());
                }
                MutableCollection::Map(values) => {
                    let (key, value) = pair_parts(item)
                        .ok_or_else(|| "IConj/conj map expects a two-element entry".to_string())?;
                    values.assoc(key, value);
                }
                MutableCollection::OrderedMap(values) => {
                    let (key, value) = pair_parts(item)
                        .ok_or_else(|| "IConj/conj map expects a two-element entry".to_string())?;
                    values.assoc(key, value);
                }
                MutableCollection::SortedMap(values) => {
                    let (key, value) = pair_parts(item)
                        .ok_or_else(|| "IConj/conj map expects a two-element entry".to_string())?;
                    values.assoc(key, value);
                }
                MutableCollection::Trie(values) => {
                    let (key, value) = pair_parts(item)
                        .ok_or_else(|| "IConj/conj trie expects a two-element entry".to_string())?;
                    values.assoc(marker_key(&key, "trie")?, value);
                }
            }
            Ok(Value::MutableCollection(collection.clone()))
        }
        Value::Array(values) => {
            values.borrow_mut().push(item.clone());
            Ok(Value::Array(values.clone()))
        }
        Value::Object(values) => {
            let (key, value) = pair_parts(item)
                .ok_or_else(|| "IConj/conj object expects a two-element entry".to_string())?;
            let key = marker_key(&key, "IConj/conj object")?;
            let mut output = values.borrow_mut();
            if let Some((_, current)) = output.iter_mut().find(|(candidate, _)| candidate == &key) {
                *current = value;
            } else {
                output.push((key, value));
            }
            drop(output);
            Ok(Value::Object(values.clone()))
        }
        Value::Tuple(values) => tuple_push_last(values, item.clone()),
        Value::Vector(values) => {
            let output = values.push_last(item.clone());
            Ok(Value::Vector(output))
        }
        Value::Queue(values) => Ok(Value::Queue(Box::new(values.push_last(item.clone())))),
        Value::Deque(values) => Ok(Value::Deque(Box::new(values.push_last(item.clone())))),
        Value::Cons(values) => Ok(Value::Cons(Box::new(
            PCons::new(item.clone(), values.iter().collect()).with_meta(values.meta().cloned()),
        ))),
        Value::List(values) => {
            let output = std::iter::once(item.clone())
                .chain(values.iter().cloned())
                .collect();
            Ok(Value::List(output))
        }
        value @ (Value::Set(_) | Value::OrderedSet(_) | Value::SortedSet(_)) => {
            set_conj_value(value, item.clone())
        }
        value @ (Value::Map(_)
        | Value::OrderedMap(_)
        | Value::SortedMap(_)
        | Value::Trie(_)
        | Value::PriorityMap(_)) => {
            let (entry_key, entry_value) = pair_parts(item)
                .ok_or_else(|| "IConj/conj map expects a two-element entry".to_string())?;
            map_assoc_value(value, entry_key, entry_value)
        }
        _ => Err("IConj/conj expects a collection".into()),
    }
}

fn protocol_call(protocol: &str, method: &str, arguments: &[Value]) -> Result<Value, String> {
    ACTIVE_PROTOCOLS.with(|active| {
        active
            .borrow()
            .as_ref()
            .cloned()
            .unwrap_or_else(ProtocolRegistry::core)
            .invoke(protocol, method, arguments)
    })
}

fn extension_protocol_call(
    receiver: &ExtensionValue,
    protocol: &str,
    method: &str,
    arguments: &[Value],
) -> Result<Value, String> {
    ACTIVE_PROTOCOLS.with(|active| {
        active
            .borrow()
            .as_ref()
            .cloned()
            .unwrap_or_else(ProtocolRegistry::core)
            .invoke_extension(receiver, protocol, method, arguments)
    })
}

fn extension_has_category(receiver: &ExtensionValue, category: &str) -> bool {
    ACTIVE_PROTOCOLS.with(|active| {
        active
            .borrow()
            .as_ref()
            .is_some_and(|registry| registry.extension_has_category(receiver, category))
    })
}

fn mutable_linear_satisfies(value: &Value, list_or_queue: bool, vector: bool) -> bool {
    let Value::MutableCollection(collection) = value else {
        return false;
    };
    let borrowed = collection.borrow();
    let Some(collection) = borrowed.as_ref() else {
        return false;
    };
    matches!(
        collection,
        MutableCollection::List(_) | MutableCollection::Queue(_)
    ) && list_or_queue
        || matches!(collection, MutableCollection::Vector(_)) && vector
}

fn builtin_protocol_satisfies(protocol: &str, value: &Value) -> bool {
    let name = protocol.rsplit('/').next().unwrap_or(protocol);
    let persistent_collection = matches!(
        value,
        Value::Map(_)
            | Value::OrderedMap(_)
            | Value::SortedMap(_)
            | Value::Trie(_)
            | Value::PriorityMap(_)
            | Value::Set(_)
            | Value::OrderedSet(_)
            | Value::SortedSet(_)
            | Value::List(_)
            | Value::Cons(_)
            | Value::Queue(_)
            | Value::Deque(_)
            | Value::Tuple(_)
            | Value::Vector(_)
            | Value::Seq(_)
    );
    let map_like = matches!(
        value,
        Value::Map(_)
            | Value::OrderedMap(_)
            | Value::SortedMap(_)
            | Value::Trie(_)
            | Value::PriorityMap(_)
    );
    let sequential = matches!(
        value,
        Value::List(_)
            | Value::Cons(_)
            | Value::Queue(_)
            | Value::Deque(_)
            | Value::Tuple(_)
            | Value::Vector(_)
    );
    let mutable_convertible = matches!(
        value,
        Value::Map(_)
            | Value::OrderedMap(_)
            | Value::SortedMap(_)
            | Value::Trie(_)
            | Value::Set(_)
            | Value::OrderedSet(_)
            | Value::SortedSet(_)
            | Value::List(_)
            | Value::Queue(_)
            | Value::Vector(_)
    );
    let iterable = persistent_collection
        || matches!(
            value,
            Value::Iterator(_)
                | Value::Nil
                | Value::String(_)
                | Value::Bytes(_)
                | Value::ByteBuffer(_)
                | Value::Array(_)
                | Value::Object(_)
                | Value::Struct(_)
                | Value::Mutable(_)
                | Value::MutableCollection(_)
                | Value::Pointer(_)
        );
    let metadata_capable = persistent_collection
        || matches!(
            value,
            Value::Symbol(_)
                | Value::Keyword(_)
                | Value::Pointer(_)
                | Value::Var(_)
                | Value::Function(_)
                | Value::Struct(_)
                | Value::Mutable(_)
                | Value::NativeType(_)
        );
    match name {
        "IColl" => persistent_collection,
        "IConj" => {
            (persistent_collection && !matches!(value, Value::Seq(_)))
                || matches!(
                    value,
                    Value::Array(_) | Value::Object(_) | Value::MutableCollection(_)
                )
        }
        "IEmpty" => {
            persistent_collection
                || matches!(
                    value,
                    Value::Nil | Value::Array(_) | Value::Object(_) | Value::Struct(_)
                )
        }
        "IToMutable" => mutable_convertible,
        "IToPersistent" => matches!(value, Value::MutableCollection(_)),
        "IIter" | "IReduce" => iterable,
        "IPeekFirst" => {
            matches!(
                value,
                Value::List(_)
                    | Value::Cons(_)
                    | Value::Queue(_)
                    | Value::Deque(_)
                    | Value::Tuple(_)
                    | Value::Vector(_)
                    | Value::Seq(_)
                    | Value::PriorityMap(_)
            ) || mutable_linear_satisfies(value, true, true)
        }
        "IPeekLast" => {
            matches!(
                value,
                Value::List(_)
                    | Value::Queue(_)
                    | Value::Deque(_)
                    | Value::Tuple(_)
                    | Value::Vector(_)
                    | Value::PriorityMap(_)
            ) || mutable_linear_satisfies(value, true, true)
        }
        "IIterator" => matches!(value, Value::Iterator(_)),
        "ICount" => {
            persistent_collection
                || matches!(
                    value,
                    Value::String(_)
                        | Value::Bytes(_)
                        | Value::ByteBuffer(_)
                        | Value::Array(_)
                        | Value::Object(_)
                        | Value::Struct(_)
                        | Value::Mutable(_)
                        | Value::Pointer(_)
                        | Value::MutableCollection(_)
                        | Value::Iterator(_)
                        | Value::Nil
                )
        }
        "INth" => {
            sequential
                || matches!(
                    value,
                    Value::String(_) | Value::Bytes(_) | Value::ByteBuffer(_) | Value::Array(_)
                )
        }
        "IAssoc" | "IDissoc" => map_like || matches!(value, Value::Vector(_)),
        "IFind" => {
            map_like
                || matches!(
                    value,
                    Value::Set(_)
                        | Value::OrderedSet(_)
                        | Value::SortedSet(_)
                        | Value::List(_)
                        | Value::Cons(_)
                        | Value::Queue(_)
                        | Value::Deque(_)
                        | Value::Vector(_)
                        | Value::Tuple(_)
                        | Value::Pointer(_)
                        | Value::Object(_)
                        | Value::Struct(_)
                        | Value::Mutable(_)
                        | Value::MutableCollection(_)
                )
        }
        "ILookup" => {
            map_like
                || matches!(
                    value,
                    Value::Vector(_) | Value::Tuple(_) | Value::Pointer(_)
                )
        }
        "IDeref" => matches!(
            value,
            Value::Atom(_)
                | Value::Promise(_)
                | Value::Var(_)
                | Value::Pointer(_)
                | Value::Schema(_)
        ),
        "IReset" => matches!(value, Value::Atom(_) | Value::Var(_)),
        "ICas" | "IWatch" => matches!(value, Value::Atom(_)),
        "IFn" => matches!(
            value,
            Value::Function(_) | Value::StructType(_) | Value::MutableType(_) | Value::Pointer(_)
        ),
        "IPointer" | "IApplicable" | "IInvokeIn" => matches!(value, Value::Pointer(_)),
        "IPair" => pair_parts(value).is_some(),
        "IObjType" => metadata_capable,
        "IStringLike" => matches!(
            value,
            Value::String(_) | Value::Keyword(_) | Value::Symbol(_) | Value::Bytes(_)
        ),
        "IPushFirst" => {
            matches!(
                value,
                Value::List(_)
                    | Value::Cons(_)
                    | Value::Tuple(_)
                    | Value::Queue(_)
                    | Value::Deque(_)
            ) || mutable_linear_satisfies(value, true, false)
        }
        "IPushLast" => {
            matches!(
                value,
                Value::List(_)
                    | Value::Tuple(_)
                    | Value::Vector(_)
                    | Value::Queue(_)
                    | Value::Deque(_)
            ) || mutable_linear_satisfies(value, true, true)
        }
        "IPopFirst" => {
            matches!(
                value,
                Value::List(_)
                    | Value::Cons(_)
                    | Value::Tuple(_)
                    | Value::Queue(_)
                    | Value::Deque(_)
                    | Value::PriorityMap(_)
                    | Value::Seq(_)
            ) || mutable_linear_satisfies(value, true, false)
        }
        "IPopLast" => {
            matches!(
                value,
                Value::List(_)
                    | Value::Tuple(_)
                    | Value::Vector(_)
                    | Value::Queue(_)
                    | Value::Deque(_)
                    | Value::PriorityMap(_)
            ) || mutable_linear_satisfies(value, true, true)
        }
        "IMutable" => matches!(value, Value::Mutable(_) | Value::MutableCollection(_)),
        "IPersistent" => persistent_collection || matches!(value, Value::Struct(_)),
        "IStream" => matches!(value, Value::Stream(_)),
        "IClose" => matches!(
            value,
            Value::Stream(_) | Value::Coroutine(_) | Value::Iterator(_)
        ),
        _ => false,
    }
}

fn protocol_satisfies(protocol: &GuestProtocol, value: &Value) -> bool {
    ACTIVE_PROTOCOLS.with(|active| {
        active
            .borrow()
            .as_ref()
            .cloned()
            .unwrap_or_else(ProtocolRegistry::core)
            .satisfies(protocol, value)
    })
}

fn named_predicate_protocol(name: &str) -> Option<&'static str> {
    match name {
        "coll?" => Some("IColl"),
        "iterable?" => Some("IIter"),
        "iterator?" => Some("IIterator"),
        "counted?" => Some("ICount"),
        "reducible?" => Some("IReduce"),
        "indexed?" => Some("INth"),
        "associative?" => Some("IAssoc"),
        "findable?" => Some("IFind"),
        "lookupable?" => Some("ILookup"),
        "derefable?" => Some("IDeref"),
        "resettable?" => Some("IReset"),
        "casable?" => Some("ICas"),
        "watchable?" => Some("IWatch"),
        "callable?" => Some("IFn"),
        "applicable?" => Some("IApplicable"),
        "pair?" => Some("IPair"),
        "mutable?" => Some("IMutable"),
        "persistent?" => Some("IPersistent"),
        _ => None,
    }
}

fn named_protocol_satisfies(name: &str, value: &Value) -> bool {
    if name == "pair?" {
        return matches!(value, Value::Tuple(tuple) if tuple.len() == 2);
    }
    let Some(protocol_name) = named_predicate_protocol(name) else {
        return false;
    };
    if protocol_name == "IColl" {
        return builtin_protocol_satisfies(protocol_name, value);
    }
    let Some((_, methods)) = FOUNDATION_PROTOCOLS
        .iter()
        .find(|(candidate, _)| *candidate == protocol_name)
    else {
        return false;
    };
    protocol_satisfies(
        &GuestProtocol {
            name: builtin_protocol_name(protocol_name),
            methods: methods
                .iter()
                .map(|(method, arity)| ((*method).to_owned(), *arity))
                .collect(),
            parents: Vec::new(),
        },
        value,
    )
}

fn promise_value(value: &Value, operation: &str) -> Result<Promise, String> {
    match value {
        Value::Promise(promise) => Ok(promise.clone()),
        _ => Err(format!("{operation} expects a promise")),
    }
}

fn promise_state_value(promise: &Promise) -> Value {
    Value::Keyword(
        match promise.state() {
            PromiseState::Pending => "pending",
            PromiseState::Fulfilled(_) => "fulfilled",
            PromiseState::Rejected(error) if error.is_cancelled() => "cancelled",
            PromiseState::Rejected(_) => "rejected",
        }
        .into(),
    )
}

fn promise_value_result(promise: &Promise) -> Result<Value, String> {
    match promise.state() {
        PromiseState::Pending => Err("promise is pending".into()),
        PromiseState::Fulfilled(value) => Ok(value),
        PromiseState::Rejected(error) => Err(promise_rejection_error(error)),
    }
}

fn promise_from(value: Value) -> Promise {
    match value {
        Value::Promise(promise) => promise,
        value => {
            let promise = Promise::new();
            promise.resolve(value);
            promise
        }
    }
}

fn promise_all(values: Vec<Value>) -> Promise {
    let output = Promise::new();
    if values.is_empty() {
        output.resolve(Value::Array(Rc::new(RefCell::new(Vec::new()))));
        return output;
    }
    let count = values.len();
    let remaining = Rc::new(Cell::new(count));
    let results = Rc::new(RefCell::new(vec![Value::Nil; count]));
    let mut sources = Vec::with_capacity(count);
    for (index, value) in values.into_iter().enumerate() {
        let source = match value {
            Value::Promise(promise) => promise,
            value => {
                let promise = Promise::new();
                promise.resolve(value);
                promise
            }
        };
        sources.push(source.clone());
        let destination = output.clone();
        let remaining = remaining.clone();
        let results = results.clone();
        source.on_settle(Rc::new(move |state| match state {
            PromiseState::Fulfilled(value) => {
                results.borrow_mut()[index] = value;
                let left = remaining.get() - 1;
                remaining.set(left);
                if left == 0 {
                    destination.resolve(Value::Array(Rc::new(RefCell::new(
                        results.borrow().clone(),
                    ))));
                }
            }
            PromiseState::Rejected(error) => {
                destination.reject_rejection(error);
            }
            PromiseState::Pending => {}
        }));
    }
    let poll_sources = sources.clone();
    output.set_poller(Rc::new(move || {
        for source in &poll_sources {
            source.state();
        }
    }));
    output.set_waiter(Rc::new(move || {
        for source in &sources {
            source.wait_state();
        }
    }));
    output
}
fn settle_promise_result(destination: &Promise, result: Result<Value, String>) {
    match result {
        Ok(Value::Promise(source)) => {
            destination.adopt(&source);
        }
        Ok(value) => {
            destination.resolve(value);
        }
        Err(error) => {
            destination.reject(error);
        }
    }
}

fn finish_promise(destination: Promise, original: PromiseState, cleanup: Result<Value, String>) {
    let preserved_destination = destination.clone();
    let preserve = move || match original.clone() {
        PromiseState::Fulfilled(value) => {
            preserved_destination.resolve(value);
        }
        PromiseState::Rejected(error) => {
            preserved_destination.reject_rejection(error);
        }
        PromiseState::Pending => {}
    };
    match cleanup {
        Ok(Value::Promise(cleanup)) => {
            cleanup.on_settle(Rc::new(move |state| match state {
                PromiseState::Fulfilled(_) => preserve(),
                PromiseState::Rejected(error) => {
                    destination.reject_rejection(error);
                }
                PromiseState::Pending => {}
            }));
        }
        Ok(_) => preserve(),
        Err(error) => {
            destination.reject(error);
        }
    }
}

fn promise_chain(source: Promise, operation: &str, function: Rc<Function>) -> Promise {
    let output = Promise::new();
    let poll_source = source.clone();
    output.set_poller(Rc::new(move || {
        poll_source.state();
    }));
    let wait_source = source.clone();
    output.set_waiter(Rc::new(move || {
        wait_source.wait_state();
    }));
    let operation = operation.to_string();
    let destination = output.clone();
    source.on_settle(Rc::new(move |state| match state.clone() {
        PromiseState::Fulfilled(value) if operation == "promise/then" => {
            settle_promise_result(&destination, call_function(&function, vec![value]));
        }
        PromiseState::Rejected(error) if operation == "promise/catch" => {
            settle_promise_result(&destination, call_function(&function, vec![error.value()]));
        }
        PromiseState::Fulfilled(_) | PromiseState::Rejected(_)
            if operation == "promise/finally" =>
        {
            finish_promise(
                destination.clone(),
                state,
                call_function(&function, Vec::new()),
            );
        }
        PromiseState::Fulfilled(value) => {
            destination.resolve(value);
        }
        PromiseState::Rejected(error) => {
            destination.reject_rejection(error);
        }
        PromiseState::Pending => {}
    }));
    output
}
