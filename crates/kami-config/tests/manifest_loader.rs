//! Integration tests for manifest_loader.

use kami_config::manifest_loader::{parse_tool_manifest, parse_tool_manifest_file};
use std::path::Path;

const SAMPLE: &str = r#"
[tool]
id = "dev.example.fetch-url"
name = "fetch-url"
version = "1.0.0"
wasm = "fetch_url.wasm"

[mcp]
description = "Fetches content from a URL"

[[mcp.arguments]]
name = "url"
type = "string"
description = "The URL to fetch"
required = true

[security]
net_allow_list = ["*.example.com", "api.github.com"]
fs_access = "none"
max_memory_mb = 64
max_execution_ms = 5000
"#;

#[test]
fn parse_valid_tool_toml() {
    let m = parse_tool_manifest(SAMPLE).expect("should parse");
    assert_eq!(m.id.as_str(), "dev.example.fetch-url");
    assert_eq!(m.version.to_string(), "1.0.0");
    assert_eq!(m.arguments.len(), 1);
    assert_eq!(m.security.net_allow_list.len(), 2);
    assert_eq!(m.security.limits.max_memory_mb, 64);
}

#[test]
fn parse_minimal_toml() {
    let toml = r#"
[tool]
id = "dev.test.minimal"
name = "minimal"
version = "0.1.0"
wasm = "minimal.wasm"

[mcp]
description = "A minimal tool"
"#;
    let m = parse_tool_manifest(toml).expect("should parse");
    assert!(m.arguments.is_empty());
    assert!(m.security.net_allow_list.is_empty());
}

#[test]
fn parse_invalid_id_rejected() {
    let toml = r#"
[tool]
id = "no-dot"
name = "bad"
version = "1.0.0"
wasm = "bad.wasm"
[mcp]
description = "Bad"
"#;
    assert!(parse_tool_manifest(toml).is_err());
}

#[test]
fn parse_missing_mcp_rejected() {
    let toml = r#"
[tool]
id = "dev.test.x"
name = "x"
version = "1.0.0"
wasm = "x.wasm"
"#;
    assert!(parse_tool_manifest(toml).is_err());
}

#[test]
fn parse_file_not_found() {
    let result = parse_tool_manifest_file(Path::new("/nonexistent/tool.toml"));
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("cannot read manifest"));
}
