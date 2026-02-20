//! Tests for Ed25519 cryptographic signing and verification.

use std::io::Write;

use kami_runtime::{generate_keypair, public_key_from_secret, sign_file, verify_file_signature};
use tempfile::NamedTempFile;

fn tmp_file(content: &[u8]) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("tempfile");
    f.write_all(content).expect("write");
    f
}

#[test]
fn keypair_hex_lengths() {
    let kp = generate_keypair();
    assert_eq!(kp.secret_key.len(), 64);
    assert_eq!(kp.public_key.len(), 64);
}

#[test]
fn sign_and_verify_roundtrip() {
    let kp = generate_keypair();
    let f = tmp_file(b"hello wasm");
    let sig = sign_file(f.path(), &kp.secret_key).expect("sign");
    assert_eq!(sig.len(), 128); // 64 bytes = 128 hex chars
    assert!(verify_file_signature(f.path(), &sig, &kp.public_key).is_ok());
}

#[test]
fn verify_rejects_wrong_key() {
    let kp1 = generate_keypair();
    let kp2 = generate_keypair();
    let f = tmp_file(b"payload");
    let sig = sign_file(f.path(), &kp1.secret_key).expect("sign");
    assert!(verify_file_signature(f.path(), &sig, &kp2.public_key).is_err());
}

#[test]
fn verify_rejects_tampered_file() {
    let kp = generate_keypair();
    let mut f = tmp_file(b"original");
    let sig = sign_file(f.path(), &kp.secret_key).expect("sign");
    f.write_all(b"tamper").expect("write");
    assert!(verify_file_signature(f.path(), &sig, &kp.public_key).is_err());
}

#[test]
fn public_key_derivation() {
    let kp = generate_keypair();
    let derived = public_key_from_secret(&kp.secret_key).expect("derive");
    assert_eq!(derived, kp.public_key);
}

#[test]
fn sign_with_invalid_hex_fails() {
    let f = tmp_file(b"data");
    assert!(sign_file(f.path(), "not-hex").is_err());
    assert!(sign_file(f.path(), "abcd").is_err()); // too short (2 bytes ≠ 32)
}

#[test]
fn sign_nonexistent_file_fails() {
    let kp = generate_keypair();
    let result = sign_file(std::path::Path::new("/no/such/file.wasm"), &kp.secret_key);
    assert!(result.is_err());
}

#[test]
fn verify_with_invalid_signature_hex_fails() {
    let kp = generate_keypair();
    let f = tmp_file(b"data");
    assert!(verify_file_signature(f.path(), "not-hex", &kp.public_key).is_err());
    // Right format, wrong length (not 64 bytes)
    assert!(verify_file_signature(f.path(), "deadbeef", &kp.public_key).is_err());
}

#[test]
fn verify_with_invalid_public_key_hex_fails() {
    let kp = generate_keypair();
    let f = tmp_file(b"data");
    let sig = sign_file(f.path(), &kp.secret_key).expect("sign");
    assert!(verify_file_signature(f.path(), &sig, "not-hex").is_err());
    assert!(verify_file_signature(f.path(), &sig, "abcd").is_err()); // too short
}

#[test]
fn public_key_from_invalid_hex_fails() {
    assert!(public_key_from_secret("not-hex").is_err());
    assert!(public_key_from_secret("abcd").is_err()); // too short
}
