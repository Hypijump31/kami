//! Parser for `tool.toml` manifest files.
//!
//! Maps the TOML structure (`[tool]`, `[mcp]`, `[security]`)
//! into the domain `ToolManifest` type.

use std::path::Path;

use crate::capability::SecurityConfig;
use crate::error::KamiError;
use crate::tool::{ToolArgument, ToolId, ToolManifest, ToolVersion};

/// Raw TOML structure mirroring the `tool.toml` file format.
#[derive(Debug, serde::Deserialize)]
struct ToolToml {
    tool: ToolSection,
    mcp: McpSection,
    #[serde(default)]
    security: SecurityConfig,
}

/// `[tool]` section of tool.toml.
#[derive(Debug, serde::Deserialize)]
struct ToolSection {
    id: String,
    name: String,
    version: String,
    wasm: String,
}

/// `[mcp]` section of tool.toml.
#[derive(Debug, serde::Deserialize)]
struct McpSection {
    description: String,
    #[serde(default)]
    arguments: Vec<ToolArgument>,
}

/// Parses a `tool.toml` string into a `ToolManifest`.
pub fn parse_tool_manifest(content: &str) -> Result<ToolManifest, KamiError> {
    let toml: ToolToml = toml::from_str(content).map_err(|e| {
        KamiError::invalid_input(format!("invalid tool.toml: {e}"))
    })?;

    let id = ToolId::new(&toml.tool.id)?;
    let version: ToolVersion = toml.tool.version.parse()?;

    Ok(ToolManifest {
        id,
        name: toml.tool.name,
        version,
        wasm: toml.tool.wasm,
        description: toml.mcp.description,
        arguments: toml.mcp.arguments,
        security: toml.security,
    })
}

/// Parses a `tool.toml` file from disk.
pub fn parse_tool_manifest_file(
    path: &Path,
) -> Result<ToolManifest, KamiError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        KamiError::invalid_input(format!(
            "cannot read {}: {e}",
            path.display()
        ))
    })?;
    parse_tool_manifest(&content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FsAccess;

    const SAMPLE_TOML: &str = r#"
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
        let manifest = parse_tool_manifest(SAMPLE_TOML)
            .expect("should parse");

        assert_eq!(manifest.id.as_str(), "dev.example.fetch-url");
        assert_eq!(manifest.name, "fetch-url");
        assert_eq!(manifest.version.to_string(), "1.0.0");
        assert_eq!(manifest.wasm, "fetch_url.wasm");
        assert_eq!(manifest.description, "Fetches content from a URL");
        assert_eq!(manifest.arguments.len(), 1);
        assert_eq!(manifest.arguments[0].name, "url");
        assert!(manifest.arguments[0].required);
        assert_eq!(manifest.security.net_allow_list.len(), 2);
        assert_eq!(manifest.security.fs_access, FsAccess::None);
        assert_eq!(manifest.security.limits.max_memory_mb, 64);
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
        let manifest = parse_tool_manifest(toml).expect("should parse");
        assert_eq!(manifest.id.as_str(), "dev.test.minimal");
        assert!(manifest.arguments.is_empty());
        // Default security: deny-all
        assert!(manifest.security.net_allow_list.is_empty());
        assert_eq!(manifest.security.fs_access, FsAccess::None);
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
description = "Bad tool"
"#;
        assert!(parse_tool_manifest(toml).is_err());
    }

    #[test]
    fn parse_invalid_version_rejected() {
        let toml = r#"
[tool]
id = "dev.test.bad"
name = "bad"
version = "not-semver"
wasm = "bad.wasm"

[mcp]
description = "Bad tool"
"#;
        assert!(parse_tool_manifest(toml).is_err());
    }

    #[test]
    fn parse_missing_section_rejected() {
        let toml = r#"
[tool]
id = "dev.test.no-mcp"
name = "no-mcp"
version = "1.0.0"
wasm = "no-mcp.wasm"
"#;
        assert!(parse_tool_manifest(toml).is_err());
    }
}
