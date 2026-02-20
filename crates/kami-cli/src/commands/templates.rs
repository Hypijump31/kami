//! Source code templates for `kami init` scaffolding.
//! Uses `__PLACEHOLDER__` substitution to avoid brace escaping in generated Rust.

/// Cargo.toml template for a new KAMI tool project.
///
/// Placeholders: `__TOOL_NAME__`
pub const CARGO_TOML: &str = r#"[package]
name = "__TOOL_NAME__"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
kami-guest = { path = "../crates/kami-guest" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#;

/// tool.toml template for a new KAMI tool project.
///
/// Placeholders: `__TOOL_ID__`, `__TOOL_NAME__`, `__CRATE_NAME__`
pub const TOOL_TOML: &str = r#"[tool]
id = "__TOOL_ID__"
name = "__TOOL_NAME__"
version = "1.0.0"
wasm = "__CRATE_NAME__.wasm"

[mcp]
description = "TODO: Describe what this tool does"

[[mcp.arguments]]
name = "input"
type = "string"
description = "TODO: Describe this argument"
required = true

[security]
net_allow_list = []
fs_access = "none"
max_memory_mb = 32
max_execution_ms = 5000
max_fuel = 1000000
"#;

/// src/lib.rs template for a new KAMI tool project.
///
/// Placeholders: `__TOOL_ID__`
/// Uses `r##"..."##` delimiter so the template can contain `"#` sequences.
pub const LIB_RS: &str = r##"use kami_guest::kami_tool;

kami_tool! {
    name: "__TOOL_ID__",
    version: "1.0.0",
    description: "TODO: Describe what this tool does",
    handler: handle,
}

fn handle(input: &str) -> Result<String, String> {
    let args: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| format!("invalid JSON: {e}"))?;

    let response = serde_json::json!({
        "result": args,
        "tool": "__TOOL_ID__"
    });

    Ok(response.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_input() {
        let result = handle(r#"{"input":"hello"}"#);
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_input() {
        let result = handle("not json");
        assert!(result.is_err());
    }

    #[test]
    fn empty_input() {
        let result = handle("{}");
        assert!(result.is_ok());
    }
}
"##;

/// .gitignore template.
pub const GITIGNORE: &str = "target/\n*.wasm\n";

/// Applies placeholder substitutions to a template string.
///
/// `substitutions` is a slice of `(placeholder, value)` pairs.
pub fn apply(template: &str, substitutions: &[(&str, &str)]) -> String {
    substitutions
        .iter()
        .fold(template.to_string(), |acc, (key, val)| {
            acc.replace(key, val)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_single_substitution() {
        let result = apply("Hello __NAME__!", &[("__NAME__", "KAMI")]);
        assert_eq!(result, "Hello KAMI!");
    }

    #[test]
    fn apply_multiple_substitutions() {
        let result = apply("__A__ and __B__", &[("__A__", "foo"), ("__B__", "bar")]);
        assert_eq!(result, "foo and bar");
    }

    #[test]
    fn apply_repeated_placeholder() {
        let result = apply("__X__ + __X__", &[("__X__", "hello")]);
        assert_eq!(result, "hello + hello");
    }

    #[test]
    fn cargo_toml_substitution() {
        let result = apply(CARGO_TOML, &[("__TOOL_NAME__", "my-tool")]);
        assert!(result.contains("name = \"my-tool\""));
    }

    #[test]
    fn lib_rs_substitution() {
        let result = apply(LIB_RS, &[("__TOOL_ID__", "dev.test.tool")]);
        assert!(result.contains("name: \"dev.test.tool\""));
        assert!(result.contains("\"tool\": \"dev.test.tool\""));
    }
}
