//! # kami-runtime
//!
//! Runtime orchestrator for KAMI.
//! Manages tool execution, scheduling, and instance pooling.
//!
//! The `WasmToolExecutor` is the main entry point: it takes a compiled
//! component, sandboxes it, and executes it asynchronously.

pub mod context;
pub mod error;
pub mod executor;
pub mod pool;
pub mod scheduler;

pub use context::ExecutionContext;
pub use error::RuntimeError;
pub use executor::{ExecutionResult, ToolExecutor, WasmToolExecutor};
pub use pool::PoolConfig;
pub use scheduler::Priority;
