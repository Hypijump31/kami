//! Engine-specific error types.

use kami_types::{DiagnosticError, ErrorKind, KamiError};
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

impl DiagnosticError for EngineError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::Compilation { reason, .. } => {
                if reason.contains("unknown import") {
                    Some(
                        "The WASM component imports an interface that KAMI does not provide."
                            .into(),
                    )
                } else {
                    Some("The WASM binary could not be compiled by the engine.".into())
                }
            }
            Self::ExportNotFound { name } => Some(format!(
                "The component does not export '{name}'. It must export 'run' and 'describe'."
            )),
            Self::ResourceExceeded { .. } => {
                Some("The component exceeded its allocated resource budget.".into())
            }
            Self::Trap { .. } => Some("The WASM instance trapped during execution.".into()),
            _ => None,
        }
    }

    fn fix(&self) -> Option<String> {
        match self {
            Self::Compilation { reason, .. } if reason.contains("unknown import") => Some(
                "Check that your tool.toml [security] section grants the required capabilities.\n\
                 Example: net_allow_list = [\"api.example.com\"]"
                    .into(),
            ),
            Self::ExportNotFound { .. } => Some(
                "Use the kami_tool! macro from kami-guest to generate the required exports.".into(),
            ),
            Self::ResourceExceeded { limit } => Some(format!(
                "Increase the limit in tool.toml:\n  [security]\n  {limit}"
            )),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_exceeded_maps_to_exhausted() {
        let err: KamiError = EngineError::ResourceExceeded {
            limit: "memory".into(),
        }
        .into();
        assert_eq!(err.kind, ErrorKind::ResourceExhausted);
    }

    #[test]
    fn export_not_found_maps_to_not_found() {
        let err: KamiError = EngineError::ExportNotFound { name: "run".into() }.into();
        assert_eq!(err.kind, ErrorKind::NotFound);
    }

    #[test]
    fn compilation_hint_mentions_unknown_import() {
        let e = EngineError::Compilation {
            reason: "unknown import: wasi:http".into(),
            source: wasmtime::Error::msg("test"),
        };
        let hint = e.hint().expect("should have hint");
        assert!(hint.contains("does not provide"));
    }
}
