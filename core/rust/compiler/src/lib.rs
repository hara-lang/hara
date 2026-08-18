//! Full Hara compilation surface, kept outside VM-only deployments.

pub use hara_vm::{Program, Value};
pub use hara_wasm::vm::{compile_halc_module, compile_source, compile_source_with, CompileError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompileTarget {
    HbcModule,
    WholeWasm,
}

impl CompileTarget {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HbcModule => "hbc-module",
            Self::WholeWasm => "whole-wasm",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledArtifact {
    target: CompileTarget,
    bytes: Vec<u8>,
}

impl CompiledArtifact {
    pub fn target(&self) -> CompileTarget {
        self.target
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

pub fn compile(source: &str, target: CompileTarget) -> Result<CompiledArtifact, String> {
    let program = compile_source(source).map_err(|error| error.to_string())?;
    let bytes = match target {
        CompileTarget::HbcModule => hara_wasm::vm::encode_program(&program)?,
        CompileTarget::WholeWasm => compile_whole_wasm(&program)?,
    };
    Ok(CompiledArtifact { target, bytes })
}

pub fn compile_bytecode(source: &str) -> Result<Vec<u8>, String> {
    compile(source, CompileTarget::HbcModule).map(CompiledArtifact::into_bytes)
}

#[cfg(feature = "full-wasm")]
pub fn compile_wasm(source: &str) -> Result<Vec<u8>, String> {
    compile(source, CompileTarget::WholeWasm).map(CompiledArtifact::into_bytes)
}

#[cfg(feature = "full-wasm")]
fn compile_whole_wasm(program: &Program) -> Result<Vec<u8>, String> {
    hara_wasm::whole_wasm::compile_artifact(program)
}

#[cfg(not(feature = "full-wasm"))]
fn compile_whole_wasm(_program: &Program) -> Result<Vec<u8>, String> {
    Err("whole-wasm compilation requires the hara-compiler/full-wasm feature".into())
}

#[cfg(test)]
mod tests {
    use super::{compile, compile_bytecode, CompileTarget};

    #[test]
    fn compiler_output_executes_in_vm_only_crate() {
        let artifact = compile_bytecode("(+ 19 23)").unwrap();
        assert_eq!(hara_vm::execute(&artifact).unwrap().display(), "42");
    }

    #[test]
    fn explicit_hbc_target_preserves_the_legacy_bytecode_contract() {
        let artifact = compile("(+ 19 23)", CompileTarget::HbcModule).unwrap();
        assert_eq!(artifact.target(), CompileTarget::HbcModule);
        assert_eq!(hara_vm::execute(artifact.bytes()).unwrap().display(), "42");
    }

    #[cfg(not(feature = "full-wasm"))]
    #[test]
    fn whole_wasm_target_requires_its_explicit_feature() {
        let error = compile("(+ 19 23)", CompileTarget::WholeWasm).unwrap_err();
        assert!(error.contains("full-wasm"));
    }

    #[cfg(feature = "full-wasm")]
    #[test]
    fn whole_wasm_target_reports_its_product_identity() {
        let artifact = compile("(+ 19 23)", CompileTarget::WholeWasm).unwrap();
        assert_eq!(artifact.target(), CompileTarget::WholeWasm);
        assert!(!artifact.bytes().is_empty());
    }
}
