//! Restricted `.hal` interface contracts for portable Wasm extension bindings.
//!
//! Sources are parsed as data with the Hara reader. This module never evaluates
//! an interface, instantiates a module, or acquires host authority.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use sha2::{Digest, Sha256};

use crate::extension::ExtensionExport;
use crate::kernel::{parse, Form};

pub const WASM_INTERFACE_SCHEMA: &str = "hara.wasm-interface/0-alpha";

const INTERFACE_FIELDS: &[&str] = &[
    "schema",
    "namespace",
    "module",
    "memory",
    "exports",
    "imports",
    "capabilities",
    "handles",
];
const MEMORY_FIELDS: &[&str] = &["export", "allocate", "reallocate", "release"];
const EXPORT_FIELDS: &[&str] = &[
    "wasm/export",
    "arguments",
    "returns",
    "async",
    "errors",
    "capabilities",
];
const PARAMETER_FIELDS: &[&str] =
    &["name", "hara/type", "wasm/type", "lower", "ownership"];
const RESULT_FIELDS: &[&str] = &["hara/type", "wasm/type", "lift", "ownership"];
const ERROR_FIELDS: &[&str] = &["convention", "codes"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WasmValueType {
    I32,
    I64,
    F32,
    F64,
    Void,
}

impl WasmValueType {
    pub fn as_keyword(self) -> &'static str {
        match self {
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::Void => "void",
        }
    }

    fn parse(form: &Form, origin: &str, field: &str) -> Result<Self, String> {
        match keyword(form, origin, field)? {
            "i32" => Ok(Self::I32),
            "i64" => Ok(Self::I64),
            "f32" => Ok(Self::F32),
            "f64" => Ok(Self::F64),
            "void" => Ok(Self::Void),
            value => Err(unsupported(
                origin,
                format!("{field} uses unsupported Wasm type :{value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HaraValueType {
    I32,
    I64,
    F32,
    F64,
    Boolean,
    String,
    Bytes,
    Record(String),
    Variant(String),
    Handle(String),
    Callback(String),
    Void,
}

impl HaraValueType {
    fn parse(form: &Form, origin: &str, field: &str) -> Result<Self, String> {
        match form {
            Form::Keyword(value) => match value.as_str() {
                "i32" => Ok(Self::I32),
                "i64" => Ok(Self::I64),
                "f32" => Ok(Self::F32),
                "f64" => Ok(Self::F64),
                "boolean" => Ok(Self::Boolean),
                "string" => Ok(Self::String),
                "bytes" => Ok(Self::Bytes),
                "void" => Ok(Self::Void),
                value => Err(unsupported(
                    origin,
                    format!("{field} uses unsupported Hara type :{value}"),
                )),
            },
            Form::Vector(values) if values.len() == 2 => {
                let kind = keyword(&values[0], origin, field)?;
                let name = named(&values[1], origin, field)?.to_owned();
                if !valid_tag(&name) {
                    return Err(malformed(
                        origin,
                        format!("{field} type name must be lower-case"),
                    ));
                }
                match kind {
                    "record" => Ok(Self::Record(name)),
                    "variant" => Ok(Self::Variant(name)),
                    "handle" => Ok(Self::Handle(name)),
                    "callback" => Ok(Self::Callback(name)),
                    value => Err(unsupported(
                        origin,
                        format!("{field} uses unsupported type constructor :{value}"),
                    )),
                }
            }
            _ => Err(malformed(
                origin,
                format!("{field} must be a type keyword or [kind name] vector"),
            )),
        }
    }

    fn direct_wasm_type(&self) -> Option<WasmValueType> {
        match self {
            Self::I32 => Some(WasmValueType::I32),
            Self::I64 => Some(WasmValueType::I64),
            Self::F32 => Some(WasmValueType::F32),
            Self::F64 => Some(WasmValueType::F64),
            Self::Void => Some(WasmValueType::Void),
            _ => None,
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ownership {
    Borrowed,
    Caller,
    Callee,
    Transferred,
}

impl Ownership {
    fn parse(form: &Form, origin: &str, field: &str) -> Result<Self, String> {
        match keyword(form, origin, field)? {
            "borrowed" => Ok(Self::Borrowed),
            "caller" => Ok(Self::Caller),
            "callee" => Ok(Self::Callee),
            "transferred" => Ok(Self::Transferred),
            value => Err(unsupported(
                origin,
                format!("{field} uses unsupported ownership :{value}"),
            )),
        }
    }

    fn as_keyword(self) -> &'static str {
        match self {
            Self::Borrowed => "borrowed",
            Self::Caller => "caller",
            Self::Callee => "callee",
            Self::Transferred => "transferred",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lowering {
    Direct,
    PointerLength,
}

impl Lowering {
    fn parse(form: &Form, origin: &str, field: &str) -> Result<Self, String> {
        match form {
            Form::Keyword(value) if value == "direct" => Ok(Self::Direct),
            Form::Vector(values) => match values.as_slice() {
                [Form::Keyword(pointer), Form::Keyword(length)]
                    if pointer == "pointer" && length == "length" =>
                {
                    Ok(Self::PointerLength)
                }
                _ => Err(unsupported(
                    origin,
                    format!("{field} uses unsupported lowering"),
                )),
            },
            _ => Err(unsupported(
                origin,
                format!("{field} uses unsupported lowering"),
            )),
        }
    }

    fn canonical_form(self) -> Form {
        match self {
            Self::Direct => keyword_form("direct"),
            Self::PointerLength => Form::Vector(vec![
                keyword_form("pointer"),
                keyword_form("length"),
            ]),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifting {
    Direct,
    PointerLength,
    PackedI64,
}

impl Lifting {
    fn parse(form: &Form, origin: &str, field: &str) -> Result<Self, String> {
        match form {
            Form::Keyword(value) if value == "direct" => Ok(Self::Direct),
            Form::Keyword(value) if value == "packed-i64" => Ok(Self::PackedI64),
            Form::Vector(values) => match values.as_slice() {
                [Form::Keyword(pointer), Form::Keyword(length)]
                    if pointer == "pointer" && length == "length" =>
                {
                    Ok(Self::PointerLength)
                }
                _ => Err(unsupported(
                    origin,
                    format!("{field} uses unsupported lifting"),
                )),
            },
            _ => Err(unsupported(
                origin,
                format!("{field} uses unsupported lifting"),
            )),
        }
    }

    fn canonical_form(self) -> Form {
        match self {
            Self::Direct => keyword_form("direct"),
            Self::PointerLength => Form::Vector(vec![
                keyword_form("pointer"),
                keyword_form("length"),
            ]),
            Self::PackedI64 => keyword_form("packed-i64"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryContract {
    pub export: String,
    pub allocate: Option<String>,
    pub reallocate: Option<String>,
    pub release: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingParameter {
    pub name: String,
    pub hara_type: HaraValueType,
    pub wasm_type: WasmValueType,
    pub lowering: Option<Lowering>,
    pub ownership: Option<Ownership>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingResult {
    pub hara_type: HaraValueType,
    pub wasm_type: WasmValueType,
    pub lifting: Option<Lifting>,
    pub ownership: Option<Ownership>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorContract {
    pub convention: String,
    pub codes: BTreeMap<i64, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingFunction {
    pub name: String,
    pub wasm_export: String,
    pub arguments: Vec<BindingParameter>,
    pub returns: BindingResult,
    pub asynchronous: bool,
    pub errors: Option<ErrorContract>,
    pub capabilities: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmInterface {
    pub schema: String,
    pub namespace: String,
    pub module: String,
    pub memory: Option<MemoryContract>,
    pub exports: Vec<BindingFunction>,
    pub capabilities: BTreeSet<String>,
}

impl WasmInterface {
    pub fn parse(source: &str, origin: &str) -> Result<Self, String> {
        let form = parse(source)
            .map_err(|error| malformed(origin, format!("cannot parse interface: {error}")))?;
        let payload = interface_payload(&form, origin)?;
        let entries = map(payload, origin, "interface")?;
        reject_unknown(entries, INTERFACE_FIELDS, origin, "interface")?;

        reject_reserved_collection(entries, "imports", origin)?;
        reject_reserved_collection(entries, "handles", origin)?;

        let schema = non_empty_string(
            required(entries, "schema", origin)?,
            origin,
            "interface schema",
        )?
        .to_owned();
        if schema != WASM_INTERFACE_SCHEMA {
            return Err(unsupported(
                origin,
                format!("unsupported interface schema {schema}"),
            ));
        }

        let namespace = named(
            required(entries, "namespace", origin)?,
            origin,
            "interface namespace",
        )?
        .to_owned();
        if !valid_namespace(&namespace) {
            return Err(malformed(
                origin,
                "namespace must be a qualified lower-case name",
            ));
        }

        let module = non_empty_string(
            required(entries, "module", origin)?,
            origin,
            "interface module",
        )?
        .to_owned();
        validate_module_path(&module, origin)?;

        let memory = optional(entries, "memory")
            .map(|form| parse_memory(form, origin))
            .transpose()?;
        let exports = parse_exports(required(entries, "exports", origin)?, origin)?;
        let capabilities = optional(entries, "capabilities")
            .map_or_else(|| Ok(BTreeSet::new()), |form| {
                keyword_set(form, origin, "interface capabilities")
            })?;

        let interface = Self {
            schema,
            namespace,
            module,
            memory,
            exports,
            capabilities,
        };
        interface.validate_alpha(origin)?;
        Ok(interface)
    }

    pub fn canonical_source(&self) -> String {
        Form::List(vec![
            symbol_form("wasm/interface"),
            self.canonical_payload(),
        ])
        .to_string()
    }

    pub fn digest(&self) -> String {
        let digest = Sha256::digest(self.canonical_source().as_bytes());
        format!("sha256:{digest:x}")
    }

    pub fn direct_exports(&self) -> Vec<(String, ExtensionExport)> {
        self.exports
            .iter()
            .map(|export| {
                (
                    export.wasm_export.clone(),
                    ExtensionExport {
                        arguments: export
                            .arguments
                            .iter()
                            .map(|argument| argument.wasm_type.as_keyword().to_owned())
                            .collect(),
                        returns: export.returns.wasm_type.as_keyword().to_owned(),
                        asynchronous: false,
                    },
                )
            })
            .collect()
    }

    fn validate_alpha(&self, origin: &str) -> Result<(), String> {
        for export in &self.exports {
            if export.asynchronous {
                return Err(unsupported(
                    origin,
                    format!(
                        "export {} is asynchronous; async bindings require HTA",
                        export.name
                    ),
                ));
            }
            for argument in &export.arguments {
                validate_parameter(argument, origin, &export.name)?;
            }
            validate_result(&export.returns, origin, &export.name)?;
        }
        Ok(())
    }

    fn canonical_payload(&self) -> Form {
        let mut entries = vec![
            (keyword_form("schema"), string_form(&self.schema)),
            (keyword_form("namespace"), symbol_form(&self.namespace)),
            (keyword_form("module"), string_form(&self.module)),
        ];
        if let Some(memory) = self.memory.as_ref() {
            entries.push((keyword_form("memory"), memory_form(memory)));
        }
        entries.push((
            keyword_form("exports"),
            Form::Map(
                self.exports
                    .iter()
                    .map(|export| {
                        (
                            symbol_form(&export.name),
                            export_form(export),
                        )
                    })
                    .collect(),
            ),
        ));
        if !self.capabilities.is_empty() {
            entries.push((
                keyword_form("capabilities"),
                Form::Vector(
                    self.capabilities
                        .iter()
                        .map(|capability| keyword_form(capability))
                        .collect(),
                ),
            ));
        }
        Form::Map(entries)
    }
}

fn interface_payload<'a>(form: &'a Form, origin: &str) -> Result<&'a Form, String> {
    match form {
        Form::Map(_) => Ok(form),
        Form::List(values)
            if values.len() == 2
                && matches!(&values[0], Form::Symbol(name) if name == "wasm/interface") =>
        {
            Ok(&values[1])
        }
        Form::List(_) => Err(malformed(
            origin,
            "interface must use exactly (wasm/interface {...})",
        )),
        _ => Err(malformed(
            origin,
            "interface must be a map or (wasm/interface {...}) data form",
        )),
    }
}

fn parse_memory(form: &Form, origin: &str) -> Result<MemoryContract, String> {
    let entries = map(form, origin, "memory")?;
    reject_unknown(entries, MEMORY_FIELDS, origin, "memory")?;
    Ok(MemoryContract {
        export: non_empty_string(
            required(entries, "export", origin)?,
            origin,
            "memory export",
        )?
        .to_owned(),
        allocate: optional_string(entries, "allocate", origin)?,
        reallocate: optional_string(entries, "reallocate", origin)?,
        release: optional_string(entries, "release", origin)?,
    })
}

fn parse_exports(form: &Form, origin: &str) -> Result<Vec<BindingFunction>, String> {
    let entries = map(form, origin, "exports")?;
    if entries.is_empty() {
        return Err(malformed(origin, "exports cannot be empty"));
    }

    let mut names = HashSet::new();
    let mut exports = entries
        .iter()
        .map(|(name, specification)| {
            let name = named(name, origin, "export name")?.to_owned();
            if !valid_binding_name(&name) {
                return Err(malformed(
                    origin,
                    format!("invalid Hara export name {name}"),
                ));
            }
            if !names.insert(name.clone()) {
                return Err(malformed(origin, format!("duplicate export {name}")));
            }
            parse_export(&name, specification, origin)
        })
        .collect::<Result<Vec<_>, _>>()?;
    exports.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(exports)
}

fn parse_export(name: &str, form: &Form, origin: &str) -> Result<BindingFunction, String> {
    let entries = map(form, origin, &format!("export {name}"))?;
    reject_unknown(entries, EXPORT_FIELDS, origin, &format!("export {name}"))?;

    let wasm_export = non_empty_string(
        required(entries, "wasm/export", origin)?,
        origin,
        &format!("export {name} wasm/export"),
    )?
    .to_owned();
    let arguments = parse_parameters(
        required(entries, "arguments", origin)?,
        origin,
        name,
    )?;
    let returns = parse_result(required(entries, "returns", origin)?, origin, name)?;
    let asynchronous = optional_bool(entries, "async", origin)?.unwrap_or(false);
    let errors = optional(entries, "errors")
        .map(|form| parse_errors(form, origin, name))
        .transpose()?;
    let capabilities = optional(entries, "capabilities")
        .map_or_else(|| Ok(BTreeSet::new()), |form| {
            keyword_set(form, origin, &format!("export {name} capabilities"))
        })?;

    Ok(BindingFunction {
        name: name.to_owned(),
        wasm_export,
        arguments,
        returns,
        asynchronous,
        errors,
        capabilities,
    })
}

fn parse_parameters(
    form: &Form,
    origin: &str,
    export: &str,
) -> Result<Vec<BindingParameter>, String> {
    let values = vector(form, origin, &format!("export {export} arguments"))?;
    let mut names = HashSet::new();
    values
        .iter()
        .map(|form| {
            let entries = map(form, origin, &format!("export {export} argument"))?;
            reject_unknown(
                entries,
                PARAMETER_FIELDS,
                origin,
                &format!("export {export} argument"),
            )?;
            let name = named(
                required(entries, "name", origin)?,
                origin,
                &format!("export {export} argument name"),
            )?
            .to_owned();
            if !valid_binding_name(&name) {
                return Err(malformed(
                    origin,
                    format!("invalid argument name {name} in export {export}"),
                ));
            }
            if !names.insert(name.clone()) {
                return Err(malformed(
                    origin,
                    format!("duplicate argument {name} in export {export}"),
                ));
            }

            Ok(BindingParameter {
                name,
                hara_type: HaraValueType::parse(
                    required(entries, "hara/type", origin)?,
                    origin,
                    &format!("export {export} argument hara/type"),
                )?,
                wasm_type: WasmValueType::parse(
                    required(entries, "wasm/type", origin)?,
                    origin,
                    &format!("export {export} argument wasm/type"),
                )?,
                lowering: optional(entries, "lower")
                    .map(|form| {
                        Lowering::parse(
                            form,
                            origin,
                            &format!("export {export} argument lower"),
                        )
                    })
                    .transpose()?,
                ownership: optional(entries, "ownership")
                    .map(|form| {
                        Ownership::parse(
                            form,
                            origin,
                            &format!("export {export} argument ownership"),
                        )
                    })
                    .transpose()?,
            })
        })
        .collect()
}

fn parse_result(form: &Form, origin: &str, export: &str) -> Result<BindingResult, String> {
    let entries = map(form, origin, &format!("export {export} result"))?;
    reject_unknown(
        entries,
        RESULT_FIELDS,
        origin,
        &format!("export {export} result"),
    )?;

    Ok(BindingResult {
        hara_type: HaraValueType::parse(
            required(entries, "hara/type", origin)?,
            origin,
            &format!("export {export} result hara/type"),
        )?,
        wasm_type: WasmValueType::parse(
            required(entries, "wasm/type", origin)?,
            origin,
            &format!("export {export} result wasm/type"),
        )?,
        lifting: optional(entries, "lift")
            .map(|form| {
                Lifting::parse(
                    form,
                    origin,
                    &format!("export {export} result lift"),
                )
            })
            .transpose()?,
        ownership: optional(entries, "ownership")
            .map(|form| {
                Ownership::parse(
                    form,
                    origin,
                    &format!("export {export} result ownership"),
                )
            })
            .transpose()?,
    })
}

fn parse_errors(form: &Form, origin: &str, export: &str) -> Result<ErrorContract, String> {
    let entries = map(form, origin, &format!("export {export} errors"))?;
    reject_unknown(
        entries,
        ERROR_FIELDS,
        origin,
        &format!("export {export} errors"),
    )?;
    let convention = keyword(
        required(entries, "convention", origin)?,
        origin,
        &format!("export {export} error convention"),
    )?
    .to_owned();
    let code_entries = map(
        required(entries, "codes", origin)?,
        origin,
        &format!("export {export} error codes"),
    )?;
    let mut codes = BTreeMap::new();
    for (code, value) in code_entries {
        let Form::Number(code) = code else {
            return Err(malformed(
                origin,
                format!("export {export} error codes require integer keys"),
            ));
        };
        let value = named(value, origin, &format!("export {export} error code"))?.to_owned();
        if codes.insert(*code, value).is_some() {
            return Err(malformed(
                origin,
                format!("duplicate error code {code} in export {export}"),
            ));
        }
    }
    Ok(ErrorContract { convention, codes })
}

fn validate_parameter(
    parameter: &BindingParameter,
    origin: &str,
    export: &str,
) -> Result<(), String> {
    if parameter.wasm_type == WasmValueType::Void {
        return Err(malformed(
            origin,
            format!("export {export} argument {} cannot be :void", parameter.name),
        ));
    }

    match parameter.hara_type.direct_wasm_type() {
        Some(expected) if expected == parameter.wasm_type => {
            if parameter.lowering.is_some() || parameter.ownership.is_some() {
                return Err(malformed(
                    origin,
                    format!(
                        "scalar argument {} in export {export} cannot declare lowering or ownership",
                        parameter.name
                    ),
                ));
            }
            Ok(())
        }
        Some(expected) => Err(malformed(
            origin,
            format!(
                "export {export} argument {} maps :{} to :{}",
                parameter.name,
                expected.as_keyword(),
                parameter.wasm_type.as_keyword()
            ),
        )),
        None => {
            if parameter.lowering.is_none() {
                return Err(malformed(
                    origin,
                    format!(
                        "non-scalar argument {} in export {export} requires :lower",
                        parameter.name
                    ),
                ));
            }
            if parameter.ownership.is_none() {
                return Err(malformed(
                    origin,
                    format!(
                        "non-scalar argument {} in export {export} requires :ownership",
                        parameter.name
                    ),
                ));
            }
            Err(unsupported(
                origin,
                format!(
                    "non-scalar argument {} in export {export} is reserved for memory bindings",
                    parameter.name
                ),
            ))
        }
    }
}

fn validate_result(result: &BindingResult, origin: &str, export: &str) -> Result<(), String> {
    match result.hara_type.direct_wasm_type() {
        Some(expected) if expected == result.wasm_type => {
            if result.lifting.is_some() || result.ownership.is_some() {
                return Err(malformed(
                    origin,
                    format!("scalar result in export {export} cannot declare lifting or ownership"),
                ));
            }
            Ok(())
        }
        Some(expected) => Err(malformed(
            origin,
            format!(
                "export {export} result maps :{} to :{}",
                expected.as_keyword(),
                result.wasm_type.as_keyword()
            ),
        )),
        None => {
            if result.lifting.is_none() {
                return Err(malformed(
                    origin,
                    format!("non-scalar result in export {export} requires :lift"),
                ));
            }
            if result.ownership.is_none() {
                return Err(malformed(
                    origin,
                    format!("non-scalar result in export {export} requires :ownership"),
                ));
            }
            Err(unsupported(
                origin,
                format!("non-scalar result in export {export} is reserved for memory bindings"),
            ))
        }
    }
}

fn reject_reserved_collection(
    entries: &[(Form, Form)],
    field: &str,
    origin: &str,
) -> Result<(), String> {
    let Some(form) = optional(entries, field) else {
        return Ok(());
    };
    let empty = matches!(form, Form::Vector(values) if values.is_empty())
        || matches!(form, Form::Map(values) if values.is_empty());
    if empty {
        Ok(())
    } else {
        Err(unsupported(
            origin,
            format!("{field} are reserved for the HTA binding tranche"),
        ))
    }
}

fn memory_form(memory: &MemoryContract) -> Form {
    let mut entries = vec![(
        keyword_form("export"),
        string_form(&memory.export),
    )];
    push_optional_string(&mut entries, "allocate", memory.allocate.as_deref());
    push_optional_string(
        &mut entries,
        "reallocate",
        memory.reallocate.as_deref(),
    );
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
        (
            keyword_form("hara/type"),
            result.hara_type.canonical_form(),
        ),
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
        (
            keyword_form("convention"),
            keyword_form(&errors.convention),
        ),
        (
            keyword_form("codes"),
            Form::Map(
                errors
                    .codes
                    .iter()
                    .map(|(code, value)| {
                        (Form::Number(*code), keyword_form(value))
                    })
                    .collect(),
            ),
        ),
    ])
}

fn named_type_form(kind: &str, name: &str) -> Form {
    Form::Vector(vec![keyword_form(kind), symbol_form(name)])
}

fn map<'a>(form: &'a Form, origin: &str, field: &str) -> Result<&'a [(Form, Form)], String> {
    match form {
        Form::Map(entries) => Ok(entries),
        _ => Err(malformed(origin, format!("{field} must be a map"))),
    }
}

fn vector<'a>(form: &'a Form, origin: &str, field: &str) -> Result<&'a [Form], String> {
    match form {
        Form::Vector(values) => Ok(values),
        _ => Err(malformed(origin, format!("{field} must be a vector"))),
    }
}

fn non_empty_string<'a>(
    form: &'a Form,
    origin: &str,
    field: &str,
) -> Result<&'a str, String> {
    match form {
        Form::String(value) if !value.is_empty() => Ok(value),
        _ => Err(malformed(
            origin,
            format!("{field} must be a non-empty string"),
        )),
    }
}

fn named<'a>(form: &'a Form, origin: &str, field: &str) -> Result<&'a str, String> {
    match form {
        Form::Symbol(value) | Form::Keyword(value) | Form::String(value) if !value.is_empty() => {
            Ok(value)
        }
        _ => Err(malformed(
            origin,
            format!("{field} must be a named value"),
        )),
    }
}

fn keyword<'a>(form: &'a Form, origin: &str, field: &str) -> Result<&'a str, String> {
    match form {
        Form::Keyword(value) => Ok(value),
        _ => Err(malformed(origin, format!("{field} must be a keyword"))),
    }
}

fn optional_string(
    entries: &[(Form, Form)],
    name: &str,
    origin: &str,
) -> Result<Option<String>, String> {
    optional(entries, name)
        .map(|form| non_empty_string(form, origin, name).map(str::to_owned))
        .transpose()
}

fn optional_bool(
    entries: &[(Form, Form)],
    name: &str,
    origin: &str,
) -> Result<Option<bool>, String> {
    optional(entries, name)
        .map(|form| match form {
            Form::Bool(value) => Ok(*value),
            _ => Err(malformed(origin, format!("{name} must be boolean"))),
        })
        .transpose()
}

fn keyword_set(form: &Form, origin: &str, field: &str) -> Result<BTreeSet<String>, String> {
    vector(form, origin, field)?
        .iter()
        .map(|form| keyword(form, origin, field).map(str::to_owned))
        .collect()
}

fn key(form: &Form) -> Option<&str> {
    match form {
        Form::Keyword(value) | Form::Symbol(value) | Form::String(value) => Some(value),
        _ => None,
    }
}

fn required<'a>(entries: &'a [(Form, Form)], name: &str, origin: &str) -> Result<&'a Form, String> {
    optional(entries, name)
        .ok_or_else(|| malformed(origin, format!("missing required field {name}")))
}

fn optional<'a>(entries: &'a [(Form, Form)], name: &str) -> Option<&'a Form> {
    entries
        .iter()
        .find(|(candidate, _)| key(candidate) == Some(name))
        .map(|(_, value)| value)
}

fn reject_unknown(
    entries: &[(Form, Form)],
    allowed: &[&str],
    origin: &str,
    scope: &str,
) -> Result<(), String> {
    let mut seen = HashSet::new();
    for (candidate, _) in entries {
        let Some(name) = key(candidate) else {
            return Err(malformed(origin, format!("{scope} keys must be named")));
        };
        if !allowed.contains(&name) {
            return Err(malformed(
                origin,
                format!("unknown {scope} field: {name}"),
            ));
        }
        if !seen.insert(name) {
            return Err(malformed(
                origin,
                format!("duplicate {scope} field: {name}"),
            ));
        }
    }
    Ok(())
}

fn validate_module_path(value: &str, origin: &str) -> Result<(), String> {
    let unsafe_path = !value.ends_with(".wasm")
        || value.starts_with('/')
        || value.contains('\\')
        || value.contains(':')
        || value.bytes().any(|byte| byte == 0)
        || value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..");
    if unsafe_path {
        return Err(malformed(
            origin,
            "module must be a safe relative .wasm package path",
        ));
    }
    Ok(())
}

fn valid_namespace(value: &str) -> bool {
    value.contains('.') && value.split('.').all(valid_component)
}

fn valid_tag(value: &str) -> bool {
    value.split('.').all(valid_component)
}

fn valid_binding_name(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|ch| {
            ch.is_ascii_lowercase()
                || ch.is_ascii_digit()
                || matches!(ch, '-' | '?' | '!' | '*' | '+' | '<' | '>' | '=')
        })
}

fn valid_component(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn push_optional_string(
    entries: &mut Vec<(Form, Form)>,
    name: &str,
    value: Option<&str>,
) {
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

fn malformed(origin: &str, message: impl AsRef<str>) -> String {
    format!("wasm-interface/malformed {origin}: {}", message.as_ref())
}

fn unsupported(origin: &str, message: impl AsRef<str>) -> String {
    format!(
        "wasm-interface/feature-unsupported {origin}: {}",
        message.as_ref()
    )
}

#[cfg(test)]
mod tests {
    use super::{WasmInterface, WasmValueType};

    const SCALAR_INTERFACE: &str = r#"
      (wasm/interface
       {:schema "hara.wasm-interface/0-alpha"
        :namespace math.scalar
        :module "modules/math.wasm"
        :exports
        {add {:wasm/export "add_i64"
              :arguments [{:name left :hara/type :i64 :wasm/type :i64}
                          {:name right :hara/type :i64 :wasm/type :i64}]
              :returns {:hara/type :i64 :wasm/type :i64}}}})"#;

    #[test]
    fn parses_scalar_interface_without_evaluation() {
        let interface = WasmInterface::parse(SCALAR_INTERFACE, "fixture").unwrap();
        assert_eq!(interface.namespace, "math.scalar");
        assert_eq!(interface.module, "modules/math.wasm");
        assert_eq!(interface.exports[0].name, "add");
        assert_eq!(interface.exports[0].wasm_export, "add_i64");
        assert_eq!(
            interface.exports[0].arguments[0].wasm_type,
            WasmValueType::I64
        );
        assert_eq!(interface.direct_exports()[0].0, "add_i64");
        assert_eq!(interface.digest().len(), 71);
        assert!(interface.digest().starts_with("sha256:"));
        assert_eq!(
            WasmInterface::parse(&interface.canonical_source(), "canonical").unwrap(),
            interface
        );
    }

    #[test]
    fn canonicalizes_map_and_set_order() {
        let left = r#"
          {:schema "hara.wasm-interface/0-alpha"
           :namespace math.scalar
           :module "modules/math.wasm"
           :capabilities [:random :clock]
           :exports
           {subtract {:wasm/export "sub"
                      :arguments [{:name left :hara/type :i64 :wasm/type :i64}
                                  {:name right :hara/type :i64 :wasm/type :i64}]
                      :returns {:hara/type :i64 :wasm/type :i64}}
            add {:wasm/export "add_i64"
                 :arguments [{:name left :hara/type :i64 :wasm/type :i64}
                             {:name right :hara/type :i64 :wasm/type :i64}]
                 :returns {:hara/type :i64 :wasm/type :i64}
                 :capabilities [:clock :random]}}}"#;
        let right = r#"
          {:exports
           {add {:capabilities [:random :clock]
                 :returns {:wasm/type :i64 :hara/type :i64}
                 :arguments [{:wasm/type :i64 :hara/type :i64 :name left}
                             {:hara/type :i64 :name right :wasm/type :i64}]
                 :wasm/export "add_i64"}
            subtract {:returns {:wasm/type :i64 :hara/type :i64}
                      :wasm/export "sub"
                      :arguments [{:wasm/type :i64 :name left :hara/type :i64}
                                  {:name right :wasm/type :i64 :hara/type :i64}]}}
           :module "modules/math.wasm"
           :capabilities [:clock :random]
           :namespace math.scalar
           :schema "hara.wasm-interface/0-alpha"}"#;
        let left = WasmInterface::parse(left, "left").unwrap();
        let right = WasmInterface::parse(right, "right").unwrap();
        assert_eq!(left, right);
        assert_eq!(left.canonical_source(), right.canonical_source());
        assert_eq!(left.digest(), right.digest());
    }

    #[test]
    fn rejects_executable_unknown_duplicate_and_unsafe_sources() {
        for source in [
            "(do (println \"not data\"))".to_owned(),
            SCALAR_INTERFACE.replace(
                ":module \"modules/math.wasm\"",
                ":module \"../math.wasm\"",
            ),
            SCALAR_INTERFACE.replace(
                ":schema \"hara.wasm-interface/0-alpha\"",
                ":schema \"hara.wasm-interface/9\"",
            ),
            SCALAR_INTERFACE.replace(
                ":namespace math.scalar",
                ":namespace Math.scalar",
            ),
            SCALAR_INTERFACE.replace(
                ":exports",
                ":unknown true :exports",
            ),
            SCALAR_INTERFACE.replace(
                ":name left",
                ":name left :name duplicate",
            ),
        ] {
            let error = WasmInterface::parse(&source, "fixture").unwrap_err();
            assert!(error.starts_with("wasm-interface/"));
        }
    }

    #[test]
    fn rejects_ambiguous_and_future_semantics() {
        let mismatch = SCALAR_INTERFACE.replace(
            ":name left :hara/type :i64 :wasm/type :i64",
            ":name left :hara/type :i32 :wasm/type :i64",
        );
        assert!(WasmInterface::parse(&mismatch, "mismatch")
            .unwrap_err()
            .contains("maps :i32 to :i64"));

        let missing_ownership = SCALAR_INTERFACE.replace(
            ":name left :hara/type :i64 :wasm/type :i64",
            ":name left :hara/type :bytes :wasm/type :i32 :lower [:pointer :length]",
        );
        assert!(WasmInterface::parse(&missing_ownership, "bytes")
            .unwrap_err()
            .contains("requires :ownership"));

        let asynchronous = SCALAR_INTERFACE.replace(
            ":returns {:hara/type :i64 :wasm/type :i64}",
            ":returns {:hara/type :i64 :wasm/type :i64} :async true",
        );
        assert!(WasmInterface::parse(&asynchronous, "async")
            .unwrap_err()
            .starts_with("wasm-interface/feature-unsupported"));

        let handles = SCALAR_INTERFACE.replace(
            ":exports",
            ":handles {stream {:tag stream :release \"stream_drop\"}} :exports",
        );
        assert!(WasmInterface::parse(&handles, "handles")
            .unwrap_err()
            .starts_with("wasm-interface/feature-unsupported"));
    }
}
