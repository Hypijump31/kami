//! Property-based tests for `FsJail::validate_path`.
//!
//! Uses `proptest` to fuzz path inputs — verifying that no generated path
//! can escape the sandbox jail root.

use std::path::Path;

use kami_sandbox::FsJail;
use proptest::prelude::*;

/// Strategy producing random path-like strings including traversal attempts.
fn path_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z0-9_./ \\]{0,64}|(\.\./){1,8}[a-z]{1,8}|/[a-z/]{0,32}")
        .expect("valid regex")
}

proptest! {
    /// A valid result from `validate_path` must be under the jail root.
    #[test]
    fn validated_path_stays_within_jail(input in path_strategy()) {
        let jail = FsJail::new("/sandbox/tool1");
        if let Ok(result) = jail.validate_path(Path::new(&input)) {
            prop_assert!(
                result.starts_with("/sandbox/tool1"),
                "escaped jail with input={input:?} → result={result:?}",
            );
        }
    }

    /// Absolute paths are always rejected.
    #[test]
    fn absolute_paths_always_rejected(input in "/[a-z/]{1,32}") {
        let jail = FsJail::new("/sandbox/tool1");
        let result = jail.validate_path(Path::new(&input));
        prop_assert!(result.is_err(), "accepted absolute: {input:?}");
    }

    /// Paths containing ".." are always rejected.
    #[test]
    fn dotdot_paths_always_rejected(
        prefix in "[a-z]{0,8}",
        suffix in "[a-z]{0,8}",
    ) {
        let jail = FsJail::new("/sandbox/tool1");
        let malicious = format!("{prefix}/../{suffix}");
        let result = jail.validate_path(Path::new(&malicious));
        prop_assert!(result.is_err(), "accepted traversal: {malicious:?}");
    }

    /// Simple relative filenames are always accepted.
    #[test]
    fn simple_relative_always_accepted(name in "[a-z]{1,16}\\.[a-z]{1,4}") {
        let jail = FsJail::new("/sandbox/tool1");
        let result = jail.validate_path(Path::new(&name));
        prop_assert!(result.is_ok(), "rejected valid name: {name:?}");
    }
}
