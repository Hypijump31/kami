//! Configuration loader (file + env + CLI merge).

use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;
use thiserror::Error;

use crate::schema::KamiConfig;

/// Errors from configuration loading.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to load or merge configuration.
    #[error("configuration error: {0}")]
    Load(String),
}

/// Loads configuration by merging layers:
/// 1. Default values
/// 2. Config file (if exists)
/// 3. Environment variables (KAMI_ prefix)
pub fn load_config(config_path: Option<&str>) -> Result<KamiConfig, ConfigError> {
    let mut figment = Figment::from(Serialized::defaults(KamiConfig::default()));

    if let Some(path) = config_path {
        figment = figment.merge(Toml::file(path));
    }

    figment = figment.merge(Env::prefixed("KAMI_").split("_"));

    figment
        .extract()
        .map_err(|e| ConfigError::Load(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_config_no_path_succeeds() {
        let config = load_config(None);
        assert!(config.is_ok(), "default config should load without error");
    }

    #[test]
    fn load_config_default_values() {
        let config = load_config(None).expect("should load");
        assert_eq!(config.runtime.max_concurrent, 10);
        assert_eq!(config.runtime.pool_size, 5);
        assert_eq!(config.runtime.default_timeout_secs, 30);
        assert_eq!(config.sandbox.default_max_memory_mb, 64);
        assert_eq!(config.sandbox.default_max_fuel, 1_000_000);
        assert_eq!(config.registry.database_path, "kami.db");
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn load_config_nonexistent_file_falls_back_to_defaults() {
        // figment::Toml::file ignores missing files (optional by default)
        let config = load_config(Some("/nonexistent/path/kami.toml"));
        assert!(
            config.is_ok(),
            "missing config file should fall back to defaults"
        );
    }

    #[test]
    fn runtime_timeout_returns_duration() {
        let config = load_config(None).expect("should load");
        assert_eq!(config.runtime.timeout().as_secs(), 30);
    }
}
