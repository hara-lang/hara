#![allow(clippy::too_many_lines)] // Temporary compatibility facade during Java-port split.
#[cfg(not(target_arch = "wasm32"))]
pub mod asset;
#[cfg(not(target_arch = "wasm32"))]
pub mod cli_app;
// Public embedding surface used by native hosts such as Hoplite. The module's
// value, protocol, promise, and host-call types form the runtime integration ABI.
pub mod core;
mod clock;
pub mod extension;
pub mod file;
#[cfg(not(target_arch = "wasm32"))]
pub mod hta;
#[cfg(not(target_arch = "wasm32"))]
pub mod invoke_hta;
#[cfg(not(target_arch = "wasm32"))]
pub use invoke_hta::{InvokeHtaError, MAX_INVOKE_HTA_RESULT_BYTES};
#[cfg(not(target_arch = "wasm32"))]
pub mod identity_tool;
#[cfg(feature = "evaluation-journal")]
pub mod journal;
pub mod interpreter_observation;
mod json;
pub mod kernel;
pub mod lang;
pub mod live_session;
#[cfg(not(target_arch = "wasm32"))]
pub mod native_cli;
#[cfg(not(target_arch = "wasm32"))]
mod native_extension;
#[cfg(not(target_arch = "wasm32"))]
pub mod native_link;
#[cfg(not(target_arch = "wasm32"))]
pub mod native_module;
#[cfg(not(target_arch = "wasm32"))]
mod native_process;
mod numeric;
pub mod package_catalog;
#[cfg(not(target_arch = "wasm32"))]
pub mod package;
#[cfg(not(target_arch = "wasm32"))]
mod process_extension;
#[cfg(not(target_arch = "wasm32"))]
pub mod project;
#[cfg(not(target_arch = "wasm32"))]
pub mod resp;
pub mod snapshot;
#[cfg(not(target_arch = "wasm32"))]
pub mod snapshot_tool;
#[cfg(not(target_arch = "wasm32"))]
pub mod tap;
pub mod task;
pub mod work;
#[path = "work/session.rs"]
mod work_session;
// Experimental staged bytecode VM (issue #195). Non-default feature; the
// default evaluator is untouched.
#[cfg(feature = "tracing-jit")]
pub mod jit;
#[cfg(feature = "bytecode-vm")]
pub mod vm;
#[cfg(not(target_arch = "wasm32"))]
pub mod wasmtime_provider;
#[cfg(feature = "whole-wasm")]
pub mod whole_wasm;
use crate::kernel::Form;
use crate::lang::data::{OrderedMap as POrderedMap, Vector as PVector};
use crate::lang::protocol::INamespaced;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

include!("runtime/evaluator.rs");
include!("runtime/model.rs");
include!("runtime/session_model.rs");
include!("runtime/sandbox_model.rs");
include!("runtime/session.rs");
include!("runtime/sandbox.rs");
include!("runtime/runtime.rs");
include!("runtime/bytecode.rs");
include!("runtime/evaluation.rs");
include!("runtime/wasm.rs");

/// Constructs the zero-authority Runtime profile used by an external secure
/// [`SandboxProvider`] evaluator process.
///
/// This is a Rust embedding seam only. It does not add a native Hara Runtime,
/// Evaluator, Kernel, or Sandbox operation and does not register the returned
/// Runtime with a parent [`SessionKernel`].
#[cfg(not(target_arch = "wasm32"))]
pub fn restricted_sandbox_runtime() -> Runtime {
    Runtime::sandbox()
}

include!("runtime/tests.rs");
