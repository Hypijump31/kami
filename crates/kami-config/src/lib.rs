//! # kami-config
//!
//! Configuration management for KAMI.
//! Supports layered config: defaults -> file -> env vars.

pub mod loader;
pub mod schema;

pub use loader::{load_config, ConfigError};
pub use schema::KamiConfig;
