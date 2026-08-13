use crate::core::Value;
#[cfg(test)]
use crate::lang::data::{Tuple as PTuple, Vector as PVector};

const MAGIC: &[u8; 4] = b"HTA0";
pub const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_NESTING_DEPTH: usize = 256;
const NIL: u8 = 0;
const FALSE: u8 = 1;
const TRUE: u8 = 2;
const I64: u8 = 3;
const STRING: u8 = 4;
const BYTES: u8 = 5;
const KEYWORD: u8 = 6;
const SYMBOL: u8 = 7;
const LIST: u8 = 8;
const VECTOR: u8 = 9;
const SET: u8 = 10;
const MAP: u8 = 11;
const HANDLE: u8 = 12;
const NAMESPACE: u8 = 13;
const VAR: u8 = 14;
const F64: u8 = 15;
const ATOM: u8 = 16;
const ARRAY: u8 = 17;
const OBJECT: u8 = 18;
const CHARACTER: u8 = 19;
const BIG_INTEGER: u8 = 20;
const DECIMAL: u8 = 21;
const REGEX: u8 = 22;
const TUPLE: u8 = 23;
const CONS: u8 = 24;
const QUEUE: u8 = 25;
const ORDERED_MAP: u8 = 26;
const SORTED_MAP: u8 = 27;
const TRIE: u8 = 28;
const ORDERED_SET: u8 = 29;
const SORTED_SET: u8 = 30;
const TAGGED: u8 = 31;
const EXCEPTION_INFO: u8 = 32;
const STRUCT: u8 = 33;

pub fn encode(value: &Value) -> Result<Vec<u8>, String> {
    let mut output = MAGIC.to_vec();
    encode_bare(value, &mut output, 0)?;
    if output.len() > MAX_FRAME_BYTES {
        return Err("hta/value-too-large: frame exceeds 64 MiB".into());
    }
    Ok(output)
}

pub fn decode(bytes: &[u8]) -> Result<Value, String> {
    if bytes.len() > MAX_FRAME_BYTES {
        return Err("hta/value-too-large: frame exceeds 64 MiB".into());
    }
    if !bytes.starts_with(MAGIC) {
        return Err("hta/value-malformed: invalid HTA0 header".into());
    }
    let mut reader = Reader {
        bytes,
        cursor: MAGIC.len(),
    };
    let value = reader.value(0)?;
    if reader.cursor != bytes.len() {
        return Err("hta/value-malformed: trailing bytes".into());
    }
    Ok(value)
}

fn encode_bare(value: &Value, output: &mut Vec<u8>, depth: usize) -> Result<(), String> {
    if depth > MAX_NESTING_DEPTH {
        return Err("hta/value-too-deep: nesting exceeds 256".into());
    }
    match value {
        Value::Nil => output.push(NIL),
        Value::Bool(false) => output.push(FALSE),
        Value::Bool(true) => output.push(TRUE),
        Value::Number(value) => {
            output.push(I64);
            output.extend_from_slice(&value.to_be_bytes());
        }
        Value::Float(value) => {
            output.push(F64);
            output.extend_from_slice(&value.to_bits().to_be_bytes());
        }
        Value::Character(value) => {
            output.push(CHARACTER);
            output.extend_from_slice(&u32::from(*value).to_be_bytes());
        }
        Value::BigInteger(value) => {
            output.push(BIG_INTEGER);
            encode_bytes(value.as_bytes(), output)?;
        }
        Value::Decimal(value) => {
            output.push(DECIMAL);
            encode_bytes(value.as_bytes(), output)?;
        }
        Value::Regex(value) => {
            output.push(REGEX);
            encode_bytes(value.as_bytes(), output)?;
        }
        Value::String(value) => {
            output.push(STRING);
            encode_bytes(value.as_str().as_bytes(), output)?;
        }
        Value::Bytes(value) => {
            output.push(BYTES);
            encode_bytes(value, output)?;
        }
        Value::ByteBuffer(value) => {
            output.push(BYTES);
            encode_bytes(&value.borrow(), output)?;
        }
        Value::Keyword(value) => {
            output.push(KEYWORD);
            encode_bytes(value.as_str().as_bytes(), output)?;
        }
        Value::Symbol(value) => {
            output.push(SYMBOL);
            encode_bytes(value.as_str().as_bytes(), output)?;
        }
        Value::List(values) => encode_sequence(LIST, values.iter(), output, depth)?,
        Value::Tuple(values) => encode_sequence(TUPLE, values.iter(), output, depth)?,
        Value::Vector(values) => encode_sequence(VECTOR, values.iter(), output, depth)?,
        Value::Cons(values) => encode_sequence(
            CONS,
            values.iter().collect::<Vec<_>>().iter(),
            output,
            depth,
        )?,
        Value::Queue(values) => encode_sequence(QUEUE, values.iter(), output, depth)?,
        Value::Set(values) => {
            let mut encoded = values
                .iter()
                .map(|value| bare(value, depth + 1))
                .collect::<Result<Vec<_>, _>>()?;
            encoded.sort();
            output.push(SET);
            encode_len(encoded.len(), output)?;
            for value in encoded {
                output.extend_from_slice(&value);
            }
        }
        Value::OrderedSet(values) => encode_sequence(ORDERED_SET, values.iter(), output, depth)?,
        Value::SortedSet(values) => {
            let mut encoded = values
                .iter()
                .map(|value| bare(value, depth + 1))
                .collect::<Result<Vec<_>, _>>()?;
            encoded.sort();
            output.push(SORTED_SET);
            encode_len(encoded.len(), output)?;
            for value in encoded {
                output.extend_from_slice(&value);
            }
        }
        Value::OrderedMap(values) => encode_map(
            ORDERED_MAP,
            values.iter().map(|pair| (&pair.0, &pair.1)),
            output,
            depth,
        )?,
        Value::SortedMap(values) => encode_map(SORTED_MAP, values.iter(), output, depth)?,
        Value::Trie(values) => {
            let entries = values
                .iter()
                .map(|key| {
                    (
                        Value::String(key.clone()),
                        values.get(&key).unwrap().clone(),
                    )
                })
                .collect::<Vec<_>>();
            encode_map(
                TRIE,
                entries.iter().map(|pair| (&pair.0, &pair.1)),
                output,
                depth,
            )?;
        }
        Value::Map(values) => {
            let mut encoded = values
                .iter()
                .map(|(key, value)| Ok((bare(key, depth + 1)?, bare(value, depth + 1)?)))
                .collect::<Result<Vec<_>, String>>()?;
            encoded.sort_by(|left, right| left.0.cmp(&right.0));
            output.push(MAP);
            encode_len(encoded.len(), output)?;
            for (key, value) in encoded {
                output.extend_from_slice(&key);
                output.extend_from_slice(&value);
            }
        }
        Value::Namespace(value) => {
            output.push(NAMESPACE);
            encode_bytes(value.name().as_str().as_bytes(), output)?;
        }
        Value::Var(value) => {
            output.push(VAR);
            encode_bare(&Value::Symbol(value.symbol().clone()), output, depth + 1)?;
            if encode_bare(&value.deref_value(), output, depth + 1).is_err() {
                encode_bare(&Value::Nil, output, depth + 1)?;
            }
        }
        Value::Atom(value) => {
            output.push(ATOM);
            encode_bare(&value.deref_value(), output, depth + 1)?;
        }
        Value::Array(values) => encode_sequence(ARRAY, values.borrow().iter(), output, depth)?,
        Value::Object(values) => {
            let values = values.borrow();
            output.push(OBJECT);
            encode_len(values.len(), output)?;
            for (key, value) in values.iter() {
                encode_bare(&Value::String(key.clone()), output, depth + 1)?;
                encode_bare(value, output, depth + 1)?;
            }
        }
        Value::Extension(value) => {
            output.push(HANDLE);
            encode_bytes(value.provider.as_bytes(), output)?;
            encode_bytes(value.type_name.as_bytes(), output)?;
            output.extend_from_slice(&value.handle.to_be_bytes());
        }
        Value::Tagged(value) => {
            output.push(TAGGED);
            encode_bare(&Value::Symbol(value.tag().clone()), output, depth + 1)?;
            encode_bare(value.form(), output, depth + 1)?;
        }
        Value::ExceptionInfo(value) => {
            output.push(EXCEPTION_INFO);
            encode_bare(&Value::String(value.message.clone()), output, depth + 1)?;
            encode_bare(&value.data, output, depth + 1)?;
            encode_bare(
                value.cause.as_deref().unwrap_or(&Value::Nil),
                output,
                depth + 1,
            )?;
        }
        Value::Struct(value) => {
            output.push(STRUCT);
            encode_bare(&Value::String(value.ty.name.clone()), output, depth + 1)?;
            let fields = value
                .ty
                .fields
                .iter()
                .cloned()
                .map(Value::String)
                .collect::<Vec<_>>();
            encode_sequence(VECTOR, fields.iter(), output, depth)?;
            let values = value.ordered_values();
            encode_sequence(VECTOR, values.into_iter(), output, depth)?;
        }
        Value::Mutable(_) | Value::MutableType(_) => {
            return Err(
                "hta/value-unsupported: mutable values are not serializable; use (into {} value)"
                    .into(),
            )
        }
        _ => return Err(format!("hta/value-unsupported: {}", value.display())),
    }
    Ok(())
}

fn encode_map<'a>(
    tag: u8,
    values: impl Iterator<Item = (&'a Value, &'a Value)>,
    output: &mut Vec<u8>,
    depth: usize,
) -> Result<(), String> {
    let values = values.collect::<Vec<_>>();
    output.push(tag);
    encode_len(values.len(), output)?;
    for (key, value) in values {
        encode_bare(key, output, depth + 1)?;
        encode_bare(value, output, depth + 1)?;
    }
    Ok(())
}

fn bare(value: &Value, depth: usize) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    encode_bare(value, &mut output, depth)?;
    Ok(output)
}

fn encode_sequence<'a>(
    tag: u8,
    values: impl Iterator<Item = &'a Value>,
    output: &mut Vec<u8>,
    depth: usize,
) -> Result<(), String> {
    let values = values.collect::<Vec<_>>();
    output.push(tag);
    encode_len(values.len(), output)?;
    for value in values {
        encode_bare(value, output, depth + 1)?;
    }
    Ok(())
}

fn encode_bytes(value: &[u8], output: &mut Vec<u8>) -> Result<(), String> {
    encode_len(value.len(), output)?;
    output.extend_from_slice(value);
    Ok(())
}
fn encode_len(value: usize, output: &mut Vec<u8>) -> Result<(), String> {
    let value = u32::try_from(value).map_err(|_| "hta/value-too-large")?;
    output.extend_from_slice(&value.to_be_bytes());
    Ok(())
}

struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}
impl Reader<'_> {
    fn value(&mut self, depth: usize) -> Result<Value, String> {
        if depth > MAX_NESTING_DEPTH {
            return Err("hta/value-too-deep: nesting exceeds 256".into());
        }
        let tag = self.byte()?;
        match tag {
            NIL => Ok(Value::Nil),
            FALSE => Ok(Value::Bool(false)),
            TRUE => Ok(Value::Bool(true)),
            I64 => {
                let bytes = self.take(8)?;
                Ok(Value::Number(i64::from_be_bytes(bytes.try_into().unwrap())))
            }
            F64 => {
                let bytes = self.take(8)?;
                Ok(Value::Float(f64::from_bits(u64::from_be_bytes(
                    bytes.try_into().unwrap(),
                ))))
            }
            CHARACTER => {
                let codepoint = u32::from_be_bytes(self.take(4)?.try_into().unwrap());
                char::from_u32(codepoint)
                    .map(Value::Character)
                    .ok_or_else(|| "hta/value-malformed: invalid character scalar".into())
            }
            BIG_INTEGER => Ok(Value::BigInteger(
                String::from_utf8(self.data()?.to_vec())
                    .map_err(|_| "hta/value-malformed: invalid big integer")?,
            )),
            DECIMAL => Ok(Value::Decimal(
                String::from_utf8(self.data()?.to_vec())
                    .map_err(|_| "hta/value-malformed: invalid decimal")?,
            )),
            REGEX => Ok(Value::Regex(
                String::from_utf8(self.data()?.to_vec())
                    .map_err(|_| "hta/value-malformed: invalid regex")?,
            )),
            STRING => Ok(Value::String(
                String::from_utf8(self.data()?.to_vec())
                    .map_err(|_| "hta/value-malformed: invalid UTF-8")?,
            )),
            BYTES => Ok(Value::Bytes(self.data()?.to_vec())),
            KEYWORD => Ok(Value::Keyword(
                String::from_utf8(self.data()?.to_vec())
                    .map_err(|_| "hta/value-malformed: invalid UTF-8")?
                    .into(),
            )),
            SYMBOL => Ok(Value::Symbol(
                String::from_utf8(self.data()?.to_vec())
                    .map_err(|_| "hta/value-malformed: invalid UTF-8")?
                    .into(),
            )),
            LIST => Ok(Value::List(self.sequence(depth)?.into())),
            TUPLE => Ok(Value::Tuple(Box::new(
                crate::lang::data::Tuple::from_values(self.sequence(depth)?)?,
            ))),
            VECTOR => Ok(Value::Vector(self.sequence(depth)?.into())),
            CONS => {
                let mut values = self.sequence(depth)?;
                if values.is_empty() {
                    return Err("hta/value-malformed: empty cons".into());
                }
                let first = values.remove(0);
                Ok(Value::Cons(Box::new(crate::lang::data::Cons::new(
                    first,
                    values.into_iter().collect(),
                ))))
            }
            QUEUE => Ok(Value::Queue(Box::new(
                self.sequence(depth)?.into_iter().collect(),
            ))),
            SET => Ok(Value::Set(self.sequence(depth)?.into())),
            ORDERED_SET => Ok(Value::OrderedSet(Box::new(
                self.sequence(depth)?.into_iter().collect(),
            ))),
            SORTED_SET => Ok(Value::SortedSet(Box::new(
                self.sequence(depth)?.into_iter().collect(),
            ))),
            MAP => {
                let size = self.len()?;
                if size > self.bytes.len().saturating_sub(self.cursor) / 2 {
                    return Err("hta/value-malformed: impossible map length".into());
                }
                let mut values = Vec::with_capacity(size);
                for _ in 0..size {
                    values.push((self.value(depth + 1)?, self.value(depth + 1)?));
                }
                Ok(Value::Map(values.into_iter().collect()))
            }
            ORDERED_MAP => Ok(Value::OrderedMap(Box::new(
                self.entries(depth)?.into_iter().collect(),
            ))),
            SORTED_MAP => Ok(Value::SortedMap(Box::new(
                self.entries(depth)?.into_iter().collect(),
            ))),
            TRIE => {
                let mut trie = crate::lang::data::Trie::new();
                for (key, value) in self.entries(depth)? {
                    let Value::String(key) = key else {
                        return Err("hta/value-malformed: invalid trie key".into());
                    };
                    trie = trie.assoc_value(key, value);
                }
                Ok(Value::Trie(Box::new(trie)))
            }
            NAMESPACE => {
                let name = String::from_utf8(self.data()?.to_vec())
                    .map_err(|_| "hta/value-malformed: invalid namespace name")?;
                Ok(Value::Namespace(std::rc::Rc::new(
                    crate::kernel::Namespace::new(name),
                )))
            }
            VAR => {
                let symbol = match self.value(depth + 1)? {
                    Value::Symbol(symbol) => symbol,
                    _ => return Err("hta/value-malformed: invalid var symbol".into()),
                };
                let value = self.value(depth + 1)?;
                Ok(Value::Var(crate::kernel::Var::new(symbol.as_str(), value)))
            }
            ATOM => Ok(Value::Atom(Box::new(crate::core::RuntimeAtom::new(
                self.value(depth + 1)?,
                true,
            )))),
            ARRAY => Ok(Value::Array(std::rc::Rc::new(std::cell::RefCell::new(
                self.sequence(depth)?,
            )))),
            OBJECT => {
                let size = self.len()?;
                if size > self.bytes.len().saturating_sub(self.cursor) / 2 {
                    return Err("hta/value-malformed: impossible object length".into());
                }
                let mut values = Vec::with_capacity(size);
                for _ in 0..size {
                    let Value::String(key) = self.value(depth + 1)? else {
                        return Err("hta/value-malformed: invalid object key".into());
                    };
                    values.push((key, self.value(depth + 1)?));
                }
                Ok(Value::Object(std::rc::Rc::new(std::cell::RefCell::new(
                    values,
                ))))
            }
            HANDLE => {
                let provider = String::from_utf8(self.data()?.to_vec())
                    .map_err(|_| "hta/value-malformed: invalid handle owner")?;
                let type_name = String::from_utf8(self.data()?.to_vec())
                    .map_err(|_| "hta/value-malformed: invalid handle type")?;
                let bytes = self.take(8)?;
                Ok(Value::Extension(crate::core::ExtensionValue {
                    provider,
                    type_name,
                    handle: u64::from_be_bytes(bytes.try_into().unwrap()),
                }))
            }
            TAGGED => {
                let Value::Symbol(tag) = self.value(depth + 1)? else {
                    return Err("hta/value-malformed: invalid tagged literal tag".into());
                };
                Ok(Value::Tagged(Box::new(
                    crate::lang::data::TaggedLiteral::new(tag, self.value(depth + 1)?),
                )))
            }
            EXCEPTION_INFO => {
                let Value::String(message) = self.value(depth + 1)? else {
                    return Err("hta/value-malformed: invalid exception message".into());
                };
                let data = self.value(depth + 1)?;
                let cause = match self.value(depth + 1)? {
                    Value::Nil => None,
                    value => Some(Box::new(value)),
                };
                Ok(Value::ExceptionInfo(std::rc::Rc::new(
                    crate::core::ExceptionInfo {
                        message,
                        data: Box::new(data),
                        cause,
                    },
                )))
            }
            STRUCT => {
                let Value::String(name) = self.value(depth + 1)? else {
                    return Err("hta/value-malformed: invalid struct name".into());
                };
                let fields = match self.value(depth + 1)? {
                    Value::Vector(values) => values
                        .iter()
                        .map(|value| match value {
                            Value::String(field) => Ok(field.clone()),
                            _ => Err("hta/value-malformed: invalid struct field".into()),
                        })
                        .collect::<Result<Vec<_>, String>>()?,
                    _ => return Err("hta/value-malformed: invalid struct fields".into()),
                };
                let values: Vec<Value> = match self.value(depth + 1)? {
                    Value::Vector(values) => values.iter().cloned().collect(),
                    _ => return Err("hta/value-malformed: invalid struct values".into()),
                };
                if fields.len() != values.len() {
                    return Err("hta/value-malformed: struct arity mismatch".into());
                }
                Ok(Value::Struct(std::rc::Rc::new(
                    crate::core::StructValue::from_values(
                        std::rc::Rc::new(crate::core::StructType { name, fields }),
                        values,
                        None,
                    )?,
                )))
            }
            _ => Err("hta/value-malformed: unknown value tag".into()),
        }
    }
    fn sequence(&mut self, depth: usize) -> Result<Vec<Value>, String> {
        let size = self.len()?;
        if size > self.bytes.len().saturating_sub(self.cursor) {
            return Err("hta/value-malformed: impossible sequence length".into());
        }
        (0..size).map(|_| self.value(depth + 1)).collect()
    }
    fn entries(&mut self, depth: usize) -> Result<Vec<(Value, Value)>, String> {
        let size = self.len()?;
        if size > self.bytes.len().saturating_sub(self.cursor) / 2 {
            return Err("hta/value-malformed: impossible map length".into());
        }
        (0..size)
            .map(|_| Ok((self.value(depth + 1)?, self.value(depth + 1)?)))
            .collect()
    }
    fn data(&mut self) -> Result<&[u8], String> {
        let size = self.len()?;
        self.take(size)
    }
    fn len(&mut self) -> Result<usize, String> {
        let bytes = self.take(4)?;
        Ok(u32::from_be_bytes(bytes.try_into().unwrap()) as usize)
    }
    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn take(&mut self, size: usize) -> Result<&[u8], String> {
        let end = self
            .cursor
            .checked_add(size)
            .ok_or("hta/value-malformed: length overflow")?;
        if end > self.bytes.len() {
            return Err("hta/value-malformed: truncated value".into());
        }
        let output = &self.bytes[self.cursor..end];
        self.cursor = end;
        Ok(output)
    }
}

#[cfg(test)]
#[path = "hta/tests.rs"]
mod tests;
