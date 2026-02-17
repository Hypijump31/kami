//! Sandbox-specific error types.

use kami_types::{ErrorKind, KamiError};
use thiserror::Error;

/// Errors from the sandbox layer.
#[derive(Debug, Error)]
pub enum SandboxError {
    /// A required capability was not granted.
    #[error("capability denied: {capability}")]
    CapabilityDenied { capability: String },
    /// Network access denied.
    #[error("network access denied for host: {host}")]
    NetworkDenied { host: String },
    /// Filesystem access denied.
    #[error("filesystem access denied: {path}")]
    FsDenied { path: String },
    /// WASI context build failure.
    #[error("failed to build WASI context: {reason}")]
    WasiBuild { reason: String },
}

impl From<SandboxError> for KamiError {
    fn from(e: SandboxError) -> Self {
        let kind = match &e {
            SandboxError::CapabilityDenied { .. }
            | SandboxError::NetworkDenied { .. }
            | SandboxError::FsDenied { .. } => ErrorKind::PermissionDenied,
            SandboxError::WasiBuild { .. } => ErrorKind::Internal,
        };
        KamiError::new(kind, e.to_string())
    }
}
