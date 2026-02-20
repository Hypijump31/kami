//! Property-based tests for `parse_tool_manifest`.
//!
//! Ensures the parser never panics on arbitrary or malformed input and
//! that well-formed input always round-trips successfully.

use kami_config::manifest_loader::parse_tool_manifest;
use proptest::prelude::*;

proptest! {
    /// Arbitrary byte strings never cause a panic.
    #[test]
    fn no_panic_on_arbitrary_input(input in "\\PC{0,256}") {
        let _ = parse_tool_manifest(&input);
    }

    /// Valid TOML with all required fields always parses successfully.
    #[test]
    fn valid_manifest_always_parses(
        name in "[a-z][a-z0-9-]{0,15}",
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100,
    ) {
        let toml = format!(
            r#"
[tool]
id = "dev.test.{name}"
name = "{name}"
version = "{major}.{minor}.{patch}"
wasm = "{name}.wasm"

[mcp]
description = "A test tool"

[security]
fs_access = "none"
"#
        );
        let result = parse_tool_manifest(&toml);
        prop_assert!(
            result.is_ok(),
            "valid manifest rejected: {result:?}",
        );
    }

    /// Missing [mcp] section always fails gracefully.
    #[test]
    fn missing_mcp_section_fails(name in "[a-z]{2,8}") {
        let toml = format!(
            r#"
[tool]
id = "dev.test.{name}"
name = "{name}"
version = "1.0.0"
wasm = "{name}.wasm"
"#
        );
        let result = parse_tool_manifest(&toml);
        prop_assert!(result.is_err());
    }

    /// Missing [tool] section always fails gracefully.
    #[test]
    fn missing_tool_section_fails(desc in "[a-z ]{2,16}") {
        let toml = format!(
            r#"
[mcp]
description = "{desc}"
"#
        );
        let result = parse_tool_manifest(&toml);
        prop_assert!(result.is_err());
    }
}
