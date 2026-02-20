//! WASM component integrity verification.
//!
//! Computes and verifies SHA-256 hashes of WASM files to detect
//! tampering between install time and execution time.

use std::io;
use std::path::Path;

use hex::ToHex;
use sha2::{Digest, Sha256};

/// Computes the SHA-256 hash of a file and returns it as a hex string.
///
/// # Errors
///
/// Returns `io::Error` if the file cannot be read.
pub fn compute_file_hash(path: &Path) -> Result<String, io::Error> {
    let bytes = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hasher.finalize().encode_hex::<String>())
}

/// Verifies that a file matches an expected SHA-256 hex digest.
///
/// Returns `Ok(())` if the hash matches or if `expected` is `None`
/// (no stored hash = verification skipped for backwards compatibility).
///
/// # Errors
///
/// Returns `Err(actual_hash)` if the computed hash differs from `expected`.
pub fn verify_hash(path: &Path, expected: &Option<String>) -> Result<(), io::Error> {
    let Some(expected_hash) = expected else {
        // No stored hash — skip verification (pre-integrity install).
        return Ok(());
    };

    let actual = compute_file_hash(path)?;
    if actual != *expected_hash {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("integrity violation: expected {expected_hash}, got {actual}"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn hash_deterministic() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"hello").unwrap();
        let h1 = compute_file_hash(f.path()).unwrap();
        let h2 = compute_file_hash(f.path()).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_known_value() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"").unwrap();
        let h = compute_file_hash(f.path()).unwrap();
        // SHA-256 of empty input
        assert_eq!(
            h,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn verify_passes_when_no_expected() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"data").unwrap();
        assert!(verify_hash(f.path(), &None).is_ok());
    }

    #[test]
    fn verify_passes_with_correct_hash() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"data").unwrap();
        let hash = compute_file_hash(f.path()).unwrap();
        assert!(verify_hash(f.path(), &Some(hash)).is_ok());
    }

    #[test]
    fn verify_fails_with_wrong_hash() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"data").unwrap();
        let wrong = "0".repeat(64);
        assert!(verify_hash(f.path(), &Some(wrong)).is_err());
    }
}
