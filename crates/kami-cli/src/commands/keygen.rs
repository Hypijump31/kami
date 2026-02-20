//! `kami keygen` command.
//!
//! Generates an Ed25519 keypair for signing WASM plugins.
//! Keys are stored in `~/.kami/keys/` as hex-encoded files.

use std::path::PathBuf;

use clap::Args;
use kami_runtime::generate_keypair;

use crate::{output, shared};

const KEY_FILE: &str = "kami_signing_key";
const PUB_FILE: &str = "kami_signing_key.pub";

/// Generate an Ed25519 signing keypair.
#[derive(Debug, Args)]
pub struct KeygenArgs {
    /// Output directory for keypair (defaults to ~/.kami/keys/).
    #[arg(long)]
    pub output: Option<String>,
    /// Overwrite existing keys without prompting.
    #[arg(long)]
    pub force: bool,
}

/// Returns the keys directory path.
pub fn keys_dir(custom: &Option<String>) -> PathBuf {
    match custom {
        Some(dir) => PathBuf::from(dir),
        None => shared::data_dir().join("keys"),
    }
}

/// Executes the keygen command.
///
/// # Errors
///
/// Returns an error if the keys directory cannot be created
/// or the key files cannot be written.
pub fn execute(args: &KeygenArgs) -> anyhow::Result<()> {
    let dir = keys_dir(&args.output);
    std::fs::create_dir_all(&dir)?;

    let secret_path = dir.join(KEY_FILE);
    let public_path = dir.join(PUB_FILE);

    if secret_path.exists() && !args.force {
        anyhow::bail!(
            "key already exists: {}. Use --force to overwrite.",
            secret_path.display()
        );
    }

    let kp = generate_keypair();

    std::fs::write(&secret_path, &kp.secret_key)?;
    std::fs::write(&public_path, &kp.public_key)?;

    // Set restrictive permissions on the secret key (Unix only).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&secret_path, std::fs::Permissions::from_mode(0o600))?;
    }

    output::print_success("Ed25519 keypair generated!");
    output::print_info(&format!("  Secret key: {}", secret_path.display()));
    output::print_info(&format!("  Public key: {}", public_path.display()));
    output::print_warning("Keep your secret key private. Never share it.");
    Ok(())
}

/// Reads the secret key from the default or custom keys directory.
///
/// # Errors
///
/// Returns an error if the key file does not exist or cannot be read.
pub fn read_secret_key(custom_dir: &Option<String>) -> anyhow::Result<String> {
    let dir = keys_dir(custom_dir);
    let path = dir.join(KEY_FILE);
    if !path.exists() {
        anyhow::bail!(
            "no signing key found at {}. Run `kami keygen` first.",
            path.display()
        );
    }
    let key = std::fs::read_to_string(&path)?.trim().to_string();
    if key.len() != 64 {
        anyhow::bail!("invalid secret key length at {}", path.display());
    }
    Ok(key)
}

/// Reads the public key from a file or hex string.
///
/// # Errors
///
/// Returns an error if the file does not exist or the format is invalid.
pub fn resolve_public_key(key_or_path: &str) -> anyhow::Result<String> {
    if key_or_path.len() == 64 && hex::decode(key_or_path).is_ok() {
        return Ok(key_or_path.to_string());
    }
    let path = std::path::Path::new(key_or_path);
    if path.exists() {
        let content = std::fs::read_to_string(path)?.trim().to_string();
        if content.len() != 64 {
            anyhow::bail!("invalid public key length in {}", path.display());
        }
        return Ok(content);
    }
    anyhow::bail!("not a valid public key hex or file path: {key_or_path}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args_at(dir: &tempfile::TempDir, force: bool) -> KeygenArgs {
        KeygenArgs {
            output: Some(dir.path().to_str().unwrap().to_string()),
            force,
        }
    }

    #[test]
    fn keygen_creates_files() {
        let dir = tempfile::tempdir().expect("tmp");
        execute(&args_at(&dir, false)).expect("keygen");
        assert!(dir.path().join(KEY_FILE).exists());
        assert!(dir.path().join(PUB_FILE).exists());
        let sk = std::fs::read_to_string(dir.path().join(KEY_FILE)).unwrap();
        assert_eq!(sk.len(), 64);
    }

    #[test]
    fn keygen_refuses_overwrite_without_force() {
        let dir = tempfile::tempdir().expect("tmp");
        execute(&args_at(&dir, false)).expect("first");
        assert!(execute(&args_at(&dir, false)).is_err());
    }

    #[test]
    fn keygen_force_overwrites() {
        let dir = tempfile::tempdir().expect("tmp");
        execute(&args_at(&dir, false)).expect("first");
        assert!(execute(&args_at(&dir, true)).is_ok());
    }
}
