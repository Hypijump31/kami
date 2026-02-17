//! Capability checking trait and types.

use kami_types::{Capability, SecurityConfig};

use crate::error::SandboxError;
use crate::network::is_host_allowed;

/// Trait for checking capabilities against a security config.
pub trait CapabilityChecker: Send + Sync {
    /// Checks whether a capability is allowed.
    fn check(
        &self,
        capability: &Capability,
        config: &SecurityConfig,
    ) -> Result<(), SandboxError>;
}

/// Default capability checker enforcing deny-all by default.
pub struct DefaultCapabilityChecker;

impl CapabilityChecker for DefaultCapabilityChecker {
    fn check(
        &self,
        capability: &Capability,
        config: &SecurityConfig,
    ) -> Result<(), SandboxError> {
        match capability {
            Capability::Network(host) => {
                if !is_host_allowed(host, &config.net_allow_list) {
                    return Err(SandboxError::NetworkDenied {
                        host: host.clone(),
                    });
                }
            }
            Capability::FsRead(path) | Capability::FsWrite(path) => {
                if config.fs_access == kami_types::FsAccess::None {
                    return Err(SandboxError::FsDenied { path: path.clone() });
                }
                if matches!(capability, Capability::FsWrite(_))
                    && config.fs_access == kami_types::FsAccess::ReadOnly
                {
                    return Err(SandboxError::FsDenied { path: path.clone() });
                }
            }
            Capability::EnvVar(var) => {
                return Err(SandboxError::CapabilityDenied {
                    capability: format!("env:{var}"),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_checker_denies_network() {
        let checker = DefaultCapabilityChecker;
        let config = SecurityConfig::default();
        let cap = Capability::Network("example.com".to_string());
        assert!(checker.check(&cap, &config).is_err());
    }

    #[test]
    fn checker_allows_listed_host() {
        let checker = DefaultCapabilityChecker;
        let config = SecurityConfig {
            net_allow_list: vec!["api.github.com".to_string()],
            ..SecurityConfig::default()
        };
        let cap = Capability::Network("api.github.com".to_string());
        assert!(checker.check(&cap, &config).is_ok());
    }

    #[test]
    fn checker_denies_fs_when_none() {
        let checker = DefaultCapabilityChecker;
        let config = SecurityConfig::default();
        let cap = Capability::FsRead("/etc/passwd".to_string());
        assert!(checker.check(&cap, &config).is_err());
    }

    #[test]
    fn checker_denies_env_always() {
        let checker = DefaultCapabilityChecker;
        let config = SecurityConfig::default();
        let cap = Capability::EnvVar("SECRET".to_string());
        assert!(checker.check(&cap, &config).is_err());
    }
}
