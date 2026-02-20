//! `kami sign` command.
//!
//! Signs a WASM plugin file with an Ed25519 private key.
//! The signature is stored in the tool.toml manifest and printed.

use std::path::Path;

use clap::Args;
use kami_config::parse_tool_manifest_file;
use kami_runtime::{compute_file_hash, sign_file};

use crate::commands::keygen;
use crate::output;

/// Sign a WASM plugin with your Ed25519 key.
#[derive(Debug, Args)]
pub struct SignArgs {
    /// Path to the tool directory containing tool.toml.
    #[arg(default_value = ".")]
    pub path: String,
    /// Custom keys directory (defaults to ~/.kami/keys/).
    #[arg(long)]
    pub keys: Option<String>,
}

/// Executes the sign command.
///
/// # Errors
///
/// Returns an error if the tool.toml or WASM file is missing,
/// or the signing key cannot be read.
pub fn execute(args: &SignArgs) -> anyhow::Result<()> {
    let tool_path = Path::new(&args.path);
    let manifest_path = if tool_path.is_dir() {
        tool_path.join("tool.toml")
    } else {
        tool_path.to_path_buf()
    };
    if !manifest_path.exists() {
        anyhow::bail!("tool.toml not found: {}", manifest_path.display());
    }

    let manifest = parse_tool_manifest_file(&manifest_path)
        .map_err(|e| anyhow::anyhow!("manifest error: {e}"))?;

    let tool_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let wasm_path = tool_dir.join(&manifest.wasm);
    if !wasm_path.exists() {
        anyhow::bail!("WASM file not found: {}", wasm_path.display());
    }

    let secret_key = keygen::read_secret_key(&args.keys)?;
    let public_key = kami_runtime::public_key_from_secret(&secret_key)
        .map_err(|e| anyhow::anyhow!("key error: {e}"))?;

    let signature =
        sign_file(&wasm_path, &secret_key).map_err(|e| anyhow::anyhow!("signing failed: {e}"))?;

    let wasm_hash =
        compute_file_hash(&wasm_path).map_err(|e| anyhow::anyhow!("hash failed: {e}"))?;

    output::print_success(&format!("Signed: {}", manifest.id));
    println!("  wasm_sha256:      {wasm_hash}");
    println!("  signature:        {signature}");
    println!("  signer_public_key: {public_key}");
    println!();
    println!("Add to your tool.toml [tool] section:");
    println!("  # wasm_sha256 = \"{wasm_hash}\"");
    println!();
    println!("Include in your registry entry:");
    println!("  \"signature\": \"{signature}\"");
    println!("  \"signer_public_key\": \"{public_key}\"");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_missing_tool_toml() {
        let dir = tempfile::tempdir().expect("tmp");
        let args = SignArgs {
            path: dir.path().to_str().expect("u").to_string(),
            keys: None,
        };
        assert!(execute(&args).is_err());
    }

    #[test]
    fn sign_with_missing_key() {
        // Create a minimal tool.toml but no key
        let dir = tempfile::tempdir().expect("tmp");
        let toml = r#"
[tool]
id = "dev.test.sign"
name = "sign-test"
version = "1.0.0"
wasm = "test.wasm"

[mcp]
description = "test"

[security]
"#;
        std::fs::write(dir.path().join("tool.toml"), toml).unwrap();
        std::fs::write(dir.path().join("test.wasm"), b"fake").unwrap();
        let args = SignArgs {
            path: dir.path().to_str().expect("u").to_string(),
            keys: Some(dir.path().join("nokeys").to_str().unwrap().to_string()),
        };
        assert!(execute(&args).is_err());
    }
}
