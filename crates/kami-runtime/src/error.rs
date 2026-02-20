//! Runtime-specific error types.

use kami_types::{DiagnosticError, ErrorKind, KamiError};
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
    /// WASM file hash does not match the stored SHA-256 digest.
    #[error("integrity violation for tool '{tool_id}': {detail}")]
    IntegrityViolation { tool_id: String, detail: String },
    /// Request rejected by rate limiter.
    #[error("rate limited: tool '{tool_id}' exceeded {limit} req/{window_secs}s")]
    RateLimited {
        tool_id: String,
        limit: u64,
        window_secs: u64,
    },
}

impl From<RuntimeError> for KamiError {
    fn from(e: RuntimeError) -> Self {
        let kind = match &e {
            RuntimeError::ToolNotFound { .. } => ErrorKind::NotFound,
            RuntimeError::Sandbox(kami_sandbox::SandboxError::InvalidConfig { .. }) => {
                ErrorKind::InvalidInput
            }
            RuntimeError::Sandbox(_) => ErrorKind::PermissionDenied,
            RuntimeError::Timeout { .. } => ErrorKind::Timeout,
            RuntimeError::PoolExhausted => ErrorKind::ResourceExhausted,
            RuntimeError::IntegrityViolation { .. } => ErrorKind::PermissionDenied,
            RuntimeError::RateLimited { .. } => ErrorKind::ResourceExhausted,
            RuntimeError::Engine(_) => ErrorKind::Internal,
        };
        KamiError::new(kind, e.to_string())
    }
}

impl DiagnosticError for RuntimeError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::ToolNotFound { name } => Some(format!(
                "No tool with id '{name}' is registered in the local database."
            )),
            Self::Timeout { timeout_ms } => {
                Some(format!("Execution exceeded the {timeout_ms}ms time limit."))
            }
            Self::PoolExhausted => {
                Some("All execution slots are busy. The system is under heavy load.".into())
            }
            Self::IntegrityViolation { .. } => Some(
                "The WASM file on disk does not match the SHA-256 hash recorded at install time."
                    .into(),
            ),
            Self::RateLimited { .. } => {
                Some("Too many requests for this tool in the current time window.".into())
            }
            Self::Engine(e) => e.hint(),
            Self::Sandbox(e) => e.hint(),
        }
    }

    fn fix(&self) -> Option<String> {
        match self {
            Self::ToolNotFound { .. } => {
                Some("Install the tool first: kami install <path-to-tool>".into())
            }
            Self::Timeout { .. } => Some(
                "Increase the timeout in tool.toml:\n  [security]\n  max_execution_ms = 10000"
                    .into(),
            ),
            Self::PoolExhausted => {
                Some("Increase runtime concurrency: kami serve --concurrency 16".into())
            }
            Self::IntegrityViolation { .. } => {
                Some("Re-install the tool: kami uninstall <id> && kami install <path>".into())
            }
            Self::RateLimited { .. } => {
                Some("Wait before retrying, or increase rate_limit_per_tool in config.".into())
            }
            Self::Engine(e) => e.fix(),
            Self::Sandbox(e) => e.fix(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_not_found_maps_to_not_found() {
        let err: KamiError = RuntimeError::ToolNotFound { name: "x".into() }.into();
        assert_eq!(err.kind, ErrorKind::NotFound);
    }

    #[test]
    fn timeout_maps_to_timeout_kind() {
        let err: KamiError = RuntimeError::Timeout { timeout_ms: 5000 }.into();
        assert_eq!(err.kind, ErrorKind::Timeout);
    }

    #[test]
    fn rate_limited_has_fix_suggestion() {
        let e = RuntimeError::RateLimited {
            tool_id: "t".into(),
            limit: 10,
            window_secs: 60,
        };
        assert!(e.fix().expect("has fix").contains("rate_limit"));
    }

    #[test]
    fn pool_exhausted_maps_to_resource_exhausted() {
        let err: KamiError = RuntimeError::PoolExhausted.into();
        assert_eq!(err.kind, ErrorKind::ResourceExhausted);
    }

    #[test]
    fn integrity_violation_maps_to_permission_denied() {
        let err: KamiError = RuntimeError::IntegrityViolation {
            tool_id: "t".into(),
            detail: "mismatch".into(),
        }
        .into();
        assert_eq!(err.kind, ErrorKind::PermissionDenied);
    }
}
