//! WASI context builder for sandboxed execution.
//!
//! Builds a `WasiCtx` from a `SecurityConfig`, enforcing deny-all defaults
//! and granular permissions for network and filesystem access.

use std::sync::Arc;

use kami_types::{FsAccess, SecurityConfig};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtx, WasiCtxBuilder};

use crate::error::SandboxError;
use crate::network::is_addr_allowed;

/// Options controlling WASI context construction.
#[derive(Debug, Clone, Default)]
pub struct WasiConfig {
    /// Whether to inherit stdout (useful for CLI tools).
    pub inherit_stdout: bool,
    /// Whether to inherit stderr (useful for debugging).
    pub inherit_stderr: bool,
    /// Extra environment variables to expose.
    pub env_vars: Vec<(String, String)>,
}

/// Builds a `WasiCtx` from a `SecurityConfig` and optional overrides.
///
/// Enforces:
/// - Network: deny-all unless hosts match `net_allow_list` patterns
/// - Filesystem: deny-all / read-only / sandbox based on `fs_access`
/// - Env vars: only explicit vars from `wasi_config`
/// - DNS: disabled unless network is allowed
pub fn build_wasi_ctx(
    security: &SecurityConfig,
    wasi_config: &WasiConfig,
    sandbox_dir: Option<&str>,
) -> Result<WasiCtx, SandboxError> {
    let mut builder = WasiCtxBuilder::new();

    // -- stdio --
    if wasi_config.inherit_stdout {
        builder.inherit_stdout();
    }
    if wasi_config.inherit_stderr {
        builder.inherit_stderr();
    }

    // -- environment variables (filtered by env_allow_list) --
    // If env_allow_list is non-empty, only listed vars are exposed.
    // If env_allow_list is empty, all explicitly provided vars are allowed.
    for (key, value) in &wasi_config.env_vars {
        if !security.env_allow_list.is_empty() && !security.env_allow_list.contains(key) {
            tracing::warn!(
                key = %key,
                "env var blocked by allow-list"
            );
            continue;
        }
        builder.env(key, value);
    }

    // -- filesystem --
    configure_filesystem(&mut builder, security, sandbox_dir)?;

    // -- network --
    configure_network(&mut builder, security);

    Ok(builder.build())
}

/// Configures filesystem access based on security policy.
fn configure_filesystem(
    builder: &mut WasiCtxBuilder,
    security: &SecurityConfig,
    sandbox_dir: Option<&str>,
) -> Result<(), SandboxError> {
    match security.fs_access {
        FsAccess::None => {
            // No filesystem access - nothing to configure
        }
        FsAccess::ReadOnly => {
            if let Some(dir) = sandbox_dir {
                builder
                    .preopened_dir(dir, ".", DirPerms::READ, FilePerms::READ)
                    .map_err(|e| SandboxError::WasiBuild {
                        reason: format!("failed to preopen read-only dir: {e}"),
                    })?;
            }
        }
        FsAccess::Sandbox => {
            if let Some(dir) = sandbox_dir {
                builder
                    .preopened_dir(dir, ".", DirPerms::all(), FilePerms::all())
                    .map_err(|e| SandboxError::WasiBuild {
                        reason: format!("failed to preopen sandbox dir: {e}"),
                    })?;
            }
        }
    }
    Ok(())
}

/// Configures network access using `socket_addr_check` for granular control.
fn configure_network(builder: &mut WasiCtxBuilder, security: &SecurityConfig) {
    let allow_list = security.net_allow_list.clone();
    let has_network = !allow_list.is_empty();

    if has_network {
        let patterns = Arc::new(allow_list);
        builder.socket_addr_check(move |addr, _addr_use| {
            let patterns = Arc::clone(&patterns);
            Box::pin(async move {
                // Use is_addr_allowed: direct IP connections require explicit
                // IP in the allow list — hostname patterns do not match IPs.
                is_addr_allowed(&addr, &patterns)
            })
        });
        builder.allow_ip_name_lookup(true);
    }
    // If no allow_list: network is deny-all by default (no inherit_network)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kami_types::SecurityConfig;

    #[test]
    fn default_security_produces_ctx() {
        let security = SecurityConfig::default();
        let wasi_config = WasiConfig::default();
        let ctx = build_wasi_ctx(&security, &wasi_config, None);
        assert!(ctx.is_ok());
    }

    #[test]
    fn ctx_with_stdout_and_env() {
        let security = SecurityConfig::default();
        let wasi_config = WasiConfig {
            inherit_stdout: true,
            inherit_stderr: true,
            env_vars: vec![("LANG".to_string(), "en_US".to_string())],
        };
        let ctx = build_wasi_ctx(&security, &wasi_config, None);
        assert!(ctx.is_ok());
    }
}
