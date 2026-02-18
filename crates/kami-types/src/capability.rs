//! Capability-based security types.

use serde::{Deserialize, Serialize};

/// Filesystem access level.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FsAccess {
    /// No filesystem access.
    #[default]
    None,
    /// Read-only access within sandbox.
    ReadOnly,
    /// Read-write within sandbox directory.
    Sandbox,
}

/// Resource limits for a WASM instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in megabytes.
    #[serde(default = "default_max_memory_mb")]
    pub max_memory_mb: u32,
    /// Maximum execution time in milliseconds.
    #[serde(default = "default_max_execution_ms")]
    pub max_execution_ms: u64,
    /// Maximum fuel (instruction budget).
    #[serde(default = "default_max_fuel")]
    pub max_fuel: u64,
}

fn default_max_memory_mb() -> u32 {
    64
}
fn default_max_execution_ms() -> u64 {
    5000
}
fn default_max_fuel() -> u64 {
    1_000_000
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: default_max_memory_mb(),
            max_execution_ms: default_max_execution_ms(),
            max_fuel: default_max_fuel(),
        }
    }
}

/// Security configuration for a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Allowed network destinations (glob patterns).
    #[serde(default)]
    pub net_allow_list: Vec<String>,
    /// Filesystem access level.
    #[serde(default)]
    pub fs_access: FsAccess,
    /// Allowed environment variable names (exact match, deny-all by default).
    #[serde(default)]
    pub env_allow_list: Vec<String>,
    /// Resource limits.
    #[serde(flatten)]
    pub limits: ResourceLimits,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            net_allow_list: Vec::new(),
            fs_access: FsAccess::None,
            env_allow_list: Vec::new(),
            limits: ResourceLimits::default(),
        }
    }
}

/// A single capability granted to a tool instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Network access to a specific pattern.
    Network(String),
    /// Filesystem read access to a path.
    FsRead(String),
    /// Filesystem write access to a path.
    FsWrite(String),
    /// Environment variable access.
    EnvVar(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_security_is_deny_all() {
        let config = SecurityConfig::default();
        assert!(config.net_allow_list.is_empty());
        assert!(config.env_allow_list.is_empty());
        assert_eq!(config.fs_access, FsAccess::None);
    }

    #[test]
    fn resource_limits_defaults() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_mb, 64);
        assert_eq!(limits.max_execution_ms, 5000);
        assert_eq!(limits.max_fuel, 1_000_000);
    }

    #[test]
    fn fs_access_serialization() {
        let access = FsAccess::ReadOnly;
        let json = serde_json::to_string(&access).unwrap();
        assert_eq!(json, "\"read-only\"");
    }
}
