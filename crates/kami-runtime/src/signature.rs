//! Ed25519 cryptographic signing and verification for WASM plugins.
//!
//! Provides key generation, file signing, and signature verification
//! using the Ed25519 algorithm. Keys are hex-encoded raw bytes.

use std::io;
use std::path::Path;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hex::ToHex;
use rand::rngs::OsRng;

/// An Ed25519 keypair represented as hex strings.
#[derive(Debug, Clone)]
pub struct KeyPair {
    /// 64-char hex-encoded secret key (32 bytes).
    pub secret_key: String,
    /// 64-char hex-encoded public key (32 bytes).
    pub public_key: String,
}

/// Generates a new Ed25519 keypair.
pub fn generate_keypair() -> KeyPair {
    let signing = SigningKey::generate(&mut OsRng);
    KeyPair {
        secret_key: signing.to_bytes().encode_hex::<String>(),
        public_key: signing.verifying_key().to_bytes().encode_hex::<String>(),
    }
}

/// Signs a file with the given hex-encoded secret key.
///
/// # Errors
///
/// Returns `io::Error` if the file cannot be read or the key is invalid.
pub fn sign_file(path: &Path, secret_key_hex: &str) -> Result<String, io::Error> {
    let key_bytes = hex::decode(secret_key_hex).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid secret key hex: {e}"),
        )
    })?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "secret key must be 32 bytes"))?;
    let signing = SigningKey::from_bytes(&key_array);
    let data = std::fs::read(path)?;
    let sig = signing.sign(&data);
    Ok(sig.to_bytes().encode_hex::<String>())
}

/// Verifies a file's Ed25519 signature against a hex-encoded public key.
///
/// # Errors
///
/// Returns `io::Error` if the file cannot be read, keys are invalid,
/// or the signature does not match.
pub fn verify_file_signature(
    path: &Path,
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<(), io::Error> {
    let pk_bytes = hex::decode(public_key_hex).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid public key hex: {e}"),
        )
    })?;
    let pk_array: [u8; 32] = pk_bytes
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "public key must be 32 bytes"))?;
    let verifying = VerifyingKey::from_bytes(&pk_array).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid public key: {e}"),
        )
    })?;

    let sig_bytes = hex::decode(signature_hex).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid signature hex: {e}"),
        )
    })?;
    let sig_array: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "signature must be 64 bytes"))?;
    let signature = Signature::from_bytes(&sig_array);

    let data = std::fs::read(path)?;
    verifying.verify(&data, &signature).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("signature invalid: {e}"),
        )
    })
}

/// Returns the public key associated with a hex-encoded secret key.
///
/// # Errors
///
/// Returns `io::Error` if the key hex is invalid.
pub fn public_key_from_secret(secret_key_hex: &str) -> Result<String, io::Error> {
    let key_bytes = hex::decode(secret_key_hex).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid secret key hex: {e}"),
        )
    })?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "secret key must be 32 bytes"))?;
    let signing = SigningKey::from_bytes(&key_array);
    Ok(signing.verifying_key().to_bytes().encode_hex::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn keypair_hex_lengths() {
        let kp = generate_keypair();
        assert_eq!(kp.secret_key.len(), 64);
        assert_eq!(kp.public_key.len(), 64);
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let kp = generate_keypair();
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"hello wasm").unwrap();
        let sig = sign_file(f.path(), &kp.secret_key).unwrap();
        assert_eq!(sig.len(), 128); // 64 bytes = 128 hex
        let result = verify_file_signature(f.path(), &sig, &kp.public_key);
        assert!(result.is_ok());
    }

    #[test]
    fn verify_rejects_wrong_key() {
        let kp1 = generate_keypair();
        let kp2 = generate_keypair();
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"payload").unwrap();
        let sig = sign_file(f.path(), &kp1.secret_key).unwrap();
        let result = verify_file_signature(f.path(), &sig, &kp2.public_key);
        assert!(result.is_err());
    }

    #[test]
    fn verify_rejects_tampered_file() {
        let kp = generate_keypair();
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"original").unwrap();
        let sig = sign_file(f.path(), &kp.secret_key).unwrap();
        f.write_all(b"tamper").unwrap();
        let result = verify_file_signature(f.path(), &sig, &kp.public_key);
        assert!(result.is_err());
    }

    #[test]
    fn public_key_derivation() {
        let kp = generate_keypair();
        let derived = public_key_from_secret(&kp.secret_key).unwrap();
        assert_eq!(derived, kp.public_key);
    }

    #[test]
    fn invalid_hex_key_rejected() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"data").unwrap();
        assert!(sign_file(f.path(), "not-hex").is_err());
        assert!(sign_file(f.path(), "abcd").is_err()); // too short
    }
}
