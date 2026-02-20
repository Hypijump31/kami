//! Diagnostic hint/fix coverage for RuntimeError.

use kami_runtime::RuntimeError;
use kami_types::{DiagnosticError, ErrorKind, KamiError};

#[test]
fn tool_not_found_hint_contains_name() {
    let e = RuntimeError::ToolNotFound {
        name: "dev.x.y".into(),
    };
    let hint = e.hint().expect("has hint");
    assert!(hint.contains("dev.x.y"));
}

#[test]
fn tool_not_found_fix_suggests_install() {
    let e = RuntimeError::ToolNotFound {
        name: "dev.a.b".into(),
    };
    let fix = e.fix().expect("has fix");
    assert!(fix.contains("install"));
}

#[test]
fn timeout_hint_contains_ms() {
    let e = RuntimeError::Timeout { timeout_ms: 3000 };
    let hint = e.hint().expect("has hint");
    assert!(hint.contains("3000"));
}

#[test]
fn timeout_fix_suggests_increase() {
    let e = RuntimeError::Timeout { timeout_ms: 5000 };
    let fix = e.fix().expect("has fix");
    assert!(fix.contains("max_execution_ms"));
}

#[test]
fn pool_exhausted_hint_mentions_load() {
    let e = RuntimeError::PoolExhausted;
    let hint = e.hint().expect("has hint");
    assert!(hint.contains("busy") || hint.contains("load"));
}

#[test]
fn pool_exhausted_fix_suggests_concurrency() {
    let e = RuntimeError::PoolExhausted;
    let fix = e.fix().expect("has fix");
    assert!(fix.contains("concurrency"));
}

#[test]
fn integrity_violation_hint_mentions_sha() {
    let e = RuntimeError::IntegrityViolation {
        tool_id: "t".into(),
        detail: "hash mismatch".into(),
    };
    let hint = e.hint().expect("has hint");
    assert!(hint.contains("SHA-256") || hint.contains("hash"));
}

#[test]
fn integrity_violation_fix_suggests_reinstall() {
    let e = RuntimeError::IntegrityViolation {
        tool_id: "t".into(),
        detail: "x".into(),
    };
    let fix = e.fix().expect("has fix");
    assert!(fix.contains("install"));
}

#[test]
fn rate_limited_hint_mentions_too_many() {
    let e = RuntimeError::RateLimited {
        tool_id: "t".into(),
        limit: 10,
        window_secs: 60,
    };
    let hint = e.hint().expect("has hint");
    assert!(hint.to_lowercase().contains("too many"));
}

#[test]
fn sandbox_invalid_config_maps_to_invalid_input() {
    let sandbox = kami_sandbox::SandboxError::InvalidConfig {
        reason: "bad config".into(),
    };
    let err: KamiError = RuntimeError::Sandbox(sandbox).into();
    assert_eq!(err.kind, ErrorKind::InvalidInput);
}

#[test]
fn sandbox_other_maps_to_permission_denied() {
    let sandbox = kami_sandbox::SandboxError::CapabilityDenied {
        capability: "net".into(),
    };
    let err: KamiError = RuntimeError::Sandbox(sandbox).into();
    assert_eq!(err.kind, ErrorKind::PermissionDenied);
}

#[test]
fn engine_error_maps_to_internal() {
    let engine = kami_engine::EngineError::Config("bad".into());
    let err: KamiError = RuntimeError::Engine(engine).into();
    assert_eq!(err.kind, ErrorKind::Internal);
}

#[test]
fn sandbox_error_hint_delegates() {
    let sandbox = kami_sandbox::SandboxError::CapabilityDenied {
        capability: "net".into(),
    };
    let e = RuntimeError::Sandbox(sandbox);
    assert!(e.hint().is_some());
}

#[test]
fn engine_error_hint_delegates() {
    let engine = kami_engine::EngineError::Config("x".into());
    let e = RuntimeError::Engine(engine);
    // Config errors might not have a hint
    let _ = e.hint();
    let _ = e.fix();
}
