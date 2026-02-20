//! Capability checking trait and types.

use kami_types::{Capability, FsAccess, SecurityConfig};

use crate::error::SandboxError;
use crate::network::is_host_allowed;

/// Trait for checking capabilities against a security config.
pub trait CapabilityChecker: Send + Sync {
    /// Checks whether a capability is allowed by the security config.
    fn check(&self, capability: &Capability, config: &SecurityConfig) -> Result<(), SandboxError>;
}

/// Default capability checker enforcing deny-all by default.
pub struct DefaultCapabilityChecker;

impl CapabilityChecker for DefaultCapabilityChecker {
    fn check(&self, capability: &Capability, config: &SecurityConfig) -> Result<(), SandboxError> {
        match capability {
            Capability::Network(host) => {
                if !is_host_allowed(host, &config.net_allow_list) {
                    return Err(SandboxError::NetworkDenied { host: host.clone() });
                }
            }
            Capability::FsRead(path) => {
                if config.fs_access == FsAccess::None {
                    return Err(SandboxError::FsDenied { path: path.clone() });
                }
            }
            Capability::FsWrite(path) => {
                if config.fs_access != FsAccess::Sandbox {
                    return Err(SandboxError::FsDenied { path: path.clone() });
                }
            }
            Capability::EnvVar(var) => {
                if !config.env_allow_list.iter().any(|v| v == var) {
                    return Err(SandboxError::CapabilityDenied {
                        capability: format!("env:{var}"),
                    });
                }
            }
        }
        Ok(())
    }
}

/// Validates a `SecurityConfig` for well-formedness before use.
///
/// Catches misconfigurations early rather than at execution time.
pub fn validate_security_config(config: &SecurityConfig) -> Result<(), SandboxError> {
    // Validate network patterns
    crate::network::validate_allow_list(&config.net_allow_list)
        .map_err(|reason| SandboxError::InvalidConfig { reason })?;

    // Validate resource limits
    if config.limits.max_fuel == 0 {
        return Err(SandboxError::InvalidConfig {
            reason: "max_fuel must be > 0".to_string(),
        });
    }
    if config.limits.max_memory_mb == 0 {
        return Err(SandboxError::InvalidConfig {
            reason: "max_memory_mb must be > 0".to_string(),
        });
    }
    if config.limits.max_execution_ms == 0 {
        return Err(SandboxError::InvalidConfig {
            reason: "max_execution_ms must be > 0".to_string(),
        });
    }

    Ok(())
}
