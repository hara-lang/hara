use crate::kernel::Form;

use super::{
    BindingFunction, BindingParameter, BindingResult, ErrorContract, HaraValueType, Lifting,
    Lowering, MemoryContract, Ownership, WasmInterface,
};

impl HaraValueType {
    fn canonical_form(&self) -> Form {
        match self {
            Self::I32 => keyword_form("i32"),
            Self::I64 => keyword_form("i64"),
            Self::F32 => keyword_form("f32"),
            Self::F64 => keyword_form("f64"),
            Self::Boolean => keyword_form("boolean"),
            Self::String => keyword_form("string"),
            Self::Bytes => keyword_form("bytes"),
            Self::Record(name) => named_type_form("record", name),
            Self::Variant(name) => named_type_form("variant", name),
            Self::Handle(name) => named_type_form("handle", name),
            Self::Callback(name) => named_type_form("callback", name),
            Self::Void => keyword_form("void"),
        }
    }
}

impl Ownership {
    fn as_keyword(self) -> &'static str {
        match self {
            Self::Borrowed => "borrowed",
            Self::Caller => "caller",
            Self::Callee => "callee",
            Self::Transferred => "transferred",
        }
    }
}

impl Lowering {
    fn canonical_form(self) -> Form {
        match self {
            Self::Direct => keyword_form("direct"),
            Self::PointerLength => {
                Form::Vector(vec![keyword_form("pointer"), keyword_form("length")])
            }
        }
    }
}

impl Lifting {
    fn canonical_form(self) -> Form {
        match self {
            Self::Direct => keyword_form("direct"),
            Self::PointerLength => {
                Form::Vector(vec![keyword_form("pointer"), keyword_form("length")])
            }
            Self::PackedI64 => keyword_form("packed-i64"),
        }
    }
}

pub(super) fn source(interface: &WasmInterface) -> String {
    Form::List(vec![symbol_form("wasm/interface"), payload_form(interface)]).to_string()
}

fn payload_form(interface: &WasmInterface) -> Form {
    let mut entries = vec![
        (keyword_form("schema"), string_form(&interface.schema)),
        (keyword_form("namespace"), symbol_form(&interface.namespace)),
        (keyword_form("module"), string_form(&interface.module)),
    ];
    if let Some(memory) = interface.memory.as_ref() {
        entries.push((keyword_form("memory"), memory_form(memory)));
    }
    entries.push((
        keyword_form("exports"),
        Form::Map(
            interface
                .exports
                .iter()
                .map(|export| (symbol_form(&export.name), export_form(export)))
                .collect(),
        ),
    ));
    if !interface.capabilities.is_empty() {
        entries.push((
            keyword_form("capabilities"),
            Form::Vector(
                interface
                    .capabilities
                    .iter()
                    .map(|capability| keyword_form(capability))
                    .collect(),
            ),
        ));
    }
    Form::Map(entries)
}

fn memory_form(memory: &MemoryContract) -> Form {
    let mut entries = vec![(keyword_form("export"), string_form(&memory.export))];
    push_optional_string(&mut entries, "allocate", memory.allocate.as_deref());
    push_optional_string(&mut entries, "reallocate", memory.reallocate.as_deref());
    push_optional_string(&mut entries, "release", memory.release.as_deref());
    Form::Map(entries)
}

fn export_form(export: &BindingFunction) -> Form {
    let mut entries = vec![
        (
            keyword_form("wasm/export"),
            string_form(&export.wasm_export),
        ),
        (
            keyword_form("arguments"),
            Form::Vector(export.arguments.iter().map(parameter_form).collect()),
        ),
        (keyword_form("returns"), result_form(&export.returns)),
    ];
    if export.asynchronous {
        entries.push((keyword_form("async"), Form::Bool(true)));
    }
    if let Some(errors) = export.errors.as_ref() {
        entries.push((keyword_form("errors"), error_form(errors)));
    }
    if !export.capabilities.is_empty() {
        entries.push((
            keyword_form("capabilities"),
            Form::Vector(
                export
                    .capabilities
                    .iter()
                    .map(|capability| keyword_form(capability))
                    .collect(),
            ),
        ));
    }
    Form::Map(entries)
}

fn parameter_form(parameter: &BindingParameter) -> Form {
    let mut entries = vec![
        (keyword_form("name"), symbol_form(&parameter.name)),
        (
            keyword_form("hara/type"),
            parameter.hara_type.canonical_form(),
        ),
        (
            keyword_form("wasm/type"),
            keyword_form(parameter.wasm_type.as_keyword()),
        ),
    ];
    if let Some(lowering) = parameter.lowering {
        entries.push((keyword_form("lower"), lowering.canonical_form()));
    }
    if let Some(ownership) = parameter.ownership {
        entries.push((
            keyword_form("ownership"),
            keyword_form(ownership.as_keyword()),
        ));
    }
    Form::Map(entries)
}

fn result_form(result: &BindingResult) -> Form {
    let mut entries = vec![
        (keyword_form("hara/type"), result.hara_type.canonical_form()),
        (
            keyword_form("wasm/type"),
            keyword_form(result.wasm_type.as_keyword()),
        ),
    ];
    if let Some(lifting) = result.lifting {
        entries.push((keyword_form("lift"), lifting.canonical_form()));
    }
    if let Some(ownership) = result.ownership {
        entries.push((
            keyword_form("ownership"),
            keyword_form(ownership.as_keyword()),
        ));
    }
    Form::Map(entries)
}

fn error_form(errors: &ErrorContract) -> Form {
    Form::Map(vec![
        (keyword_form("convention"), keyword_form(&errors.convention)),
        (
            keyword_form("codes"),
            Form::Map(
                errors
                    .codes
                    .iter()
                    .map(|(code, value)| (Form::Number(*code), keyword_form(value)))
                    .collect(),
            ),
        ),
    ])
}

fn named_type_form(kind: &str, name: &str) -> Form {
    Form::Vector(vec![keyword_form(kind), symbol_form(name)])
}

fn push_optional_string(entries: &mut Vec<(Form, Form)>, name: &str, value: Option<&str>) {
    if let Some(value) = value {
        entries.push((keyword_form(name), string_form(value)));
    }
}

fn keyword_form(value: &str) -> Form {
    Form::Keyword(value.to_owned())
}

fn symbol_form(value: &str) -> Form {
    Form::Symbol(value.to_owned())
}

fn string_form(value: &str) -> Form {
    Form::String(value.to_owned())
}
