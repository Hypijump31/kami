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
