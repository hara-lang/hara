//! Import-free bytecode VM facade for the standalone observation payload.
//!
//! This compiles the shared VM source directly, matching `hara-wasm-raw`, so
//! the plain-C observation ABI does not inherit the browser wasm-bindgen API.

#[path = "../../src/vm/artifact.rs"]
pub mod artifact;
#[path = "../../src/vm/compiler.rs"]
pub mod compiler;
#[path = "../../src/vm/error.rs"]
pub mod error;
#[path = "../../src/vm/fiber.rs"]
pub mod fiber;
#[path = "../../src/vm/frame.rs"]
pub mod frame;
#[path = "../../src/vm/machine.rs"]
pub mod machine;
#[path = "../../src/vm/opcode.rs"]
pub mod opcode;
#[path = "../../src/vm/program.rs"]
pub mod program;
#[path = "../../src/vm/session.rs"]
pub mod session;
#[path = "../../src/vm/slot.rs"]
mod slot;
#[path = "../../src/vm/source_map.rs"]
pub mod source_map;
#[path = "../../src/vm/validate.rs"]
pub mod validate;

pub use artifact::decode_program;
pub use compiler::compile_source_with;
pub use error::VmError;
pub use machine::Machine;
pub use program::Program;
pub use validate::validate;
