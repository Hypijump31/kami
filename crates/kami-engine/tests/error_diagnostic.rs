//! Engine diagnostic hint/fix coverage.

use kami_engine::EngineError;
use kami_types::{DiagnosticError, ErrorKind, KamiError};

#[test]
fn trap_maps_to_internal() {
    let err: KamiError = EngineError::Trap {
        message: "out of fuel".into(),
    }
    .into();
    assert_eq!(err.kind, ErrorKind::Internal);
}

#[test]
fn config_error_maps_to_internal() {
    let err: KamiError = EngineError::Config("bad".into()).into();
    assert_eq!(err.kind, ErrorKind::Internal);
}

#[test]
fn compilation_generic_hint() {
    let e = EngineError::Compilation {
        reason: "invalid wasm".into(),
        source: wasmtime::Error::msg("test"),
    };
    let hint = e.hint().expect("has hint");
    assert!(hint.contains("compiled"));
}

#[test]
fn compilation_unknown_import_fix() {
    let e = EngineError::Compilation {
        reason: "unknown import: wasi:http".into(),
        source: wasmtime::Error::msg("test"),
    };
    let fix = e.fix().expect("has fix");
    assert!(fix.contains("tool.toml"));
}

#[test]
fn export_not_found_hint_mentions_export() {
    let e = EngineError::ExportNotFound { name: "run".into() };
    let hint = e.hint().expect("has hint");
    assert!(hint.contains("run"));
}

#[test]
fn export_not_found_fix_mentions_macro() {
    let e = EngineError::ExportNotFound { name: "run".into() };
    let fix = e.fix().expect("has fix");
    assert!(fix.contains("kami_tool"));
}

#[test]
fn resource_exceeded_hint_mentions_budget() {
    let e = EngineError::ResourceExceeded {
        limit: "memory".into(),
    };
    let hint = e.hint().expect("has hint");
    assert!(hint.contains("budget") || hint.contains("exceeded"));
}

#[test]
fn resource_exceeded_fix_mentions_increase() {
    let e = EngineError::ResourceExceeded {
        limit: "max_memory_mb = 128".into(),
    };
    let fix = e.fix().expect("has fix");
    assert!(fix.contains("max_memory_mb"));
}

#[test]
fn trap_hint_mentions_trapped() {
    let e = EngineError::Trap {
        message: "unreachable".into(),
    };
    let hint = e.hint().expect("has hint");
    assert!(hint.to_lowercase().contains("trap"));
}

#[test]
fn config_error_has_no_hint() {
    let e = EngineError::Config("bad value".into());
    assert!(e.hint().is_none());
}

#[test]
fn config_error_has_no_fix() {
    let e = EngineError::Config("bad".into());
    assert!(e.fix().is_none());
}
