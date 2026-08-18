fn vm_tool_keyword(name: &str) -> Value {
    Value::Keyword(name.into())
}

fn vm_tool_vector(values: impl IntoIterator<Item = Value>) -> Value {
    Value::Vector(PVector::from_iter(values))
}

fn vm_tool_map(entries: impl IntoIterator<Item = (Value, Value)>) -> Value {
    Value::OrderedMap(Box::new(POrderedMap::from_iter(entries)))
}

fn vm_tool_keywords(values: &[&str]) -> Value {
    vm_tool_vector(values.iter().map(|value| vm_tool_keyword(value)))
}

fn vm_tool_provider_descriptor() -> Value {
    #[cfg(feature = "bytecode-vm")]
    let operations = &["validate", "inspect", "disassemble"][..];
    #[cfg(not(feature = "bytecode-vm"))]
    let operations = &["validate", "inspect"][..];

    #[cfg(feature = "bytecode-vm")]
    let formats = vec![
        (
            vm_tool_keyword("halc"),
            vm_tool_keywords(&["validate", "inspect"]),
        ),
        (
            vm_tool_keyword("hbc"),
            vm_tool_keywords(&["validate", "inspect", "disassemble"]),
        ),
    ];
    #[cfg(not(feature = "bytecode-vm"))]
    let formats = vec![(
        vm_tool_keyword("halc"),
        vm_tool_keywords(&["validate", "inspect"]),
    )];

    vm_tool_map([
        (vm_tool_keyword("provider/id"), vm_tool_keyword("rust")),
        (
            vm_tool_keyword("provider/operations"),
            vm_tool_keywords(operations),
        ),
        (vm_tool_keyword("provider/formats"), vm_tool_map(formats)),
        (
            vm_tool_keyword("provider/transforms"),
            vm_tool_vector(std::iter::empty()),
        ),
        (
            vm_tool_keyword("provider/engines"),
            vm_tool_map(std::iter::empty()),
        ),
    ])
}

fn vm_tool_format<'a>(value: &'a Value, operation: &str) -> Result<&'a str, String> {
    match value {
        Value::Keyword(format)
            if format.get_namespace().is_none() && matches!(format.get_name(), "halc" | "hbc") =>
        {
            Ok(format.get_name())
        }
        _ => Err(format!(
            "tool.vm.provider/{operation} expects :halc or :hbc as its format"
        )),
    }
}

fn vm_tool_bytes(value: &Value, operation: &str) -> Result<Vec<u8>, String> {
    match value {
        Value::Bytes(bytes) => Ok(bytes.clone()),
        Value::ByteBuffer(bytes) => Ok(bytes.borrow().clone()),
        _ => Err(format!("tool.vm.provider/{operation} expects Bytes")),
    }
}

fn vm_tool_validate(format: &str, bytes: &[u8]) -> Result<(), String> {
    match format {
        "halc" => crate::kernel::halc::decode_halc(bytes).map(|_| ()),
        "hbc" => {
            #[cfg(feature = "bytecode-vm")]
            {
                crate::vm::decode_program(bytes).map(|_| ())
            }
            #[cfg(not(feature = "bytecode-vm"))]
            {
                let _ = bytes;
                Err("tool.vm.provider does not support :hbc in this runtime profile".into())
            }
        }
        _ => Err(format!("unknown tool.vm format: :{format}")),
    }
}

fn vm_tool_checksum(bytes: &[u8], start: usize) -> Value {
    Value::Bytes(bytes[start..start + 32].to_vec())
}

fn vm_tool_names(values: impl Iterator<Item = String>) -> Value {
    let mut values = values.collect::<Vec<_>>();
    values.sort();
    vm_tool_vector(values.into_iter().map(Value::String))
}

fn vm_tool_inspect_halc(bytes: &[u8]) -> Result<Value, String> {
    let module = crate::kernel::halc::decode_halc(bytes)?;
    let payload_bytes = u32::from_be_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let origin = match module.origin {
        crate::kernel::halc::HalcOrigin::Halc => "halc",
        crate::kernel::halc::HalcOrigin::LegacyHir => "legacy-hir",
    };
    Ok(vm_tool_map([
        (vm_tool_keyword("artifact/format"), vm_tool_keyword("halc")),
        (vm_tool_keyword("artifact/version"), Value::Number(1)),
        (vm_tool_keyword("artifact/origin"), vm_tool_keyword(origin)),
        (
            vm_tool_keyword("artifact/bytes"),
            Value::Number(bytes.len() as i64),
        ),
        (
            vm_tool_keyword("payload/bytes"),
            Value::Number(payload_bytes as i64),
        ),
        (
            vm_tool_keyword("payload/checksum"),
            vm_tool_checksum(bytes, 12),
        ),
        (
            vm_tool_keyword("module/namespace"),
            Value::String(module.namespace),
        ),
        (
            vm_tool_keyword("module/resource"),
            Value::String(module.resource),
        ),
        (
            vm_tool_keyword("source/hash"),
            Value::Bytes(module.source_hash),
        ),
        (
            vm_tool_keyword("forms/count"),
            Value::Number(module.forms.len() as i64),
        ),
        (
            vm_tool_keyword("schemas/definitions"),
            vm_tool_names(module.schemas.definitions.keys().cloned()),
        ),
        (
            vm_tool_keyword("schemas/functions"),
            vm_tool_names(module.schemas.functions.keys().cloned()),
        ),
    ]))
}

#[cfg(feature = "bytecode-vm")]
fn vm_tool_inspect_hbc(bytes: &[u8]) -> Result<Value, String> {
    let program = crate::vm::decode_program(bytes)?;
    let payload_bytes = u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as usize;
    let checksum_start = 8 + payload_bytes;
    let instructions = program
        .functions
        .iter()
        .map(|function| function.code.len())
        .sum::<usize>();
    let handlers = program
        .functions
        .iter()
        .map(|function| function.handlers.len())
        .sum::<usize>();
    Ok(vm_tool_map([
        (vm_tool_keyword("artifact/format"), vm_tool_keyword("hbc")),
        (vm_tool_keyword("artifact/version"), Value::Number(0)),
        (
            vm_tool_keyword("artifact/bytes"),
            Value::Number(bytes.len() as i64),
        ),
        (
            vm_tool_keyword("payload/bytes"),
            Value::Number(payload_bytes as i64),
        ),
        (
            vm_tool_keyword("payload/checksum"),
            vm_tool_checksum(bytes, checksum_start),
        ),
        (
            vm_tool_keyword("module/namespace"),
            program.namespace.map(Value::String).unwrap_or(Value::Nil),
        ),
        (
            vm_tool_keyword("program/entry"),
            Value::Number(i64::from(program.entry)),
        ),
        (
            vm_tool_keyword("constants/count"),
            Value::Number(program.constants.len() as i64),
        ),
        (
            vm_tool_keyword("functions/count"),
            Value::Number(program.functions.len() as i64),
        ),
        (
            vm_tool_keyword("instructions/count"),
            Value::Number(instructions as i64),
        ),
        (
            vm_tool_keyword("handlers/count"),
            Value::Number(handlers as i64),
        ),
    ]))
}

fn vm_tool_inspect(format: &str, bytes: &[u8]) -> Result<Value, String> {
    match format {
        "halc" => vm_tool_inspect_halc(bytes),
        "hbc" => {
            #[cfg(feature = "bytecode-vm")]
            {
                vm_tool_inspect_hbc(bytes)
            }
            #[cfg(not(feature = "bytecode-vm"))]
            {
                let _ = bytes;
                Err("tool.vm.provider does not support :hbc in this runtime profile".into())
            }
        }
        _ => Err(format!("unknown tool.vm format: :{format}")),
    }
}

fn vm_tool_disassemble(bytes: &[u8]) -> Result<String, String> {
    #[cfg(feature = "bytecode-vm")]
    {
        let program = crate::vm::decode_program(bytes)?;
        Ok(crate::vm::disassemble(&program))
    }
    #[cfg(not(feature = "bytecode-vm"))]
    {
        let _ = bytes;
        Err("tool.vm.provider does not support HBC disassembly in this runtime profile".into())
    }
}

pub(crate) fn vm_tool_provider_values() -> Vec<(&'static str, Value)> {
    vec![
        (
            "provider",
            native_function("tool.vm.provider/provider", 0, |_| {
                Ok(vm_tool_provider_descriptor())
            }),
        ),
        (
            "validate",
            native_function("tool.vm.provider/validate", 2, |arguments| {
                let format = vm_tool_format(&arguments[0], "validate")?;
                let bytes = vm_tool_bytes(&arguments[1], "validate")?;
                vm_tool_validate(format, &bytes)?;
                Ok(Value::Bool(true))
            }),
        ),
        (
            "inspect",
            native_function("tool.vm.provider/inspect", 2, |arguments| {
                let format = vm_tool_format(&arguments[0], "inspect")?;
                let bytes = vm_tool_bytes(&arguments[1], "inspect")?;
                vm_tool_inspect(format, &bytes)
            }),
        ),
        (
            "disassemble",
            native_function("tool.vm.provider/disassemble", 1, |arguments| {
                let bytes = vm_tool_bytes(&arguments[0], "disassemble")?;
                vm_tool_disassemble(&bytes).map(Value::String)
            }),
        ),
    ]
}

#[cfg(test)]
mod vm_tool_tests {
    use super::*;

    fn field(value: &Value, key: &str) -> Value {
        map_value(value, &vm_tool_keyword(key))
            .cloned()
            .unwrap_or(Value::Nil)
    }

    #[test]
    fn provider_reports_exact_read_only_capabilities() {
        let provider = vm_tool_provider_descriptor();
        assert_eq!(field(&provider, "provider/id").display(), ":rust");
        #[cfg(feature = "bytecode-vm")]
        assert_eq!(
            field(&provider, "provider/operations").display(),
            "[:validate :inspect :disassemble]"
        );
        #[cfg(not(feature = "bytecode-vm"))]
        assert_eq!(
            field(&provider, "provider/operations").display(),
            "[:validate :inspect]"
        );
        assert_eq!(field(&provider, "provider/transforms").display(), "[]");
        assert_eq!(field(&provider, "provider/engines").display(), "{}");
    }

    #[test]
    fn halc_validation_and_inspection_use_canonical_decoder() {
        let source = "(ns sample.vm) (def value 42)";
        let forms = crate::kernel::parse_forms(source).unwrap();
        let bytes =
            crate::kernel::halc::encode_halc_module("sample.vm", "sample/vm.hal", source, forms)
                .unwrap();
        vm_tool_validate("halc", &bytes).unwrap();
        let inspection = vm_tool_inspect("halc", &bytes).unwrap();
        assert_eq!(field(&inspection, "artifact/format").display(), ":halc");
        assert_eq!(
            field(&inspection, "module/namespace").display(),
            "\"sample.vm\""
        );
        assert_eq!(field(&inspection, "forms/count").display(), "2");
        assert_eq!(field(&inspection, "artifact/origin").display(), ":halc");
    }

    #[cfg(feature = "bytecode-vm")]
    #[test]
    fn hbc_validation_inspection_and_disassembly_use_canonical_vm() {
        let program = crate::vm::compile_source("(+ 19 23)").unwrap();
        let bytes = crate::vm::encode_program(&program).unwrap();
        vm_tool_validate("hbc", &bytes).unwrap();
        let inspection = vm_tool_inspect("hbc", &bytes).unwrap();
        assert_eq!(field(&inspection, "artifact/format").display(), ":hbc");
        assert_eq!(field(&inspection, "functions/count").display(), "1");
        assert!(vm_tool_disassemble(&bytes)
            .unwrap()
            .starts_with("== program:"));
    }
}
