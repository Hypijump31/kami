//! Additional integration tests for filesystem jail.

use kami_sandbox::FsJail;
use std::path::Path;

#[test]
fn jail_root_returns_configured_root() {
    let jail = FsJail::new("/sandbox/tool1");
    assert_eq!(jail.root(), Path::new("/sandbox/tool1"));
}

#[test]
fn accept_simple_filename() {
    let jail = FsJail::new("/sandbox/tool1");
    let result = jail.validate_path(Path::new("output.txt"));
    assert!(result.is_ok());
}

#[test]
fn accept_deeply_nested_path() {
    let jail = FsJail::new("/sandbox/tool1");
    let result = jail.validate_path(Path::new("a/b/c/d/e/file.txt"));
    assert!(result.is_ok());
}

#[test]
fn reject_double_dot_hidden_in_middle() {
    let jail = FsJail::new("/sandbox/tool1");
    let result = jail.validate_path(Path::new("a/b/../../../etc/passwd"));
    assert!(result.is_err());
}

#[test]
fn reject_windows_rooted_path() {
    let jail = FsJail::new("/sandbox/tool1");
    // On Windows, paths starting with / have a root
    let result = jail.validate_path(Path::new("/absolute/path"));
    assert!(result.is_err());
}

#[test]
fn accept_path_with_dots_in_filename() {
    let jail = FsJail::new("/sandbox/tool1");
    // ".hidden" and "file.tar.gz" are valid — not parent traversal
    let result = jail.validate_path(Path::new("data/.hidden"));
    assert!(result.is_ok());
}

#[test]
fn accept_current_dir_component() {
    let jail = FsJail::new("/sandbox/tool1");
    // "./data" is valid — current dir is fine, only ".." is rejected
    let result = jail.validate_path(Path::new("./data/file.txt"));
    assert!(result.is_ok());
}

#[test]
fn resolved_path_within_jail_root() {
    let jail = FsJail::new("/sandbox/tool1");
    let full = jail
        .validate_path(Path::new("out/result.json"))
        .expect("valid");
    assert!(full.starts_with("/sandbox/tool1"));
}
