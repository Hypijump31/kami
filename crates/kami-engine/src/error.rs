//! Engine-specific error types.

use kami_types::{ErrorKind, KamiError};
use thiserror::Error;

/// Errors from the WASM engine.
#[derive(Debug, Error)]
pub enum EngineError {
    /// Failed to compile a WASM component.
    #[error("failed to compile WASM component: {reason}")]
    Compilation {
        reason: String,
        #[source]
        source: wasmtime::Error,
    },
    /// Failed to instantiate a component.
    #[error("failed to instantiate component: {reason}")]
    Instantiation {
        reason: String,
        #[source]
        source: wasmtime::Error,
    },
    /// WASM instance trapped during execution.
    #[error("instance trapped: {message}")]
    Trap { message: String },
    /// Exported function not found.
    #[error("export not found: {name}")]
    ExportNotFound { name: String },
    /// Resource limit exceeded.
    #[error("resource limit exceeded: {limit}")]
    ResourceExceeded { limit: String },
    /// Configuration error.
    #[error("engine configuration error: {0}")]
    Config(String),
}

impl From<EngineError> for KamiError {
    fn from(e: EngineError) -> Self {
        let kind = match &e {
            EngineError::ResourceExceeded { .. } => ErrorKind::ResourceExhausted,
            EngineError::ExportNotFound { .. } => ErrorKind::NotFound,
            _ => ErrorKind::Internal,
        };
        KamiError::new(kind, e.to_string())
    }
}
