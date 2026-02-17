//! Configuration schema types.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Top-level KAMI configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KamiConfig {
    /// Runtime settings.
    #[serde(default)]
    pub runtime: RuntimeConfig,
    /// Sandbox settings.
    #[serde(default)]
    pub sandbox: SandboxConfig,
    /// Registry settings.
    #[serde(default)]
    pub registry: RegistryConfig,
    /// Logging settings.
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// Runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Maximum concurrent tool executions.
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,
    /// Instance pool size.
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,
    /// Default execution timeout in seconds.
    #[serde(default = "default_timeout_secs")]
    pub default_timeout_secs: u64,
}

impl RuntimeConfig {
    /// Returns the timeout as a `Duration`.
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.default_timeout_secs)
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrent: default_max_concurrent(),
            pool_size: default_pool_size(),
            default_timeout_secs: default_timeout_secs(),
        }
    }
}

fn default_max_concurrent() -> usize {
    10
}
fn default_pool_size() -> usize {
    5
}
fn default_timeout_secs() -> u64 {
    30
}

/// Sandbox default settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Default maximum memory per tool (MB).
    #[serde(default = "default_max_memory")]
    pub default_max_memory_mb: u32,
    /// Default maximum fuel per tool.
    #[serde(default = "default_max_fuel")]
    pub default_max_fuel: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            default_max_memory_mb: default_max_memory(),
            default_max_fuel: default_max_fuel(),
        }
    }
}

fn default_max_memory() -> u32 {
    64
}
fn default_max_fuel() -> u64 {
    1_000_000
}

/// Registry storage settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Path to the SQLite database.
    #[serde(default = "default_db_path")]
    pub database_path: String,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            database_path: default_db_path(),
        }
    }
}

fn default_db_path() -> String {
    "kami.db".to_string()
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level filter (e.g. "info", "debug", "kami=trace").
    #[serde(default = "default_log_level")]
    pub level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}
