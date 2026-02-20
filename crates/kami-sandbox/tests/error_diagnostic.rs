//! Diagnostic error tests for SandboxError hint/fix coverage.

use kami_sandbox::SandboxError;
use kami_types::{DiagnosticError, ErrorKind, KamiError};

#[test]
fn wasi_build_maps_to_internal() {
    let err: KamiError = SandboxError::WasiBuild {
        reason: "oom".into(),
    }
    .into();
    assert_eq!(err.kind, ErrorKind::Internal);
}

#[test]
fn capability_denied_hint_contains_capability_name() {
    let e = SandboxError::CapabilityDenied {
        capability: "network".into(),
    };
    let hint = e.hint().expect("has hint");
    assert!(hint.contains("network"));
}

#[test]
fn capability_denied_fix_mentions_tool_toml() {
    let e = SandboxError::CapabilityDenied {
        capability: "net".into(),
    };
    let fix = e.fix().expect("has fix");
    assert!(fix.contains("tool.toml"));
}

#[test]
fn fs_denied_hint_mentions_path() {
    let e = SandboxError::FsDenied {
        path: "/etc/passwd".into(),
    };
    let hint = e.hint().expect("has hint");
    assert!(hint.contains("/etc/passwd"));
}

#[test]
fn fs_denied_fix_mentions_fs_access() {
    let e = SandboxError::FsDenied {
        path: "/tmp".into(),
    };
    let fix = e.fix().expect("has fix");
    assert!(fix.contains("fs_access"));
}

#[test]
fn invalid_config_hint_mentions_tool_toml() {
    let e = SandboxError::InvalidConfig {
        reason: "bad value".into(),
    };
    let hint = e.hint().expect("has hint");
    assert!(hint.contains("tool.toml"));
}

#[test]
fn invalid_config_has_no_fix() {
    let e = SandboxError::InvalidConfig { reason: "x".into() };
    assert!(e.fix().is_none());
}

#[test]
fn wasi_build_has_no_hint() {
    let e = SandboxError::WasiBuild {
        reason: "oom".into(),
    };
    assert!(e.hint().is_none());
}
