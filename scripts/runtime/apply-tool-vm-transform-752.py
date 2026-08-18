#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    if text.count(old) != 1:
        raise SystemExit(f"{path}: expected one replacement, found {text.count(old)}")
    path.write_text(text.replace(old, new, 1))


# Compiler tooling is an explicit profile. hara-vm continues to request only
# bytecode-vm, so it receives the read-only provider and no transformation edges.
cargo = ROOT / "core/rust/Cargo.toml"
replace_once(
    cargo,
    'default = ["bytecode-vm"]\n',
    'default = ["bytecode-vm", "tool-vm-transform"]\n',
)
replace_once(
    cargo,
    '# Existing optional encoder gate (declared here so Cargo can validate it).\nhalc-encoder = []\n',
    '# Existing optional encoder gate (declared here so Cargo can validate it).\nhalc-encoder = []\n'
    '# Public compiler tooling profile. VM-only consumers intentionally omit it.\n'
    'tool-vm-transform = ["bytecode-vm", "halc-encoder"]\n',
)

(ROOT / "core/rust/src/core/vm_tool.rs").write_text(r'''fn vm_tool_keyword(name: &str) -> Value {
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

fn vm_tool_transform_pair(from: &str, to: &str) -> Value {
    vm_tool_vector([vm_tool_keyword(from), vm_tool_keyword(to)])
}

fn vm_tool_provider_descriptor() -> Value {
    #[cfg(all(feature = "bytecode-vm", feature = "tool-vm-transform"))]
    let operations = &["validate", "inspect", "transform", "disassemble"][..];
    #[cfg(all(feature = "bytecode-vm", not(feature = "tool-vm-transform")))]
    let operations = &["validate", "inspect", "disassemble"][..];
    #[cfg(not(feature = "bytecode-vm"))]
    let operations = &["validate", "inspect"][..];

    #[cfg(all(feature = "bytecode-vm", feature = "tool-vm-transform"))]
    let formats = vec![
        (
            vm_tool_keyword("hal"),
            vm_tool_vector(std::iter::empty::<Value>()),
        ),
        (
            vm_tool_keyword("halc"),
            vm_tool_keywords(&["validate", "inspect"]),
        ),
        (
            vm_tool_keyword("hbc"),
            vm_tool_keywords(&["validate", "inspect", "disassemble"]),
        ),
    ];
    #[cfg(all(feature = "bytecode-vm", not(feature = "tool-vm-transform")))]
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

    #[cfg(feature = "tool-vm-transform")]
    let transforms = vm_tool_vector([
        vm_tool_transform_pair("hal", "halc"),
        vm_tool_transform_pair("hal", "hbc"),
        vm_tool_transform_pair("halc", "hbc"),
    ]);
    #[cfg(not(feature = "tool-vm-transform"))]
    let transforms = vm_tool_vector(std::iter::empty::<Value>());

    vm_tool_map([
        (vm_tool_keyword("provider/id"), vm_tool_keyword("rust")),
        (
            vm_tool_keyword("provider/operations"),
            vm_tool_keywords(operations),
        ),
        (vm_tool_keyword("provider/formats"), vm_tool_map(formats)),
        (vm_tool_keyword("provider/transforms"), transforms),
        (
            vm_tool_keyword("provider/engines"),
            vm_tool_map(std::iter::empty::<(Value, Value)>()),
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

fn vm_tool_transform_format<'a>(value: &'a Value) -> Result<&'a str, String> {
    match value {
        Value::Keyword(format)
            if format.get_namespace().is_none()
                && matches!(format.get_name(), "hal" | "halc" | "hbc") =>
        {
            Ok(format.get_name())
        }
        _ => Err("tool.vm.provider/transform expects :hal, :halc, or :hbc formats".into()),
    }
}

fn vm_tool_bytes(value: &Value, operation: &str) -> Result<Vec<u8>, String> {
    match value {
        Value::Bytes(bytes) => Ok(bytes.clone()),
        Value::ByteBuffer(bytes) => Ok(bytes.borrow().clone()),
        _ => Err(format!("tool.vm.provider/{operation} expects Bytes")),
    }
}

fn vm_tool_source<'a>(value: &'a Value) -> Result<&'a str, String> {
    match value {
        Value::String(source) => Ok(source),
        _ => Err("tool.vm.provider/transform expects HAL input as a string".into()),
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

fn vm_tool_declared_namespace(forms: &[Form]) -> Result<Option<String>, String> {
    for form in forms {
        let Form::List(items) = form_without_metadata(form) else {
            continue;
        };
        if !matches!(items.first(), Some(Form::Symbol(operator)) if operator == "ns") {
            continue;
        }
        return match items.get(1) {
            Some(Form::Symbol(namespace)) if !namespace.contains('/') => {
                Ok(Some(namespace.clone()))
            }
            _ => Err("HAL source has an invalid ns declaration".into()),
        };
    }
    Ok(None)
}

fn vm_tool_resource_option(options: &Value) -> Result<Option<String>, String> {
    let entries = map_entries(options)
        .ok_or_else(|| "tool.vm.provider/transform expects an options map".to_string())?;
    let mut resource = None;
    for (key, value) in entries {
        match key {
            Value::Keyword(key)
                if key.get_namespace().is_none() && key.get_name() == "resource" =>
            {
                let Value::String(value) = value else {
                    return Err("tool.vm.provider/transform :resource must be a string".into());
                };
                resource = Some(value);
            }
            Value::Keyword(key) => {
                return Err(format!(
                    "tool.vm.provider/transform does not support option :{}",
                    key.get_name()
                ));
            }
            _ => {
                return Err(
                    "tool.vm.provider/transform options must use unqualified keyword keys".into(),
                );
            }
        }
    }
    Ok(resource)
}

fn vm_tool_default_resource(namespace: &str) -> String {
    format!("{}.hal", namespace.replace('.', "/"))
}

#[cfg(feature = "tool-vm-transform")]
fn vm_tool_encode_halc(source: &str, resource: Option<&str>) -> Result<Vec<u8>, String> {
    let forms = crate::kernel::parse_forms(source)?;
    let namespace = vm_tool_declared_namespace(&forms)?
        .ok_or_else(|| "HAL -> HALC requires a top-level ns declaration".to_string())?;
    let resource = resource
        .map(str::to_owned)
        .unwrap_or_else(|| vm_tool_default_resource(&namespace));
    crate::kernel::halc::encode_halc_module(&namespace, &resource, source, forms)
}

#[cfg(feature = "tool-vm-transform")]
fn vm_tool_compile_hbc_source(source: &str, resource: Option<&str>) -> Result<Vec<u8>, String> {
    let forms = crate::kernel::parse_forms(source)?;
    if let Some(namespace) = vm_tool_declared_namespace(&forms)? {
        let resource = resource
            .map(str::to_owned)
            .unwrap_or_else(|| vm_tool_default_resource(&namespace));
        let halc =
            crate::kernel::halc::encode_halc_module(&namespace, &resource, source, forms)?;
        let mut runtime = crate::Runtime::new();
        runtime.compile_halc_bytecode_artifact(&halc)
    } else {
        if resource.is_some() {
            return Err(
                "tool.vm.provider/transform :resource requires HAL source with an ns declaration"
                    .into(),
            );
        }
        crate::Runtime::new().compile_bytecode_artifact(source)
    }
}

#[cfg(feature = "tool-vm-transform")]
fn vm_tool_artifact(format: &str, version: i64, bytes: Vec<u8>) -> Value {
    vm_tool_map([
        (
            vm_tool_keyword("artifact/format"),
            vm_tool_keyword(format),
        ),
        (
            vm_tool_keyword("artifact/version"),
            Value::Number(version),
        ),
        (vm_tool_keyword("artifact/bytes"), Value::Bytes(bytes)),
    ])
}

fn vm_tool_transform(
    from: &str,
    to: &str,
    input: &Value,
    options: &Value,
) -> Result<Value, String> {
    #[cfg(feature = "tool-vm-transform")]
    {
        let resource = vm_tool_resource_option(options)?;
        match (from, to) {
            ("hal", "halc") => {
                let source = vm_tool_source(input)?;
                vm_tool_encode_halc(source, resource.as_deref())
                    .map(|bytes| vm_tool_artifact("halc", 1, bytes))
            }
            ("hal", "hbc") => {
                let source = vm_tool_source(input)?;
                vm_tool_compile_hbc_source(source, resource.as_deref())
                    .map(|bytes| vm_tool_artifact("hbc", 0, bytes))
            }
            ("halc", "hbc") => {
                if resource.is_some() {
                    return Err(
                        "tool.vm.provider/transform :resource is only valid for HAL input".into(),
                    );
                }
                let bytes = vm_tool_bytes(input, "transform")?;
                let mut runtime = crate::Runtime::new();
                runtime
                    .compile_halc_bytecode_artifact(&bytes)
                    .map(|bytes| vm_tool_artifact("hbc", 0, bytes))
            }
            _ => Err(format!(
                "tool.vm.provider/transform does not support :{from} -> :{to}"
            )),
        }
    }
    #[cfg(not(feature = "tool-vm-transform"))]
    {
        let _ = (from, to, input, options);
        Err("tool.vm.provider transform capability is unavailable in this runtime profile".into())
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
            "transform",
            native_function("tool.vm.provider/transform", 4, |arguments| {
                let from = vm_tool_transform_format(&arguments[0])?;
                let to = vm_tool_transform_format(&arguments[1])?;
                vm_tool_transform(from, to, &arguments[2], &arguments[3])
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

    fn artifact_bytes(value: &Value) -> Vec<u8> {
        match field(value, "artifact/bytes") {
            Value::Bytes(bytes) => bytes,
            other => panic!("expected artifact Bytes, got {other:?}"),
        }
    }

    #[test]
    fn provider_reports_exact_profile_capabilities() {
        let provider = vm_tool_provider_descriptor();
        assert_eq!(field(&provider, "provider/id").display(), ":rust");
        #[cfg(all(feature = "bytecode-vm", feature = "tool-vm-transform"))]
        {
            assert_eq!(
                field(&provider, "provider/operations").display(),
                "[:validate :inspect :transform :disassemble]"
            );
            assert_eq!(
                field(&provider, "provider/transforms").display(),
                "[[:hal :halc] [:hal :hbc] [:halc :hbc]]"
            );
        }
        #[cfg(all(feature = "bytecode-vm", not(feature = "tool-vm-transform")))]
        {
            assert_eq!(
                field(&provider, "provider/operations").display(),
                "[:validate :inspect :disassemble]"
            );
            assert_eq!(field(&provider, "provider/transforms").display(), "[]");
        }
        #[cfg(not(feature = "bytecode-vm"))]
        {
            assert_eq!(
                field(&provider, "provider/operations").display(),
                "[:validate :inspect]"
            );
            assert_eq!(field(&provider, "provider/transforms").display(), "[]");
        }
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

    #[cfg(feature = "tool-vm-transform")]
    #[test]
    fn transformations_use_canonical_halc_and_hbc_compilers() {
        let source = "(ns sample.vm) (def answer (+ 19 23)) answer";
        let options = vm_tool_map([(
            vm_tool_keyword("resource"),
            Value::String("sample/vm.hal".into()),
        )]);
        let halc = vm_tool_transform(
            "hal",
            "halc",
            &Value::String(source.into()),
            &options,
        )
        .unwrap();
        let halc_again = vm_tool_transform(
            "hal",
            "halc",
            &Value::String(source.into()),
            &options,
        )
        .unwrap();
        let halc_bytes = artifact_bytes(&halc);
        assert_eq!(halc_bytes, artifact_bytes(&halc_again));
        vm_tool_validate("halc", &halc_bytes).unwrap();

        let hbc_from_source = vm_tool_transform(
            "hal",
            "hbc",
            &Value::String(source.into()),
            &options,
        )
        .unwrap();
        let hbc_from_halc = vm_tool_transform(
            "halc",
            "hbc",
            &Value::Bytes(halc_bytes),
            &vm_tool_map(std::iter::empty::<(Value, Value)>()),
        )
        .unwrap();
        let source_bytes = artifact_bytes(&hbc_from_source);
        let halc_bytes = artifact_bytes(&hbc_from_halc);
        assert_eq!(source_bytes, halc_bytes);
        vm_tool_validate("hbc", &source_bytes).unwrap();
    }
}
''')

(ROOT / "core/java/src/main/java/hara/truffle/ToolVmLibrary.java").write_text(r'''package hara.truffle;

import hara.lang.data.Keyword;
import hara.lang.data.types.IMapType;
import hara.truffle.bytecode.HbcCodec;
import hara.truffle.bytecode.HbcDisassembler;
import hara.truffle.bytecode.HbcProgram;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.Map.Entry;

/** Portable HALC/HBC tooling provider implementation for the Truffle runtime. */
public final class ToolVmLibrary {
  private static final Keyword HAL = Keyword.create("hal");
  private static final Keyword HALC = Keyword.create("halc");
  private static final Keyword HBC = Keyword.create("hbc");

  private ToolVmLibrary() {}

  @HaraExport(
      name = "provider",
      doc = "Returns the exact VM tooling capabilities of the Truffle runtime.",
      arglists = {"[]"})
  public static Object provider(HaraContext context, Object[] arguments) {
    expectArity("provider", arguments, 0);
    return orderedMap(
        "provider/id", keyword("truffle"),
        "provider/operations", keywords("validate", "inspect", "transform", "disassemble"),
        "provider/formats", orderedMap(
            "hal", vector(),
            "halc", keywords("validate", "inspect"),
            "hbc", keywords("validate", "inspect", "disassemble")),
        "provider/transforms", vector(vector(HAL, HALC)),
        "provider/engines", orderedMap());
  }

  @HaraExport(
      name = "validate",
      doc = "Authenticates and validates canonical HALC or HBC bytes.",
      arglists = {"[format bytes]"})
  public static Object validate(HaraContext context, Object[] arguments) {
    expectArity("validate", arguments, 2);
    String format = format(arguments[0], "validate");
    byte[] bytes = bytes(arguments[1], "validate");
    switch (format) {
      case "halc" -> HalcArtifact.decode(bytes);
      case "hbc" -> HbcCodec.decode(bytes);
      default -> throw unsupported(format, "validate");
    }
    return Boolean.TRUE;
  }

  @HaraExport(
      name = "inspect",
      doc = "Returns ordinary Hara metadata derived from a validated HALC or HBC artifact.",
      arglists = {"[format bytes]"})
  public static Object inspect(HaraContext context, Object[] arguments) {
    expectArity("inspect", arguments, 2);
    String format = format(arguments[0], "inspect");
    byte[] bytes = bytes(arguments[1], "inspect");
    return switch (format) {
      case "halc" -> inspectHalc(bytes);
      case "hbc" -> inspectHbc(bytes);
      default -> throw unsupported(format, "inspect");
    };
  }

  @HaraExport(
      name = "transform",
      doc = "Transforms one provider-declared portable code format into another.",
      arglists = {"[from to input options]"})
  public static Object transform(HaraContext context, Object[] arguments) {
    expectArity("transform", arguments, 4);
    String from = transformFormat(arguments[0]);
    String to = transformFormat(arguments[1]);
    if (!"hal".equals(from) || !"halc".equals(to)) {
      throw new HaraException(
          "tool.vm.provider/transform does not support :" + from + " -> :" + to);
    }
    String source = source(arguments[2]);
    Object[] forms = HaraLanguage.readAll(source, null);
    String namespace = HalcArtifact.declaredNamespace(forms);
    String resource = transformResource(arguments[3], namespace);
    byte[] artifact =
        HalcArtifact.encode(
            namespace, resource, source.getBytes(StandardCharsets.UTF_8), forms);
    return orderedMap(
        "artifact/format", HALC,
        "artifact/version", 1L,
        "artifact/bytes", artifact);
  }

  @HaraExport(
      name = "disassemble",
      doc = "Returns deterministic HBC diagnostics; this is not source decompilation.",
      arglists = {"[bytes]"})
  public static Object disassemble(HaraContext context, Object[] arguments) {
    expectArity("disassemble", arguments, 1);
    byte[] bytes = bytes(arguments[0], "disassemble");
    return HbcDisassembler.disassemble(HbcCodec.decode(bytes));
  }

  private static Object inspectHalc(byte[] bytes) {
    HalcArtifact.Module module = HalcArtifact.decode(bytes);
    int payloadBytes = unsignedInt(bytes, 8, "HALC payload length");
    return orderedMap(
        "artifact/format", HALC,
        "artifact/version", 1L,
        "artifact/origin", keyword(module.origin == HalcArtifact.Origin.HALC ? "halc" : "legacy-hir"),
        "artifact/bytes", (long) bytes.length,
        "payload/bytes", (long) payloadBytes,
        "payload/checksum", Arrays.copyOfRange(bytes, 12, 44),
        "module/namespace", module.namespace,
        "module/resource", module.resource,
        "source/hash", module.sourceHash.clone(),
        "forms/count", (long) module.forms.length,
        "schemas/definitions", sortedStrings(module.schemas.definitions.keySet()),
        "schemas/functions", sortedStrings(module.schemas.functions.keySet()));
  }

  private static Object inspectHbc(byte[] bytes) {
    HbcProgram program = HbcCodec.decode(bytes);
    int payloadBytes = unsignedInt(bytes, 4, "HBC payload length");
    long instructions = program.functions().stream().mapToLong(function -> function.code().size()).sum();
    long handlers = program.functions().stream().mapToLong(function -> function.handlers().size()).sum();
    return orderedMap(
        "artifact/format", HBC,
        "artifact/version", 0L,
        "artifact/bytes", (long) bytes.length,
        "payload/bytes", (long) payloadBytes,
        "payload/checksum", Arrays.copyOfRange(bytes, 8 + payloadBytes, bytes.length),
        "module/namespace", program.namespace() == null ? HaraNull.SINGLETON : program.namespace(),
        "program/entry", (long) program.entry(),
        "constants/count", (long) program.constants().size(),
        "functions/count", (long) program.functions().size(),
        "instructions/count", instructions,
        "handlers/count", handlers);
  }

  private static int unsignedInt(byte[] bytes, int offset, String field) {
    if (offset < 0 || bytes.length < offset + Integer.BYTES) {
      throw new HaraException("Invalid " + field + ": truncated artifact");
    }
    int value = ByteBuffer.wrap(bytes, offset, Integer.BYTES).order(ByteOrder.BIG_ENDIAN).getInt();
    if (value < 0) throw new HaraException("Invalid " + field + ": length overflow");
    return value;
  }

  private static String format(Object value, String operation) {
    Object unwrapped = HaraBox.unwrap(value);
    if (unwrapped instanceof Keyword keyword
        && keyword.getNamespace() == null
        && (keyword.equals(HALC) || keyword.equals(HBC))) {
      return keyword.getName();
    }
    throw new HaraException(
        "tool.vm.provider/" + operation + " expects :halc or :hbc as its format");
  }

  private static String transformFormat(Object value) {
    Object unwrapped = HaraBox.unwrap(value);
    if (unwrapped instanceof Keyword keyword
        && keyword.getNamespace() == null
        && (keyword.equals(HAL) || keyword.equals(HALC) || keyword.equals(HBC))) {
      return keyword.getName();
    }
    throw new HaraException(
        "tool.vm.provider/transform expects :hal, :halc, or :hbc formats");
  }

  private static String source(Object value) {
    Object unwrapped = HaraBox.unwrap(value);
    if (unwrapped instanceof String source) return source;
    throw new HaraException("tool.vm.provider/transform expects HAL input as a string");
  }

  private static String transformResource(Object value, String namespace) {
    Object unwrapped = HaraBox.unwrap(value);
    if (!(unwrapped instanceof IMapType<?, ?> options)) {
      throw new HaraException("tool.vm.provider/transform expects an options map");
    }
    String resource = null;
    for (Object item : options) {
      Entry<?, ?> entry = (Entry<?, ?>) item;
      if (!(entry.getKey() instanceof Keyword key) || key.getNamespace() != null) {
        throw new HaraException(
            "tool.vm.provider/transform options must use unqualified keyword keys");
      }
      if (!"resource".equals(key.getName())) {
        throw new HaraException(
            "tool.vm.provider/transform does not support option :" + key.getName());
      }
      if (!(HaraBox.unwrap(entry.getValue()) instanceof String selected)) {
        throw new HaraException("tool.vm.provider/transform :resource must be a string");
      }
      resource = selected;
    }
    return resource == null ? namespace.replace('.', '/') + ".hal" : resource;
  }

  private static byte[] bytes(Object value, String operation) {
    Object unwrapped = HaraBox.unwrap(value);
    if (unwrapped instanceof byte[] bytes) return bytes.clone();
    throw new HaraException("tool.vm.provider/" + operation + " expects Bytes");
  }

  private static HaraException unsupported(String format, String operation) {
    return new HaraException(
        "tool.vm.provider/" + operation + " does not support format :" + format);
  }

  private static void expectArity(String operation, Object[] arguments, int arity) {
    if (arguments.length != arity) {
      throw new HaraException(
          "tool.vm.provider/" + operation + " expects " + arity + " arguments");
    }
  }

  private static Keyword keyword(String value) {
    return Keyword.create(value);
  }

  private static Object vector(Object... values) {
    return hara.lang.data.Vector.Standard.from(null, values);
  }

  private static Object keywords(String... values) {
    Object[] keywords = new Object[values.length];
    for (int index = 0; index < values.length; index++) keywords[index] = keyword(values[index]);
    return vector(keywords);
  }

  private static Object sortedStrings(Iterable<String> values) {
    ArrayList<String> sorted = new ArrayList<>();
    for (String value : values) sorted.add(value);
    sorted.sort(Comparator.naturalOrder());
    return vector(sorted.toArray());
  }

  private static Object orderedMap(Object... entries) {
    if ((entries.length & 1) != 0) throw new IllegalArgumentException("ordered map requires pairs");
    Object[] values = new Object[entries.length];
    for (int index = 0; index < entries.length; index += 2) {
      values[index] = keyword((String) entries[index]);
      values[index + 1] = entries[index + 1];
    }
    return hara.lang.data.OrderedMap.Standard.from(null, values);
  }
}
''')

(ROOT / "core/java/src/test/java/hara/truffle/ToolVmLibraryTest.java").write_text(r'''package hara.truffle;

import static org.junit.Assert.assertArrayEquals;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

import hara.lang.data.Keyword;
import hara.lang.data.types.IMapType;
import hara.truffle.bytecode.HbcCodec;
import hara.truffle.bytecode.HbcProgram;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.ServiceLoader;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.StreamSupport;
import org.graalvm.polyglot.Context;
import org.junit.Test;

public class ToolVmLibraryTest {
  @Test
  public void providerIsDiscoverableAndPublicFacadeLoads() {
    Set<String> namespaces =
        StreamSupport.stream(ServiceLoader.load(HaraLibraryProvider.class).spliterator(), false)
            .map(HaraLibraryProvider::namespace)
            .collect(Collectors.toSet());
    assertTrue(namespaces.contains("tool.vm.provider"));

    try (Context context = Context.newBuilder(HaraLanguage.ID).build()) {
      assertEquals(
          ":truffle",
          context
              .eval(
                  HaraLanguage.ID,
                  "(ns tool.vm.provider-probe (:require [tool.vm :as vm])) "
                      + "(:provider/id (vm/current-provider))")
              .toString());
    }
  }

  @Test
  public void providerReportsOnlyTheImplementedTransformation() {
    @SuppressWarnings("unchecked")
    IMapType<Keyword, Object> provider =
        (IMapType<Keyword, Object>) ToolVmLibrary.provider(null, new Object[0]);
    assertEquals(
        "[:validate :inspect :transform :disassemble]",
        provider.lookup(Keyword.create("provider/operations")).toString());
    assertEquals(
        "[[:hal :halc]]",
        provider.lookup(Keyword.create("provider/transforms")).toString());
  }

  @Test
  public void halcValidationAndInspectionUseCanonicalCodec() {
    String source = "(ns sample.vm) (def value 42)";
    Object[] forms = HaraLanguage.readAll(source, "sample/vm.hal");
    byte[] artifact =
        HalcArtifact.encode(
            "sample.vm",
            "sample/vm.hal",
            source.getBytes(StandardCharsets.UTF_8),
            forms);

    assertEquals(
        Boolean.TRUE,
        ToolVmLibrary.validate(null, new Object[] {Keyword.create("halc"), artifact}));
    @SuppressWarnings("unchecked")
    IMapType<Keyword, Object> inspection =
        (IMapType<Keyword, Object>)
            ToolVmLibrary.inspect(null, new Object[] {Keyword.create("halc"), artifact});
    assertEquals(Keyword.create("halc"), inspection.lookup(Keyword.create("artifact/format")));
    assertEquals("sample.vm", inspection.lookup(Keyword.create("module/namespace")));
    assertEquals(2L, inspection.lookup(Keyword.create("forms/count")));
  }

  @Test
  public void halToHalcTransformUsesTheCanonicalEncoderDeterministically() {
    String source = "(ns sample.vm) (def value 42)";
    Object options =
        hara.lang.data.OrderedMap.Standard.from(
            null, new Object[] {Keyword.create("resource"), "sample/vm.hal"});
    @SuppressWarnings("unchecked")
    IMapType<Keyword, Object> first =
        (IMapType<Keyword, Object>)
            ToolVmLibrary.transform(
                null,
                new Object[] {
                  Keyword.create("hal"), Keyword.create("halc"), source, options
                });
    @SuppressWarnings("unchecked")
    IMapType<Keyword, Object> second =
        (IMapType<Keyword, Object>)
            ToolVmLibrary.transform(
                null,
                new Object[] {
                  Keyword.create("hal"), Keyword.create("halc"), source, options
                });
    byte[] actual = (byte[]) first.lookup(Keyword.create("artifact/bytes"));
    byte[] expected =
        HalcArtifact.encode(
            "sample.vm",
            "sample/vm.hal",
            source.getBytes(StandardCharsets.UTF_8),
            HaraLanguage.readAll(source, null));
    assertArrayEquals(expected, actual);
    assertArrayEquals(actual, (byte[]) second.lookup(Keyword.create("artifact/bytes")));
    assertEquals(
        Boolean.TRUE,
        ToolVmLibrary.validate(null, new Object[] {Keyword.create("halc"), actual}));

    HaraException unsupported =
        assertThrows(
            HaraException.class,
            () ->
                ToolVmLibrary.transform(
                    null,
                    new Object[] {
                      Keyword.create("hal"), Keyword.create("hbc"), source, options
                    }));
    assertTrue(unsupported.getMessage().contains(":hal -> :hbc"));
  }

  @Test
  public void hbcValidationInspectionAndDisassemblyUseCanonicalCodec() {
    HbcProgram.Function function =
        new HbcProgram.Function(
            "entry",
            false,
            0,
            false,
            0,
            0,
            2,
            List.of(
                new HbcProgram.Instruction(HbcProgram.Opcode.CONSTANT, 0, 0, 0),
                new HbcProgram.Instruction(HbcProgram.Opcode.CONSTANT, 1, 0, 0),
                new HbcProgram.Instruction(
                    HbcProgram.Opcode.PRIMITIVE, HbcProgram.Primitive.ADD.id(), 2, 0),
                new HbcProgram.Instruction(HbcProgram.Opcode.RETURN, 0, 0, 0)),
            java.util.Arrays.asList(null, null, null, null),
            List.of());
    HbcProgram program =
        new HbcProgram(List.of(19L, 23L), List.of(), List.of(function), 0);
    byte[] artifact = HbcCodec.encode(program);

    assertEquals(
        Boolean.TRUE,
        ToolVmLibrary.validate(null, new Object[] {Keyword.create("hbc"), artifact}));
    @SuppressWarnings("unchecked")
    IMapType<Keyword, Object> inspection =
        (IMapType<Keyword, Object>)
            ToolVmLibrary.inspect(null, new Object[] {Keyword.create("hbc"), artifact});
    assertEquals(Keyword.create("hbc"), inspection.lookup(Keyword.create("artifact/format")));
    assertEquals(1L, inspection.lookup(Keyword.create("functions/count")));
    assertTrue(
        ToolVmLibrary.disassemble(null, new Object[] {artifact})
            .toString()
            .startsWith("HBC0 entry="));
  }
}
''')

vm = ROOT / "core/lib/src/tool/vm.hal"
text = vm.read_text()
if "(defn transform\n" in text:
    raise SystemExit("tool.vm/transform already exists")
text += r'''

;; ---------------------------------------------------------------------------
;; Portable transformations
;; ---------------------------------------------------------------------------

(def +transform-option-keys+
  #{:provider :resource})

(defn- transform-options
  [value]
  (let [value (or value {})
        value (exact-map
               "tool.vm transform options"
               :tool.vm/invalid-request
               +transform-option-keys+
               value)]
    (if (and (has? value :resource)
             (not (string? (:resource value))))
      (fail "tool.vm transform :resource must be a string"
            {:type :tool.vm/invalid-request
             :operation :transform
             :field :resource
             :value (:resource value)})
      value)))

(defn- transform-routing-options
  [options]
  (if (has? options :provider)
    {:provider (:provider options)}
    {}))

(defn- transform-provider-options
  [options]
  (if (has? options :resource)
    {:resource (:resource options)}
    {}))

(defn- transform-input
  [from input]
  (cond
    (= from :hal)
    (if (string? input)
      {:input input
       :source-provenance nil}
      (fail "HAL transformation input must be a source string"
            {:type :tool.vm/invalid-transform-input
             :operation :transform
             :from from
             :value input}))

    (= from :halc)
    (let [input (artifact input)]
      (if (= :halc (:artifact/format input))
        {:input (:artifact/bytes input)
         :source-provenance (:artifact/provenance input)}
        (fail "HALC transformation input must be a HALC artifact"
              {:type :tool.vm/invalid-transform-input
               :operation :transform
               :from from
               :format (:artifact/format input)})))

    :else
    (fail "Unsupported transformation input format"
          {:type :tool.vm/invalid-transform-input
           :operation :transform
           :from from})))

(defn- transform-failed
  [from to provider error]
  (let [message (ex-message error)]
    (fail (str "tool.vm transformation failed: " message)
          {:type :tool.vm/transform-failed
           :operation :transform
           :from from
           :to to
           :provider (:provider/id provider)
           :message message})))

(defn transform
  "Transforms HAL/HALC through one explicitly selected current-runtime capability."
  ([from to input]
   (transform from to input {}))
  ([from to input options]
   (let [from (format-id from)
         to (format-id to)
         _ (transformation from to)
         options (transform-options options)
         _ (if (and (has? options :resource)
                    (not (= from :hal)))
             (fail "tool.vm transform :resource is only valid for HAL input"
                   {:type :tool.vm/invalid-request
                    :operation :transform
                    :field :resource
                    :from from
                    :to to})
             nil)
         prepared (transform-input from input)
         provider
         (resolve-provider
          {:operation :transform
           :from from
           :to to
           :options (transform-routing-options options)})
         result
         (try
           (artifact
            (runtime-provider/transform
             from
             to
             (:input prepared)
             (transform-provider-options options)))
           (catch Throwable error
             (transform-failed from to provider error)))]
     (if (not (= to (:artifact/format result)))
       (fail "tool.vm provider returned the wrong target artifact"
             {:type :tool.vm/invalid-provider-result
              :operation :transform
              :from from
              :to to
              :provider (:provider/id provider)
              :actual (:artifact/format result)})
       nil)
     (assoc
      result
      :artifact/provenance
      (merge
       {:provider/id (:provider/id provider)
        :transform/from from
        :transform/to to
        :transform/options (transform-provider-options options)}
       (if (nil? (:source-provenance prepared))
         {}
         {:transform/source-provenance (:source-provenance prepared)}))))))
'''
vm.write_text(text)

halc = ROOT / "core/lib/src/tool/vm/halc.hal"
text = halc.read_text()
if "(defn compile-source" in text:
    raise SystemExit("tool.vm.halc/compile-source already exists")
text += r'''

(defn compile-source
  "Compiles one namespace-declaring HAL source string into canonical HALC."
  ([source]
   (compile-source source {}))
  ([source options]
   (vm/transform :hal :halc source options)))
'''
halc.write_text(text)

hbc = ROOT / "core/lib/src/tool/vm/hbc.hal"
replace_once(
    hbc,
    '(ns tool.vm.hbc\n  (:require [tool.vm :as vm]))\n',
    '(ns tool.vm.hbc\n  (:require [tool.vm :as vm]\n            [tool.vm.halc :as halc]))\n',
)
text = hbc.read_text()
if "(defn compile-source" in text:
    raise SystemExit("tool.vm.hbc compiler helpers already exist")
text += r'''

(defn compile-source
  "Compiles HAL source into canonical HBC0 through a provider-declared edge."
  ([source]
   (compile-source source {}))
  ([source options]
   (vm/transform :hal :hbc source options)))

(defn compile-halc
  "Lowers canonical HALC Bytes into canonical HBC0."
  ([bytes]
   (compile-halc bytes {}))
  ([bytes options]
   (vm/transform :halc :hbc (halc/artifact bytes) options)))
'''
hbc.write_text(text)

(ROOT / "core/lib/test/tool/vm_provider_test.hal").write_text(r'''(ns tool.vm-provider-test
  (:require [tool.vm :as vm]
            [tool.vm.halc :as halc]
            [tool.vm.hbc :as hbc]))

(defn- failure-data
  [function]
  (try
    (do
      (function)
      nil)
    (catch Throwable error
      (ex-data error))))

(def provider
  (vm/current-provider))

(def provider-id
  (:provider/id provider))

(def expected-transforms
  (if (= provider-id :rust)
    [[:hal :halc] [:hal :hbc] [:halc :hbc]]
    [[:hal :halc]]))

(def sample-source
  "(ns sample.tool.vm) (def answer (+ 19 23)) answer")

(def results
  [(test-check "current provider reports exact transformation capabilities"
               [(has? #{:rust :truffle} provider-id)
                (:provider/operations provider)
                (:provider/transforms provider)
                (:provider/engines provider)
                (get (:provider/formats provider) :hal)
                (get (:provider/formats provider) :halc)
                (get (:provider/formats provider) :hbc)]
               [true
                [:validate :inspect :transform :disassemble]
                expected-transforms
                {}
                []
                [:validate :inspect]
                [:validate :inspect :disassemble]])

   (test-check "convenience namespaces construct exact portable artifacts"
               [(select-keys
                 (halc/artifact (str/encode-utf8 "HALC"))
                 [:artifact/format :artifact/version])
                (select-keys
                 (hbc/artifact (str/encode-utf8 "HBC0"))
                 [:artifact/format :artifact/version])]
               [{:artifact/format :halc :artifact/version 1}
                {:artifact/format :hbc :artifact/version 0}])

   (test-check "HAL to HALC is deterministic and records exact provenance"
               (let [first
                     (halc/compile-source
                      sample-source
                      {:resource "sample/tool/vm.hal"})
                     second
                     (halc/compile-source
                      sample-source
                      {:resource "sample/tool/vm.hal"})]
                 [(halc/validate (:artifact/bytes first))
                  (= (:artifact/bytes first)
                     (:artifact/bytes second))
                  (select-keys
                   (:artifact/provenance first)
                   [:provider/id
                    :transform/from
                    :transform/to
                    :transform/options])])
               [true
                true
                {:provider/id provider-id
                 :transform/from :hal
                 :transform/to :halc
                 :transform/options
                 {:resource "sample/tool/vm.hal"}}])

   (test-check "Rust exposes both HBC compiler edges while Truffle rejects them explicitly"
               (if (= provider-id :rust)
                 (let [halc-artifact
                       (halc/compile-source
                        sample-source
                        {:resource "sample/tool/vm.hal"})
                       direct
                       (hbc/compile-source
                        sample-source
                        {:resource "sample/tool/vm.hal"})
                       lowered
                       (hbc/compile-halc
                        (:artifact/bytes halc-artifact))]
                   [(hbc/validate (:artifact/bytes direct))
                    (hbc/validate (:artifact/bytes lowered))
                    (= (:artifact/bytes direct)
                       (:artifact/bytes lowered))
                    (:transform/source-provenance
                     (:artifact/provenance lowered))])
                 (select-keys
                  (failure-data
                   (fn []
                     (hbc/compile-source sample-source)))
                  [:type :operation :from :to :provider :available]))
               (if (= provider-id :rust)
                 [true
                  true
                  true
                  {:provider/id :rust
                   :transform/from :hal
                   :transform/to :halc
                   :transform/options
                   {:resource "sample/tool/vm.hal"}}]
                 {:type :tool.vm/capability-unavailable
                  :operation :transform
                  :from :hal
                  :to :hbc
                  :provider :truffle
                  :available []}))

   (test-check "requesting another provider never falls back to the current runtime"
               (select-keys
                (failure-data
                 (fn []
                   (halc/compile-source
                    sample-source
                    {:provider :not-current})))
                [:type :operation :from :to :provider :available])
               {:type :tool.vm/capability-unavailable
                :operation :transform
                :from :hal
                :to :halc
                :provider :not-current
                :available [provider-id]})

   (test-check "reverse and identity edges remain unsupported transformations"
               [(select-keys
                 (failure-data
                  (fn []
                    (vm/transform
                     :hbc
                     :halc
                     (hbc/artifact (str/encode-utf8 "HBC0")))))
                 [:type :from :to])
                (select-keys
                 (failure-data
                  (fn []
                    (vm/transform
                     :halc
                     :halc
                     (halc/artifact (str/encode-utf8 "HALC")))))
                 [:type :from :to])]
               [{:type :tool.vm/unsupported-transformation
                 :from :hbc
                 :to :halc}
                {:type :tool.vm/unsupported-transformation
                 :from :halc
                 :to :halc}])

   (test-check "HALC does not acquire HBC-only disassembly by provider-wide inference"
               (select-keys
                (failure-data
                 (fn []
                   (vm/disassemble
                    (halc/artifact (str/encode-utf8 "HALC")))))
                [:type :operation :format :provider :available])
               {:type :tool.vm/capability-unavailable
                :operation :disassemble
                :format :halc
                :provider provider-id
                :available []})

   (test-check "transform options reject unrelated routing and target-only data"
               [(select-keys
                 (failure-data
                  (fn []
                    (halc/compile-source
                     sample-source
                     {:fallback :rust})))
                 [:type :field :unknown])
                (select-keys
                 (failure-data
                  (fn []
                    (vm/transform
                     :halc
                     :hbc
                     (halc/artifact (str/encode-utf8 "HALC"))
                     {:resource "not-valid.hal"})))
                 [:type :operation :field :from :to])]
               [{:type :tool.vm/invalid-request
                 :field "tool.vm transform options"
                 :unknown [:fallback]}
                {:type :tool.vm/invalid-request
                 :operation :transform
                 :field :resource
                 :from :halc
                 :to :hbc}])])

results
''')

print("Applied #752 transformation implementation")
