//! Sandbox-specific error types.

use kami_types::{DiagnosticError, ErrorKind, KamiError};
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
    /// Invalid security configuration.
    #[error("invalid security config: {reason}")]
    InvalidConfig { reason: String },
}

impl From<SandboxError> for KamiError {
    fn from(e: SandboxError) -> Self {
        let kind = match &e {
            SandboxError::CapabilityDenied { .. }
            | SandboxError::NetworkDenied { .. }
            | SandboxError::FsDenied { .. } => ErrorKind::PermissionDenied,
            SandboxError::WasiBuild { .. } => ErrorKind::Internal,
            SandboxError::InvalidConfig { .. } => ErrorKind::InvalidInput,
        };
        KamiError::new(kind, e.to_string())
    }
}

impl DiagnosticError for SandboxError {
    fn hint(&self) -> Option<String> {
        match self {
            Self::CapabilityDenied { capability } => Some(format!(
                "The tool requires the '{capability}' capability but it was not granted."
            )),
            Self::NetworkDenied { host } => Some(format!(
                "The tool tried to connect to '{host}' but network access is denied."
            )),
            Self::FsDenied { path } => Some(format!(
                "The tool tried to access '{path}' outside its sandbox."
            )),
            Self::InvalidConfig { .. } => {
                Some("The security configuration in tool.toml has invalid values.".into())
            }
            Self::WasiBuild { .. } => None,
        }
    }

    fn fix(&self) -> Option<String> {
        match self {
            Self::NetworkDenied { host } => Some(format!(
                "Add to tool.toml:\n  [security]\n  net_allow_list = [\"{host}\"]"
            )),
            Self::FsDenied { .. } => Some(
                "Set fs_access in tool.toml:\n  [security]\n  fs_access = \"read-only\"".into(),
            ),
            Self::CapabilityDenied { .. } => {
                Some("Grant the required capability in tool.toml [security] section.".into())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_denied_maps_to_permission_denied() {
        let err: KamiError = SandboxError::CapabilityDenied {
            capability: "net".into(),
        }
        .into();
        assert_eq!(err.kind, ErrorKind::PermissionDenied);
    }

    #[test]
    fn network_denied_has_hint_with_host() {
        let e = SandboxError::NetworkDenied {
            host: "evil.com".into(),
        };
        let hint = e.hint().expect("has hint");
        assert!(hint.contains("evil.com"));
    }

    #[test]
    fn network_denied_has_fix_with_toml() {
        let e = SandboxError::NetworkDenied {
            host: "api.x.com".into(),
        };
        let fix = e.fix().expect("has fix");
        assert!(fix.contains("net_allow_list"));
        assert!(fix.contains("api.x.com"));
    }

    #[test]
    fn fs_denied_maps_to_permission_denied() {
        let err: KamiError = SandboxError::FsDenied {
            path: "/etc".into(),
        }
        .into();
        assert_eq!(err.kind, ErrorKind::PermissionDenied);
    }

    #[test]
    fn invalid_config_maps_to_invalid_input() {
        let err: KamiError = SandboxError::InvalidConfig {
            reason: "bad".into(),
        }
        .into();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
    }

    #[test]
    fn wasi_build_has_no_fix() {
        let e = SandboxError::WasiBuild {
            reason: "oom".into(),
        };
        assert!(e.fix().is_none());
    }
}
