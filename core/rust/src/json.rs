use crate::core::{ExceptionInfo, ResultValue, Value};
use crate::lang::data::{OrderedMap, Vector};
use std::rc::Rc;

const MAX_DEPTH: usize = 256;

pub fn read(source: &str) -> Result<Value, String> {
    let mut parser = Parser::new(source);
    let value = parser.value(0)?;
    parser.whitespace();
    if parser.peek().is_some() {
        return Err(parser.error("trailing content after JSON value"));
    }
    decode_envelopes(value)
}

pub fn write(value: &Value) -> Result<String, String> {
    let mut out = String::new();
    encode(&mut out, value, 0, false)?;
    Ok(out)
}

pub fn write_pretty(value: &Value) -> Result<String, String> {
    let mut out = String::new();
    encode(&mut out, value, 0, true)?;
    Ok(out)
}

fn encode(out: &mut String, value: &Value, depth: usize, pretty: bool) -> Result<(), String> {
    match value {
        Value::Nil => out.push_str("null"),
        Value::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
        Value::Number(value) => out.push_str(&value.to_string()),
        Value::String(value) => string(out, value),
        Value::Result(result) => encode_object(
            out,
            &[
                ("$hara", Value::String("result".into())),
                ("status", Value::String(result.status.keyword().into())),
                ("data", result.data.clone()),
                ("error", result.error_value()),
                ("context", result.transport_context()),
            ],
            depth,
            pretty,
        )?,
        Value::ExceptionInfo(error) => encode_object(
            out,
            &[
                ("$hara", Value::String("error".into())),
                ("message", Value::String(error.message.clone())),
                ("data", (*error.data).clone()),
                (
                    "cause",
                    error.cause.as_deref().cloned().unwrap_or(Value::Nil),
                ),
            ],
            depth,
            pretty,
        )?,
        Value::Vector(values) => {
            out.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 { out.push(','); }
                if pretty { newline(out, depth + 1); }
                encode(out, value, depth + 1, pretty)?;
            }
            if pretty && values.len() > 0 { newline(out, depth); }
            out.push(']');
        }
        Value::Tuple(values) => {
            out.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 { out.push(','); }
                if pretty { newline(out, depth + 1); }
                encode(out, value, depth + 1, pretty)?;
            }
            if pretty && !values.is_empty() { newline(out, depth); }
            out.push(']');
        }
        value @ (Value::Map(_) | Value::OrderedMap(_) | Value::SortedMap(_) | Value::Trie(_)) => {
            let values = crate::core::map_entries(value).expect("map values have entries");
            out.push('{');
            for (index, (key, value)) in values.iter().enumerate() {
                let Value::String(key) = key else {
                    return Err("json/write expects maps with string keys".into());
                };
                if index > 0 { out.push(','); }
                if pretty { newline(out, depth + 1); }
                string(out, key);
                out.push_str(if pretty { ": " } else { ":" });
                encode(out, value, depth + 1, pretty)?;
            }
            if pretty && !values.is_empty() { newline(out, depth); }
            out.push('}');
        }
        _ => return Err("json/write accepts nil, booleans, signed 64-bit integers, strings, vectors, string-key maps, and native Result/Error envelopes containing those values".into()),
    }
    Ok(())
}

fn encode_object(
    out: &mut String,
    entries: &[(&str, Value)],
    depth: usize,
    pretty: bool,
) -> Result<(), String> {
    out.push('{');
    for (index, (key, value)) in entries.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        if pretty {
            newline(out, depth + 1);
        }
        string(out, key);
        out.push_str(if pretty { ": " } else { ":" });
        encode(out, value, depth + 1, pretty)?;
    }
    if pretty && !entries.is_empty() {
        newline(out, depth);
    }
    out.push('}');
    Ok(())
}

fn decode_envelopes(value: Value) -> Result<Value, String> {
    match value {
        Value::Vector(values) => Ok(Value::Vector(Vector::from_iter(
            values
                .iter()
                .cloned()
                .map(decode_envelopes)
                .collect::<Result<Vec<_>, _>>()?,
        ))),
        value @ (Value::Map(_) | Value::OrderedMap(_) | Value::SortedMap(_) | Value::Trie(_)) => {
            let entries = crate::core::map_entries(&value)
                .expect("JSON object values have map entries")
                .into_iter()
                .map(|(key, value)| Ok((key, decode_envelopes(value)?)))
                .collect::<Result<Vec<_>, String>>()?;
            decode_object(entries)
        }
        value => Ok(value),
    }
}

fn decode_object(entries: Vec<(Value, Value)>) -> Result<Value, String> {
    let generic = || Value::OrderedMap(Box::new(OrderedMap::from_iter(entries.iter().cloned())));
    let Some(Value::String(tag)) = object_field(&entries, "$hara") else {
        return Ok(generic());
    };
    match tag.as_str() {
        "result" if exact_object(&entries, &["$hara", "status", "data", "error", "context"]) => {
            decode_result(&entries)
        }
        "error" if exact_object(&entries, &["$hara", "message", "data", "cause"]) => {
            decode_error(&entries)
        }
        _ => Ok(generic()),
    }
}

fn decode_result(entries: &[(Value, Value)]) -> Result<Value, String> {
    let status = match object_field(entries, "status") {
        Some(Value::String(status)) => status.as_str(),
        _ => return Err("json/read: malformed Hara Result status".into()),
    };
    let data = object_field(entries, "data").expect("exact Result envelope");
    let error = object_field(entries, "error").expect("exact Result envelope");
    let context = object_field(entries, "context").expect("exact Result envelope");
    if crate::core::map_entries(context).is_none() {
        return Err("json/read: malformed Hara Result context".into());
    }
    let result = match status {
        "success" => {
            if !matches!(error, Value::Nil) {
                return Err("json/read: malformed success Result contains an error".into());
            }
            ResultValue::success(data.clone(), context.clone())
        }
        "error" => {
            if !matches!(data, Value::Nil) {
                return Err("json/read: malformed error Result contains success data".into());
            }
            if !matches!(error, Value::ExceptionInfo(_)) {
                return Err("json/read: malformed error Result lacks a native Error".into());
            }
            ResultValue::error(error.clone(), context.clone())
        }
        _ => return Err("json/read: malformed Hara Result status".into()),
    }
    .map_err(|error| format!("json/read: malformed Hara Result: {error}"))?;
    Ok(Value::Result(Rc::new(result)))
}

fn decode_error(entries: &[(Value, Value)]) -> Result<Value, String> {
    let message = match object_field(entries, "message") {
        Some(Value::String(message)) => message.clone(),
        _ => return Err("json/read: malformed Hara Error message".into()),
    };
    let data = object_field(entries, "data").expect("exact Error envelope");
    if crate::core::map_entries(data).is_none() {
        return Err("json/read: malformed Hara Error data".into());
    }
    let cause = match object_field(entries, "cause").expect("exact Error envelope") {
        Value::Nil => None,
        value => Some(Box::new(value.clone())),
    };
    Ok(Value::ExceptionInfo(Rc::new(ExceptionInfo {
        message,
        data: Box::new(data.clone()),
        cause,
    })))
}

fn object_field<'a>(entries: &'a [(Value, Value)], name: &str) -> Option<&'a Value> {
    entries
        .iter()
        .find_map(|(key, value)| matches!(key, Value::String(key) if key == name).then_some(value))
}

fn exact_object(entries: &[(Value, Value)], fields: &[&str]) -> bool {
    entries.len() == fields.len()
        && fields
            .iter()
            .all(|field| object_field(entries, field).is_some())
}

fn newline(out: &mut String, depth: usize) {
    out.push('\n');
    out.push_str(&"  ".repeat(depth));
}

fn string(out: &mut String, value: &str) {
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            character if character <= '\u{1f}' => {
                out.push_str(&format!("\\u{:04x}", character as u32))
            }
            character => out.push(character),
        }
    }
    out.push('"');
}

struct Parser {
    input: Vec<char>,
    offset: usize,
}

impl Parser {
    fn new(source: &str) -> Self {
        Self {
            input: source.chars().collect(),
            offset: 0,
        }
    }
    fn peek(&self) -> Option<char> {
        self.input.get(self.offset).copied()
    }
    fn take(&mut self) -> Option<char> {
        let value = self.peek();
        if value.is_some() {
            self.offset += 1;
        }
        value
    }
    fn error(&self, message: &str) -> String {
        format!("json/read: {message} at character {}", self.offset)
    }
    fn whitespace(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.offset += 1;
        }
    }
    fn expect(&mut self, expected: char) -> Result<(), String> {
        if self.take() == Some(expected) {
            Ok(())
        } else {
            Err(self.error(&format!("expected '{expected}'")))
        }
    }
    fn value(&mut self, depth: usize) -> Result<Value, String> {
        if depth > MAX_DEPTH {
            return Err(self.error("JSON nesting exceeds 256"));
        }
        self.whitespace();
        match self.peek() {
            Some('n') => {
                self.literal("null")?;
                Ok(Value::Nil)
            }
            Some('t') => {
                self.literal("true")?;
                Ok(Value::Bool(true))
            }
            Some('f') => {
                self.literal("false")?;
                Ok(Value::Bool(false))
            }
            Some('"') => self.string().map(Value::String),
            Some('[') => self.array(depth + 1),
            Some('{') => self.object(depth + 1),
            Some(_) => self.number(),
            None => Err(self.error("expected a JSON value")),
        }
    }
    fn literal(&mut self, literal: &str) -> Result<(), String> {
        for expected in literal.chars() {
            if self.take() != Some(expected) {
                return Err(self.error("invalid JSON token"));
            }
        }
        Ok(())
    }
    fn number(&mut self) -> Result<Value, String> {
        let start = self.offset;
        if self.peek() == Some('-') {
            self.offset += 1;
        }
        match self.peek() {
            Some('0') => {
                self.offset += 1;
                if matches!(self.peek(), Some('0'..='9')) {
                    return Err(self.error("leading zero in JSON number"));
                }
            }
            Some('1'..='9') => {
                while matches!(self.peek(), Some('0'..='9')) {
                    self.offset += 1;
                }
            }
            _ => return Err(self.error("expected a JSON value")),
        }
        if matches!(self.peek(), Some('.' | 'e' | 'E')) {
            return Err(self.error("JSON v1 supports signed 64-bit integers only"));
        }
        self.input[start..self.offset]
            .iter()
            .collect::<String>()
            .parse::<i64>()
            .map(Value::Number)
            .map_err(|_| self.error("JSON integer is outside the signed 64-bit range"))
    }
    fn array(&mut self, depth: usize) -> Result<Value, String> {
        self.expect('[')?;
        self.whitespace();
        let mut values = Vec::new();
        if self.peek() == Some(']') {
            self.offset += 1;
            return Ok(Value::Vector(Vector::from_iter(values)));
        }
        loop {
            values.push(self.value(depth)?);
            self.whitespace();
            if self.peek() == Some(']') {
                self.offset += 1;
                return Ok(Value::Vector(Vector::from_iter(values)));
            }
            self.expect(',')?;
            self.whitespace();
            if self.peek() == Some(']') {
                return Err(self.error("trailing commas are not valid JSON"));
            }
        }
    }
    fn object(&mut self, depth: usize) -> Result<Value, String> {
        self.expect('{')?;
        self.whitespace();
        let mut values: Vec<(Value, Value)> = Vec::new();
        if self.peek() == Some('}') {
            self.offset += 1;
            return Ok(Value::OrderedMap(Box::new(OrderedMap::from_iter(values))));
        }
        loop {
            if self.peek() != Some('"') {
                return Err(self.error("JSON object keys must be strings"));
            }
            let key = self.string()?;
            self.whitespace();
            self.expect(':')?;
            let value = self.value(depth)?;
            if values.iter().any(
                |(existing, _)| matches!(existing, Value::String(existing) if existing == &key),
            ) {
                return Err(self.error("duplicate JSON object key"));
            }
            values.push((Value::String(key), value));
            self.whitespace();
            if self.peek() == Some('}') {
                self.offset += 1;
                return Ok(Value::OrderedMap(Box::new(OrderedMap::from_iter(values))));
            }
            self.expect(',')?;
            self.whitespace();
            if self.peek() == Some('}') {
                return Err(self.error("trailing commas are not valid JSON"));
            }
        }
    }
    fn string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            let Some(character) = self.take() else {
                return Err(self.error("unterminated JSON string"));
            };
            match character {
                '"' => return Ok(out),
                character if character < '\u{20}' => {
                    return Err(self.error("unescaped control character"))
                }
                '\\' => match self.take() {
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('/') => out.push('/'),
                    Some('b') => out.push('\u{08}'),
                    Some('f') => out.push('\u{0c}'),
                    Some('n') => out.push('\n'),
                    Some('r') => out.push('\r'),
                    Some('t') => out.push('\t'),
                    Some('u') => out.push(self.unicode_escape()?),
                    _ => return Err(self.error("invalid JSON escape")),
                },
                character => out.push(character),
            }
        }
    }
    fn unicode_escape(&mut self) -> Result<char, String> {
        let mut value = 0u32;
        for _ in 0..4 {
            let Some(character) = self.take() else {
                return Err(self.error("incomplete Unicode escape"));
            };
            value = value
                .checked_mul(16)
                .and_then(|value| character.to_digit(16).map(|digit| value + digit))
                .ok_or_else(|| self.error("invalid Unicode escape"))?;
        }
        char::from_u32(value).ok_or_else(|| self.error("invalid Unicode escape"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::data::{Keyword, Map as PMap};

    fn string_map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
        Value::Map(PMap::from_iter(
            entries
                .into_iter()
                .map(|(key, value)| (Value::String(key.into()), value)),
        ))
    }

    #[test]
    fn native_results_and_errors_round_trip_through_json_envelopes() {
        let success = Value::Result(Rc::new(
            ResultValue::success(
                string_map([("value", Value::Number(42))]),
                string_map([("source", Value::String("rpc".into()))]),
            )
            .unwrap(),
        ));
        let encoded = write(&success).unwrap();
        assert!(encoded.starts_with("{\"$hara\":\"result\",\"status\":\"success\""));
        let Value::Result(decoded) = read(&encoded).unwrap() else {
            panic!("expected native Result");
        };
        assert!(decoded.is_success());
        assert_eq!(
            crate::core::map_entries(&decoded.context)
                .unwrap()
                .into_iter()
                .find(|(key, _)| key == &Value::String("source".into()))
                .map(|(_, value)| value),
            Some(Value::String("rpc".into()))
        );

        let error = Value::ExceptionInfo(Rc::new(ExceptionInfo {
            message: "boom".into(),
            data: Box::new(string_map([("code", Value::String("demo/boom".into()))])),
            cause: None,
        }));
        let failure = Value::Result(Rc::new(
            ResultValue::error(error, string_map([("source", Value::String("rpc".into()))]))
                .unwrap(),
        ));
        let encoded = write(&failure).unwrap();
        let Value::Result(decoded) = read(&encoded).unwrap() else {
            panic!("expected native Result");
        };
        assert!(decoded.is_error());
        let Value::ExceptionInfo(error) = decoded.error_value() else {
            panic!("expected native Error");
        };
        assert_eq!(error.message, "boom");
    }

    #[test]
    fn result_json_strips_display_and_rejects_other_nonportable_context() {
        let display = crate::core::native_function("result-display", 1, |_| {
            Ok(Value::String("rendered".into()))
        });
        let context = Value::Map(PMap::from_iter([
            (Value::Keyword(Keyword::from("display")), display),
            (Value::String("source".into()), Value::String("json".into())),
        ]));
        let result = Value::Result(Rc::new(
            ResultValue::success(Value::Number(1), context).unwrap(),
        ));
        let encoded = write(&result).unwrap();
        assert!(!encoded.contains("display"));
        assert!(encoded.contains("\"source\":\"json\""));

        let nonportable = Value::Result(Rc::new(
            ResultValue::success(
                Value::Number(1),
                string_map([("native", Value::Promise(crate::core::Promise::new()))]),
            )
            .unwrap(),
        ));
        assert!(write(&nonportable)
            .unwrap_err()
            .contains("json/write accepts"));
    }

    #[test]
    fn malformed_exact_result_envelopes_are_rejected() {
        let malformed =
            r#"{"$hara":"result","status":"success","data":1,"error":{"bad":true},"context":{}}"#;
        assert!(read(malformed)
            .unwrap_err()
            .contains("success Result contains an error"));
        let generic = r#"{"$hara":"result","status":"success","data":1,"error":null,"context":{},"extra":true}"#;
        assert!(matches!(read(generic).unwrap(), Value::OrderedMap(_)));
    }
}
