//! Deterministic indexed container for the embedded standard library.

use sha2::{Digest, Sha256};

use crate::{core, kernel, Runtime, EAGER_HAL_RESOURCES, EMBEDDED_HAL_RESOURCES};

const MAGIC: &[u8; 4] = b"HBB2";

pub struct ModuleSource<'a> {
    pub resource: &'a str,
    pub source: &'a str,
}

struct Module<'a> {
    resource: &'a str,
    namespace_form: &'a str,
    source_digest: &'a [u8; 32],
    dependencies: &'a [String],
    eager: bool,
    artifact: &'a [u8],
}

pub fn embedded_standard_library_sources() -> Vec<ModuleSource<'static>> {
    let ordered = std::iter::once("std.foundation")
        .chain(EAGER_HAL_RESOURCES.iter().copied())
        .chain(
            EMBEDDED_HAL_RESOURCES
                .iter()
                .map(|(namespace, _, _)| *namespace)
                .filter(|namespace| {
                    standard_library_namespace(namespace)
                        && *namespace != "std.foundation"
                        && !EAGER_HAL_RESOURCES.contains(namespace)
                }),
        );
    ordered
        .map(|resource| {
            let source = EMBEDDED_HAL_RESOURCES
                .iter()
                .find_map(|(name, _, source)| (*name == resource).then_some(*source))
                .unwrap_or_else(|| panic!("missing embedded HAL resource: {resource}"));
            ModuleSource { resource, source }
        })
        .collect()
}

pub fn compile_bytecode_bundle(sources: &[ModuleSource<'_>]) -> Result<Vec<u8>, String> {
    let mut runtime = Runtime::core();
    for &(name, _, source) in EMBEDDED_HAL_RESOURCES {
        runtime.register_resource(name, source);
    }
    for source in sources {
        runtime.register_resource(source.resource, source.source);
    }
    let mut encoded = Vec::new();
    for source in sources {
        let (namespace_form, body) = split_namespace_form(source.source)?;
        runtime
            .eval_text(namespace_form)
            .map_err(|error| format!("{}: namespace declaration: {error}", source.resource))?;
        if source.resource == "std.foundation" {
            runtime.prepare_foundation_bytecode();
        }
        let artifact = runtime
            .compile_bytecode_artifact(body)
            .map_err(|error| format!("{}: bytecode compilation: {error}", source.resource))?;
        core::with_definition_origin(kernel::VarOrigin::HalFallback, || {
            runtime.eval_bytecode_artifact(&artifact)
        })
        .map_err(|error| format!("{}: bytecode execution: {error}", source.resource))?;
        let source_digest: [u8; 32] = Sha256::digest(source.source.as_bytes()).into();
        let dependencies = namespace_dependencies(namespace_form)?;
        let eager =
            source.resource == "std.foundation" || EAGER_HAL_RESOURCES.contains(&source.resource);
        encoded.push((
            source.resource,
            namespace_form,
            source_digest,
            dependencies,
            eager,
            artifact,
        ));
    }

    let modules = encoded
        .iter()
        .map(
            |(resource, namespace_form, source_digest, dependencies, eager, artifact)| Module {
                resource,
                namespace_form,
                source_digest,
                dependencies,
                eager: *eager,
                artifact,
            },
        )
        .collect::<Vec<_>>();
    encode(&modules)
}

pub fn compile_embedded_standard_library_bundle() -> Result<Vec<u8>, String> {
    compile_bytecode_bundle(&embedded_standard_library_sources())
}

pub fn eval_bytecode_bundle(runtime: &mut Runtime, bytes: &[u8]) -> Result<(), String> {
    let modules = decode(bytes)?;
    let mut names = std::collections::HashSet::with_capacity(modules.len());
    for module in &modules {
        if !names.insert(module.resource.clone()) {
            return Err(format!(
                "duplicate bytecode bundle module: {}",
                module.resource
            ));
        }
        crate::vm::decode_program(&module.artifact)
            .map_err(|error| format!("{}: invalid bytecode artifact: {error}", module.resource))?;
    }
    let namespaces_before = runtime.namespace_registry.snapshot();
    let environment_before = runtime.env.clone();
    let macros_before = runtime.macros.borrow().clone();
    let protocols_before = runtime.protocols.snapshot();
    let multimethods_before = core::snapshot_multimethods();
    let resources_before = runtime.bytecode_resources.clone();
    let loaded_before = runtime.loaded_resources.clone();
    let loaded = (|| {
        for module in &modules {
            runtime.register_bytecode_resource(
                module.resource.clone(),
                module.namespace_form.clone(),
                module.artifact.clone(),
            );
        }
        for module in modules.iter().filter(|module| module.eager) {
            if module.resource == "std.foundation" {
                runtime.prepare_foundation_bytecode();
            }
            core::with_definition_origin(kernel::VarOrigin::HalFallback, || {
                runtime.load_bytecode_resource(&module.resource).map(|_| ())
            })
            .map_err(|error| format!("{}: {error}", module.resource))?;
            runtime.loaded_resources.insert(module.resource.clone());
        }
        runtime.use_namespace("user");
        Ok(())
    })();
    if let Err(error) = loaded {
        runtime.namespace_registry.restore(namespaces_before);
        runtime.env = environment_before;
        *runtime.macros.borrow_mut() = macros_before;
        runtime.protocols.restore(protocols_before);
        core::restore_multimethods(multimethods_before);
        runtime.bytecode_resources = resources_before;
        runtime.loaded_resources = loaded_before;
        return Err(error);
    }
    Ok(())
}

#[derive(Clone)]
pub(crate) struct OwnedModule {
    pub resource: String,
    pub namespace_form: String,
    pub source_digest: [u8; 32],
    pub dependencies: Vec<String>,
    pub eager: bool,
    pub artifact: Vec<u8>,
}

fn encode(modules: &[Module<'_>]) -> Result<Vec<u8>, String> {
    let mut payload = Vec::new();
    put_u32(&mut payload, modules.len())?;
    for module in modules {
        put_bytes(&mut payload, module.resource.as_bytes())?;
        put_bytes(&mut payload, module.namespace_form.as_bytes())?;
        payload.extend_from_slice(module.source_digest);
        put_u32(&mut payload, module.dependencies.len())?;
        for dependency in module.dependencies {
            put_bytes(&mut payload, dependency.as_bytes())?;
        }
        payload.push(u8::from(module.eager));
        put_bytes(&mut payload, module.artifact)?;
    }
    let checksum = Sha256::digest(&payload);
    let mut output = Vec::with_capacity(4 + checksum.len() + payload.len());
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&checksum);
    output.extend_from_slice(&payload);
    Ok(output)
}

pub(crate) fn decode(bytes: &[u8]) -> Result<Vec<OwnedModule>, String> {
    if bytes.len() < 36 || &bytes[..4] != MAGIC {
        return Err("invalid foundation bytecode bundle header".into());
    }
    let payload = &bytes[36..];
    if Sha256::digest(payload).as_slice() != &bytes[4..36] {
        return Err("foundation bytecode bundle checksum mismatch".into());
    }
    let mut input = payload;
    let count = take_u32(&mut input)? as usize;
    let mut modules = Vec::with_capacity(count);
    for _ in 0..count {
        let resource = take_string(&mut input)?;
        let namespace_form = take_string(&mut input)?;
        let source_digest = take(&mut input, 32)?.try_into().unwrap();
        let dependency_count = take_u32(&mut input)? as usize;
        let dependencies = (0..dependency_count)
            .map(|_| take_string(&mut input))
            .collect::<Result<Vec<_>, _>>()?;
        let eager = match take(&mut input, 1)?[0] {
            0 => false,
            1 => true,
            _ => return Err("standard-library bundle contains invalid eager flag".into()),
        };
        let artifact = take_bytes(&mut input)?.to_vec();
        modules.push(OwnedModule {
            resource,
            namespace_form,
            source_digest,
            dependencies,
            eager,
            artifact,
        });
    }
    if !input.is_empty() {
        return Err("trailing bytes in foundation bytecode bundle".into());
    }
    Ok(modules)
}

fn standard_library_namespace(namespace: &str) -> bool {
    ["std.", "code.", "lang."]
        .iter()
        .any(|prefix| namespace.starts_with(prefix))
}

fn namespace_dependencies(namespace_form: &str) -> Result<Vec<String>, String> {
    let forms = kernel::parse_forms(namespace_form)?;
    let Some(kernel::Form::List(items)) = forms.first() else {
        return Err("standard-library module has invalid ns form".into());
    };
    let config = kernel::GeneratedNamespaceConfig::configure_with(&items[2..], |_| true)?;
    let mut dependencies = config.required_namespaces().to_vec();
    dependencies.extend(config.used_namespaces().iter().cloned());
    dependencies.sort();
    dependencies.dedup();
    Ok(dependencies)
}

fn split_namespace_form(source: &str) -> Result<(&str, &str), String> {
    let start = source.find("(ns ").ok_or("HAL module is missing ns form")?;
    let mut depth = 0usize;
    let mut string = false;
    let mut escape = false;
    for (offset, ch) in source[start..].char_indices() {
        if string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                string = false;
            }
            continue;
        }
        match ch {
            '"' => string = true,
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1).ok_or("invalid ns form")?;
                if depth == 0 {
                    let end = start + offset + ch.len_utf8();
                    return Ok((&source[start..end], &source[end..]));
                }
            }
            _ => {}
        }
    }
    Err("unterminated ns form".into())
}

fn put_u32(output: &mut Vec<u8>, value: usize) -> Result<(), String> {
    let value = u32::try_from(value).map_err(|_| "foundation bundle exceeds u32 limits")?;
    output.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

fn put_bytes(output: &mut Vec<u8>, value: &[u8]) -> Result<(), String> {
    put_u32(output, value.len())?;
    output.extend_from_slice(value);
    Ok(())
}

fn take_u32(input: &mut &[u8]) -> Result<u32, String> {
    let bytes = take(input, 4)?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}

fn take_bytes<'a>(input: &mut &'a [u8]) -> Result<&'a [u8], String> {
    let len = take_u32(input)? as usize;
    take(input, len)
}

fn take_string(input: &mut &[u8]) -> Result<String, String> {
    String::from_utf8(take_bytes(input)?.to_vec())
        .map_err(|_| "foundation bundle contains invalid UTF-8".into())
}

fn take<'a>(input: &mut &'a [u8], len: usize) -> Result<&'a [u8], String> {
    if input.len() < len {
        return Err("truncated foundation bytecode bundle".into());
    }
    let (value, rest) = input.split_at(len);
    *input = rest;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_bundle_round_trips_and_bootstraps() {
        let bytes =
            compile_embedded_standard_library_bundle().expect("compile standard library bundle");
        let mut runtime = Runtime::core();
        for &(name, _, source) in EMBEDDED_HAL_RESOURCES {
            runtime.register_resource(name, source);
        }
        eval_bytecode_bundle(&mut runtime, &bytes).expect("load foundation bundle");
        let publics = runtime
            .eval_native("(keys (ns-publics 'std.foundation.string))")
            .expect("inspect string namespace");
        assert!(publics.contains("upper"), "{publics}");
        assert!(runtime.use_namespace("std.foundation.string"));
        assert_eq!(runtime.eval_native("(upper \"hara\")").unwrap(), "\"HARA\"");
        assert!(runtime.use_namespace("std.foundation"));
        assert_eq!(runtime.eval_native("(if-not false 42)").unwrap(), "42");
        assert!(
            runtime.namespace_registry.find("lang.core").is_none(),
            "non-eager namespaces must remain indexed but unloaded"
        );
        runtime
            .load_bytecode_resource("lang.core")
            .expect("load lazy bytecode namespace");
        assert!(runtime.namespace_registry.find("lang.core").is_some());
    }

    #[test]
    fn foundation_module_loads_through_the_bytecode_index() {
        let source = embedded_standard_library_sources()
            .into_iter()
            .find(|source| source.resource == "std.foundation")
            .expect("embedded foundation source");
        let bytes = compile_bytecode_bundle(&[source]).expect("compile foundation module");
        let modules = decode(&bytes).expect("decode foundation bundle");
        let program =
            crate::vm::decode_program(&modules[0].artifact).expect("decode foundation HBC");
        let first_macro = program
            .entry_function()
            .code
            .iter()
            .position(|instruction| matches!(instruction, crate::vm::Instruction::DefMacro { .. }));
        let first_return = program
            .entry_function()
            .code
            .iter()
            .position(|instruction| matches!(instruction, crate::vm::Instruction::Return));
        assert!(
            first_macro.is_some() && first_return.is_some_and(|index| index > first_macro.unwrap()),
            "Foundation artifact must execute macros before return: macro={first_macro:?}, return={first_return:?}"
        );
        let mut runtime = Runtime::core();
        eval_bytecode_bundle(&mut runtime, &bytes).expect("load indexed foundation module");
        assert!(runtime.use_namespace("std.foundation"));
        assert_eq!(
            runtime.eval_native("(vec (repeat 3 :x))").unwrap(),
            "[:x :x :x]"
        );
        assert!(
            runtime
                .macros
                .borrow()
                .contains_key(&("std.foundation".into(), "if-not".into())),
            "indexed Foundation load must register macros: {:?}",
            runtime.macros.borrow().keys().collect::<Vec<_>>()
        );
        assert_eq!(runtime.eval_native("(if-not false 42)").unwrap(), "42");
    }

    #[test]
    fn bundle_encoding_is_deterministic() {
        let sources = [ModuleSource {
            resource: "example.deterministic",
            source: "(ns example.deterministic) (def answer 42)",
        }];
        let first = compile_bytecode_bundle(&sources).expect("first deterministic bundle");
        let second = compile_bytecode_bundle(&sources).expect("second deterministic bundle");
        assert_eq!(first, second);
    }

    #[test]
    fn eager_failure_rolls_back_the_whole_bundle() {
        let mut compiler = Runtime::core();
        compiler.use_namespace("example.good");
        let good_artifact = compiler
            .compile_bytecode_artifact("(def marker 42)")
            .expect("compile successful eager module");
        compiler.use_namespace("example.bad");
        let bad_artifact = compiler
            .compile_bytecode_artifact("(throw \"boom\")")
            .expect("compile failing eager module");
        let good_digest = Sha256::digest(b"good").into();
        let bad_digest = Sha256::digest(b"bad").into();
        let modules = [
            Module {
                resource: "example.good",
                namespace_form: "(ns example.good)",
                source_digest: &good_digest,
                dependencies: &[],
                eager: true,
                artifact: &good_artifact,
            },
            Module {
                resource: "example.bad",
                namespace_form: "(ns example.bad)",
                source_digest: &bad_digest,
                dependencies: &[],
                eager: true,
                artifact: &bad_artifact,
            },
        ];
        let bytes = encode(&modules).expect("encode transactional fixture");
        let mut runtime = Runtime::core();
        let namespaces_before = runtime
            .namespace_registry
            .all()
            .into_iter()
            .map(|namespace| namespace.name().as_str().to_owned())
            .collect::<std::collections::HashSet<_>>();

        let error = eval_bytecode_bundle(&mut runtime, &bytes).unwrap_err();

        assert!(error.contains("example.bad"), "{error}");
        assert!(!runtime.bytecode_resources.contains_key("example.good"));
        assert!(!runtime.bytecode_resources.contains_key("example.bad"));
        assert!(!runtime.loaded_resources.contains("example.good"));
        assert!(!runtime.loaded_resources.contains("example.bad"));
        assert_eq!(
            runtime
                .namespace_registry
                .all()
                .into_iter()
                .map(|namespace| namespace.name().as_str().to_owned())
                .collect::<std::collections::HashSet<_>>(),
            namespaces_before
        );
        assert_eq!(runtime.namespace_registry.current().name().as_str(), "user");
    }

    #[test]
    fn lazy_module_loads_protocol_dependency_before_extend_type() {
        let sources = [
            ModuleSource {
                resource: "example.protocol",
                source: "(ns example.protocol) (defprotocol IEmitter (emit-form [value]))",
            },
            ModuleSource {
                resource: "example.emit",
                source: "(ns example.emit (:require [example.protocol :as compiler])) (defstruct Emitter []) (extend-type Emitter compiler/IEmitter (emit-form [value] value))",
            },
        ];
        let bytes = compile_bytecode_bundle(&sources).expect("compile lazy protocol fixture");
        let mut runtime = Runtime::core();
        eval_bytecode_bundle(&mut runtime, &bytes).expect("index lazy protocol fixture");

        runtime
            .load_bytecode_resource("example.emit")
            .expect("load protocol consumer and dependency");

        assert!(runtime
            .namespace_registry
            .find("example.protocol")
            .is_some());
        assert!(runtime.namespace_registry.find("example.emit").is_some());
    }

    #[test]
    fn eager_modules_load_in_their_own_namespaces() {
        let sources = embedded_standard_library_sources()
            .into_iter()
            .filter(|source| {
                source.resource == "std.foundation"
                    || EAGER_HAL_RESOURCES.contains(&source.resource)
            })
            .collect::<Vec<_>>();
        let bytes = compile_bytecode_bundle(&sources).expect("compile eager modules");
        let mut runtime = Runtime::core();
        eval_bytecode_bundle(&mut runtime, &bytes).expect("load eager modules");
        assert!(runtime.use_namespace("std.foundation.string"));
        assert_eq!(runtime.eval_native("(repeat \"x\" 3)").unwrap(), "\"xxx\"");
    }

    #[test]
    fn embedded_bundle_indexes_every_standard_library_namespace() {
        let sources = embedded_standard_library_sources();
        let expected = sources
            .iter()
            .map(|source| source.resource)
            .collect::<Vec<_>>();
        let bytes = compile_bytecode_bundle(&sources).expect("compile standard library bundle");
        let modules = decode(&bytes).expect("decode standard library bundle");
        let actual = modules
            .iter()
            .map(|module| module.resource.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            actual, expected,
            "bundle inventory must be exact and ordered"
        );
        assert!(
            modules.len() >= 250,
            "standard-library inventory was truncated"
        );
        assert!(modules.iter().any(|module| module.resource == "code.test"));
        assert!(modules.iter().any(|module| module.resource == "lang.core"));
    }
}
