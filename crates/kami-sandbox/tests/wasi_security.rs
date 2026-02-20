//! Additional tests for WASI context building (security-focused).

use kami_sandbox::{build_wasi_ctx, WasiConfig};
use kami_types::{FsAccess, SecurityConfig};

#[test]
fn env_allow_list_blocks_unlisted_var() {
    let security = SecurityConfig {
        env_allow_list: vec!["ALLOWED".to_string()],
        ..SecurityConfig::default()
    };
    let wasi_config = WasiConfig {
        env_vars: vec![
            ("ALLOWED".to_string(), "yes".to_string()),
            ("BLOCKED".to_string(), "no".to_string()),
        ],
        ..WasiConfig::default()
    };
    // Should succeed — blocked var is skipped silently
    let ctx = build_wasi_ctx(&security, &wasi_config, None);
    assert!(ctx.is_ok());
}

#[test]
fn empty_allow_list_permits_all_env_vars() {
    let security = SecurityConfig {
        env_allow_list: vec![],
        ..SecurityConfig::default()
    };
    let wasi_config = WasiConfig {
        env_vars: vec![("FOO".to_string(), "bar".to_string())],
        ..WasiConfig::default()
    };
    let ctx = build_wasi_ctx(&security, &wasi_config, None);
    assert!(ctx.is_ok());
}

#[test]
fn fs_none_no_sandbox_dir_succeeds() {
    let security = SecurityConfig {
        fs_access: FsAccess::None,
        ..SecurityConfig::default()
    };
    let ctx = build_wasi_ctx(&security, &WasiConfig::default(), None);
    assert!(ctx.is_ok());
}

#[test]
fn fs_read_only_without_dir_succeeds() {
    let security = SecurityConfig {
        fs_access: FsAccess::ReadOnly,
        ..SecurityConfig::default()
    };
    // No sandbox_dir → no preopened dir, but no error
    let ctx = build_wasi_ctx(&security, &WasiConfig::default(), None);
    assert!(ctx.is_ok());
}

#[test]
fn fs_sandbox_without_dir_succeeds() {
    let security = SecurityConfig {
        fs_access: FsAccess::Sandbox,
        ..SecurityConfig::default()
    };
    let ctx = build_wasi_ctx(&security, &WasiConfig::default(), None);
    assert!(ctx.is_ok());
}

#[test]
fn network_allow_list_with_patterns_creates_ctx() {
    let security = SecurityConfig {
        net_allow_list: vec!["*.example.com".to_string()],
        ..SecurityConfig::default()
    };
    let ctx = build_wasi_ctx(&security, &WasiConfig::default(), None);
    assert!(ctx.is_ok());
}

#[test]
fn inherit_stdout_and_stderr() {
    let security = SecurityConfig::default();
    let wasi_config = WasiConfig {
        inherit_stdout: true,
        inherit_stderr: true,
        ..WasiConfig::default()
    };
    let ctx = build_wasi_ctx(&security, &wasi_config, None);
    assert!(ctx.is_ok());
}

#[test]
fn fs_read_only_with_real_dir_creates_ctx() {
    let dir = std::env::temp_dir().join("kami_wasi_ro_test");
    let _ = std::fs::create_dir_all(&dir);
    let security = SecurityConfig {
        fs_access: FsAccess::ReadOnly,
        ..SecurityConfig::default()
    };
    let ctx = build_wasi_ctx(
        &security,
        &WasiConfig::default(),
        Some(dir.to_str().expect("utf8")),
    );
    assert!(ctx.is_ok());
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn fs_sandbox_with_real_dir_creates_ctx() {
    let dir = std::env::temp_dir().join("kami_wasi_rw_test");
    let _ = std::fs::create_dir_all(&dir);
    let security = SecurityConfig {
        fs_access: FsAccess::Sandbox,
        ..SecurityConfig::default()
    };
    let ctx = build_wasi_ctx(
        &security,
        &WasiConfig::default(),
        Some(dir.to_str().expect("utf8")),
    );
    assert!(ctx.is_ok());
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn network_multiple_patterns_creates_ctx() {
    let security = SecurityConfig {
        net_allow_list: vec![
            "api.github.com".to_string(),
            "*.example.com".to_string(),
            "10.0.0.1".to_string(),
        ],
        ..SecurityConfig::default()
    };
    let ctx = build_wasi_ctx(&security, &WasiConfig::default(), None);
    assert!(ctx.is_ok());
}
