//! # kami-runtime
//!
//! Runtime orchestrator for KAMI.
//! Manages tool execution, scheduling, component caching, and
//! tool resolution from the registry.
//!
//! Use `KamiRuntime` for high-level tool execution by ID, or
//! `WasmToolExecutor` for direct component execution.

pub mod cache;
pub mod context;
pub mod error;
pub mod executor;
pub mod orchestrator;
pub mod pool;
pub mod resolver;
pub mod scheduler;

pub use cache::{CachedComponent, ComponentCache};
pub use context::ExecutionContext;
pub use error::RuntimeError;
pub use executor::{ExecutionResult, ToolExecutor, WasmToolExecutor};
pub use orchestrator::{KamiRuntime, RuntimeConfig};
pub use pool::PoolConfig;
pub use resolver::ToolResolver;
pub use scheduler::{Priority, Scheduler, SchedulerConfig};
