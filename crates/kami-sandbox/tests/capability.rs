//! Tests for capability checking and security config validation.

use kami_sandbox::{validate_security_config, CapabilityChecker, DefaultCapabilityChecker};
use kami_types::{Capability, FsAccess, ResourceLimits, SecurityConfig};

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
fn checker_denies_fs_read_when_none() {
    let checker = DefaultCapabilityChecker;
    let config = SecurityConfig::default();
    let cap = Capability::FsRead("/etc/passwd".to_string());
    assert!(checker.check(&cap, &config).is_err());
}

#[test]
fn checker_allows_fs_read_when_readonly() {
    let checker = DefaultCapabilityChecker;
    let config = SecurityConfig {
        fs_access: FsAccess::ReadOnly,
        ..SecurityConfig::default()
    };
    let cap = Capability::FsRead("/data/file.txt".to_string());
    assert!(checker.check(&cap, &config).is_ok());
}

#[test]
fn checker_denies_fs_write_when_readonly() {
    let checker = DefaultCapabilityChecker;
    let config = SecurityConfig {
        fs_access: FsAccess::ReadOnly,
        ..SecurityConfig::default()
    };
    let cap = Capability::FsWrite("/data/file.txt".to_string());
    assert!(checker.check(&cap, &config).is_err());
}

#[test]
fn checker_allows_fs_write_when_sandbox() {
    let checker = DefaultCapabilityChecker;
    let config = SecurityConfig {
        fs_access: FsAccess::Sandbox,
        ..SecurityConfig::default()
    };
    let cap = Capability::FsWrite("/sandbox/out.txt".to_string());
    assert!(checker.check(&cap, &config).is_ok());
}

#[test]
fn checker_denies_env_when_not_listed() {
    let checker = DefaultCapabilityChecker;
    let config = SecurityConfig::default();
    let cap = Capability::EnvVar("SECRET".to_string());
    assert!(checker.check(&cap, &config).is_err());
}

#[test]
fn checker_allows_env_when_listed() {
    let checker = DefaultCapabilityChecker;
    let config = SecurityConfig {
        env_allow_list: vec!["LANG".to_string(), "HOME".to_string()],
        ..SecurityConfig::default()
    };
    let cap = Capability::EnvVar("LANG".to_string());
    assert!(checker.check(&cap, &config).is_ok());
}

#[test]
fn validate_default_config_is_ok() {
    let config = SecurityConfig::default();
    assert!(validate_security_config(&config).is_ok());
}

#[test]
fn validate_rejects_zero_fuel() {
    let config = SecurityConfig {
        limits: ResourceLimits {
            max_fuel: 0,
            ..ResourceLimits::default()
        },
        ..SecurityConfig::default()
    };
    assert!(validate_security_config(&config).is_err());
}

#[test]
fn validate_rejects_zero_memory() {
    let config = SecurityConfig {
        limits: ResourceLimits {
            max_memory_mb: 0,
            ..ResourceLimits::default()
        },
        ..SecurityConfig::default()
    };
    assert!(validate_security_config(&config).is_err());
}

#[test]
fn validate_rejects_zero_execution_ms() {
    let config = SecurityConfig {
        limits: ResourceLimits {
            max_execution_ms: 0,
            ..ResourceLimits::default()
        },
        ..SecurityConfig::default()
    };
    assert!(validate_security_config(&config).is_err());
}

#[test]
fn validate_rejects_empty_net_pattern() {
    let config = SecurityConfig {
        net_allow_list: vec!["".to_string()],
        ..SecurityConfig::default()
    };
    assert!(validate_security_config(&config).is_err());
}

#[test]
fn checker_denies_unlisted_host_with_wildcard() {
    let checker = DefaultCapabilityChecker;
    let config = SecurityConfig {
        net_allow_list: vec!["*.example.com".to_string()],
        ..SecurityConfig::default()
    };
    let cap = Capability::Network("evil.com".to_string());
    assert!(checker.check(&cap, &config).is_err());
}

#[test]
fn checker_allows_wildcard_subdomain() {
    let checker = DefaultCapabilityChecker;
    let config = SecurityConfig {
        net_allow_list: vec!["*.example.com".to_string()],
        ..SecurityConfig::default()
    };
    let cap = Capability::Network("api.example.com".to_string());
    assert!(checker.check(&cap, &config).is_ok());
}

#[test]
fn checker_denies_env_not_in_allow_list() {
    let checker = DefaultCapabilityChecker;
    let config = SecurityConfig {
        env_allow_list: vec!["ALLOWED".to_string()],
        ..SecurityConfig::default()
    };
    let cap = Capability::EnvVar("SECRET_KEY".to_string());
    assert!(checker.check(&cap, &config).is_err());
}
