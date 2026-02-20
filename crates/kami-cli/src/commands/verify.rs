//! `kami verify` command.
//!
//! Verifies the SHA-256 integrity and optional Ed25519 signature
//! of an installed tool's WASM file.

use std::path::Path;

use clap::Args;

use kami_runtime::{compute_file_hash, verify_file_signature};

use crate::{output, shared};

/// Verify the integrity of an installed tool's WASM file.
#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// Tool ID to verify (e.g. dev.example.fetch-url).
    pub tool_id: String,
    /// Database path (defaults to .kami/registry.db).
    #[arg(long)]
    pub db: Option<String>,
    /// Public key (hex or file path) to verify signature against.
    #[arg(long)]
    pub public_key: Option<String>,
}

/// Executes the verify command.
///
/// # Errors
///
/// Returns an error if the tool is not found, the WASM file is missing,
/// the hash does not match, or the signature is invalid.
pub async fn execute(args: &VerifyArgs) -> anyhow::Result<()> {
    let tool_id: kami_types::ToolId = args
        .tool_id
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid tool id: {e}"))?;

    let repo = shared::open_repository(&args.db)?;

    let tool = repo
        .find_by_id(&tool_id)
        .await
        .map_err(|e| anyhow::anyhow!("registry error: {e}"))?
        .ok_or_else(|| anyhow::anyhow!("tool '{}' not found", args.tool_id))?;

    let wasm_path = Path::new(&tool.install_path).join(&tool.manifest.wasm);

    if !wasm_path.exists() {
        anyhow::bail!("WASM file missing: {}", wasm_path.display());
    }

    verify_sha256(args, &wasm_path, &tool)?;
    verify_signature(args, &wasm_path, &tool)?;

    Ok(())
}

fn verify_sha256(
    args: &VerifyArgs,
    wasm_path: &Path,
    tool: &kami_types::Tool,
) -> anyhow::Result<()> {
    let actual_hash = compute_file_hash(wasm_path)
        .map_err(|e| anyhow::anyhow!("failed to hash WASM file: {e}"))?;

    match &tool.manifest.wasm_sha256 {
        None => output::print_warning(&format!(
            "{}: no stored hash (installed before integrity support)",
            args.tool_id
        )),
        Some(expected) if expected == &actual_hash => output::print_success(&format!(
            "{}: SHA-256 OK ({})",
            args.tool_id,
            &actual_hash[..16]
        )),
        Some(expected) => anyhow::bail!(
            "{}: INTEGRITY VIOLATION\n  expected: {}\n  actual:   {}",
            args.tool_id,
            expected,
            actual_hash
        ),
    }
    Ok(())
}

fn verify_signature(
    args: &VerifyArgs,
    wasm_path: &Path,
    tool: &kami_types::Tool,
) -> anyhow::Result<()> {
    let sig = match &tool.manifest.signature {
        Some(s) => s,
        None => {
            output::print_info(&format!("{}: no signature stored", args.tool_id));
            return Ok(());
        }
    };

    let pk = resolve_verify_key(args, tool)?;

    verify_file_signature(wasm_path, sig, &pk)
        .map_err(|e| anyhow::anyhow!("{}: SIGNATURE INVALID: {e}", args.tool_id))?;

    output::print_success(&format!(
        "{}: signature OK (signer={})",
        args.tool_id,
        &pk[..16]
    ));
    Ok(())
}

fn resolve_verify_key(args: &VerifyArgs, tool: &kami_types::Tool) -> anyhow::Result<String> {
    if let Some(ref key) = args.public_key {
        return crate::commands::keygen::resolve_public_key(key);
    }
    tool.manifest
        .signer_public_key
        .clone()
        .ok_or_else(|| anyhow::anyhow!("no public key stored or provided (use --public-key)"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn execute_invalid_id_fails() {
        let dir = tempfile::tempdir().expect("tmp");
        let args = VerifyArgs {
            tool_id: "bad".into(),
            db: Some(dir.path().join("v.db").to_str().expect("p").into()),
            public_key: None,
        };
        assert!(execute(&args).await.is_err());
    }

    #[tokio::test]
    async fn execute_not_found_fails() {
        let dir = tempfile::tempdir().expect("tmp");
        let args = VerifyArgs {
            tool_id: "dev.test.noexist".into(),
            db: Some(dir.path().join("v2.db").to_str().expect("p").into()),
            public_key: None,
        };
        assert!(execute(&args).await.is_err());
    }
}
