//! # kami-config
//!
//! Configuration management for KAMI.
//! Supports layered config: defaults -> file -> env vars.

pub mod loader;
pub mod manifest_loader;
pub mod schema;

pub use loader::{load_config, ConfigError};
pub use manifest_loader::{parse_tool_manifest, parse_tool_manifest_file, ManifestError};
pub use schema::KamiConfig;
