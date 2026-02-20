//! Integration tests for kami-config schema types.

use kami_config::schema::{
    KamiConfig, LoggingConfig, RegistryConfig, RuntimeConfig, SandboxConfig,
};
use std::time::Duration;

#[test]
fn kami_config_default_values() {
    let config = KamiConfig::default();
    assert_eq!(config.runtime.max_concurrent, 10);
    assert_eq!(config.runtime.pool_size, 5);
    assert_eq!(config.runtime.default_timeout_secs, 30);
    assert_eq!(config.sandbox.default_max_memory_mb, 64);
    assert_eq!(config.sandbox.default_max_fuel, 1_000_000);
    assert_eq!(config.registry.database_path, "kami.db");
    assert_eq!(config.logging.level, "info");
}

#[test]
fn kami_config_serde_roundtrip() {
    let config = KamiConfig::default();
    let json = serde_json::to_string(&config).expect("serialize");
    let back: KamiConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.runtime.max_concurrent, config.runtime.max_concurrent);
    assert_eq!(
        back.sandbox.default_max_fuel,
        config.sandbox.default_max_fuel
    );
}

#[test]
fn runtime_timeout_returns_correct_duration() {
    let rt = RuntimeConfig {
        max_concurrent: 1,
        pool_size: 1,
        default_timeout_secs: 45,
    };
    assert_eq!(rt.timeout(), Duration::from_secs(45));
}

#[test]
fn runtime_default_timeout_30s() {
    let rt = RuntimeConfig::default();
    assert_eq!(rt.timeout(), Duration::from_secs(30));
}

#[test]
fn registry_default_path() {
    let reg = RegistryConfig::default();
    assert_eq!(reg.database_path, "kami.db");
}

#[test]
fn logging_default_level() {
    let log = LoggingConfig::default();
    assert_eq!(log.level, "info");
}

#[test]
fn sandbox_default_values() {
    let sb = SandboxConfig::default();
    assert_eq!(sb.default_max_memory_mb, 64);
    assert_eq!(sb.default_max_fuel, 1_000_000);
}

#[test]
fn deny_unknown_fields_rejects_extra_key() {
    let json = r#"{"runtime":{},"sandbox":{},"registry":{},"logging":{},"unknown_key":"bad"}"#;
    let result: Result<KamiConfig, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn partial_config_uses_defaults_for_missing() {
    let json = r#"{"runtime":{"max_concurrent":20}}"#;
    let config: KamiConfig = serde_json::from_str(json).expect("parse");
    assert_eq!(config.runtime.max_concurrent, 20);
    assert_eq!(config.runtime.pool_size, 5); // default
    assert_eq!(config.sandbox.default_max_memory_mb, 64); // default
}
