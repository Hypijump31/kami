//! Isolation and security integration tests for the KAMI engine.

mod common;

use kami_engine::{create_engine, create_linker, load_component, InstanceConfig};
use kami_types::{ResourceLimits, SecurityConfig};

use common::ECHO_COMPONENT_WAT;

#[tokio::test]
async fn executor_rejects_invalid_config() {
    use kami_runtime::{ToolExecutor, WasmToolExecutor};

    let config = InstanceConfig {
        epoch_interruption: true,
        ..InstanceConfig::default()
    };
    let engine = create_engine(&config).expect("engine");
    let linker = create_linker(&engine).expect("linker");
    let component = load_component(&engine, ECHO_COMPONENT_WAT.as_bytes()).expect("component");

    let executor = WasmToolExecutor::new(engine, linker);
    let bad_security = SecurityConfig {
        limits: ResourceLimits {
            max_fuel: 0,
            ..ResourceLimits::default()
        },
        ..SecurityConfig::default()
    };

    let result = executor.execute(&component, "{}", &bad_security).await;
    assert!(result.is_err(), "should reject zero fuel config");
}

#[tokio::test]
async fn executor_uses_security_config_fuel() {
    use kami_runtime::{ToolExecutor, WasmToolExecutor};

    let config = InstanceConfig {
        epoch_interruption: true,
        ..InstanceConfig::default()
    };
    let engine = create_engine(&config).expect("engine");
    let linker = create_linker(&engine).expect("linker");
    let component = load_component(&engine, ECHO_COMPONENT_WAT.as_bytes()).expect("component");

    let executor = WasmToolExecutor::new(engine, linker);
    let security = SecurityConfig {
        limits: ResourceLimits {
            max_fuel: 500_000,
            ..ResourceLimits::default()
        },
        ..SecurityConfig::default()
    };

    let result = executor
        .execute(&component, "test", &security)
        .await
        .expect("execution");

    assert!(result.success);
    assert!(result.fuel_consumed > 0);
    assert!(result.fuel_consumed < 500_000);
}

#[tokio::test]
async fn executor_with_memory_limits() {
    use kami_runtime::{ToolExecutor, WasmToolExecutor};

    let config = InstanceConfig {
        epoch_interruption: true,
        ..InstanceConfig::default()
    };
    let engine = create_engine(&config).expect("engine");
    let linker = create_linker(&engine).expect("linker");
    let component = load_component(&engine, ECHO_COMPONENT_WAT.as_bytes()).expect("component");

    let executor = WasmToolExecutor::new(engine, linker);
    let security = SecurityConfig {
        limits: ResourceLimits {
            max_memory_mb: 16,
            ..ResourceLimits::default()
        },
        ..SecurityConfig::default()
    };

    let result = executor
        .execute(&component, "memory test", &security)
        .await
        .expect("should succeed with 16MB");

    assert!(result.success);
    assert_eq!(result.content, "memory test");
}

#[tokio::test]
async fn capability_checker_denies_unlisted_env() {
    use kami_sandbox::{CapabilityChecker, DefaultCapabilityChecker};
    use kami_types::Capability;

    let checker = DefaultCapabilityChecker;
    let config = SecurityConfig {
        env_allow_list: vec!["LANG".to_string()],
        ..SecurityConfig::default()
    };

    assert!(checker
        .check(&Capability::EnvVar("LANG".to_string()), &config)
        .is_ok());
    assert!(checker
        .check(&Capability::EnvVar("SECRET".to_string()), &config)
        .is_err());

    let deny_config = SecurityConfig::default();
    assert!(checker
        .check(&Capability::EnvVar("HOME".to_string()), &deny_config)
        .is_err());
}

#[tokio::test]
async fn validate_security_config_catches_issues() {
    use kami_sandbox::validate_security_config;

    assert!(validate_security_config(&SecurityConfig::default()).is_ok());

    assert!(validate_security_config(&SecurityConfig {
        limits: ResourceLimits {
            max_memory_mb: 0,
            ..ResourceLimits::default()
        },
        ..SecurityConfig::default()
    })
    .is_err());

    assert!(validate_security_config(&SecurityConfig {
        limits: ResourceLimits {
            max_execution_ms: 0,
            ..ResourceLimits::default()
        },
        ..SecurityConfig::default()
    })
    .is_err());
}
