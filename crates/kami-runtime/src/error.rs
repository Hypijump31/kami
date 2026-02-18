//! Runtime-specific error types.

use kami_types::{ErrorKind, KamiError};
use thiserror::Error;

/// Errors from the runtime orchestrator.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Tool not found in registry.
    #[error("tool not found: {name}")]
    ToolNotFound { name: String },
    /// Engine error during execution.
    #[error("engine error: {0}")]
    Engine(#[from] kami_engine::EngineError),
    /// Sandbox policy violation.
    #[error("sandbox error: {0}")]
    Sandbox(#[from] kami_sandbox::SandboxError),
    /// Execution timed out.
    #[error("execution timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    /// Pool exhausted (no available instances).
    #[error("instance pool exhausted")]
    PoolExhausted,
}

impl From<RuntimeError> for KamiError {
    fn from(e: RuntimeError) -> Self {
        let kind = match &e {
            RuntimeError::ToolNotFound { .. } => ErrorKind::NotFound,
            RuntimeError::Sandbox(
                kami_sandbox::SandboxError::InvalidConfig { .. },
            ) => ErrorKind::InvalidInput,
            RuntimeError::Sandbox(_) => ErrorKind::PermissionDenied,
            RuntimeError::Timeout { .. } => ErrorKind::Timeout,
            RuntimeError::PoolExhausted => ErrorKind::ResourceExhausted,
            RuntimeError::Engine(_) => ErrorKind::Internal,
        };
        KamiError::new(kind, e.to_string())
    }
}
