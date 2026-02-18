//! # kami-sandbox
//!
//! Isolation and security enforcement for WASM tool execution.
//! Implements capability-based security with deny-all defaults.
//!
//! Produces a `WasiCtx` that enforces network allow-lists,
//! filesystem jailing, and resource limits.

pub mod capability;
pub mod error;
pub mod filesystem;
pub mod network;
pub mod wasi;

pub use capability::{
    validate_security_config, CapabilityChecker, DefaultCapabilityChecker,
};
pub use error::SandboxError;
pub use filesystem::FsJail;
pub use wasi::{build_wasi_ctx, WasiConfig};
