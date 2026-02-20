//! Tool manifest loader — reads and parses `tool.toml` files.
//!
//! Lives in `kami-config` (Adapter layer) because it performs filesystem I/O.
//! Pure string-to-manifest conversion is also here to keep `toml` out of the
//! domain layer (`kami-types`).

use std::path::Path;

use kami_types::{SecurityConfig, ToolArgument, ToolId, ToolManifest, ToolVersion};

/// Error type for manifest parsing failures.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    /// The file could not be read.
    #[error("cannot read manifest at '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    /// The TOML content is invalid or missing required fields.
    #[error("invalid tool.toml: {0}")]
    Parse(String),
}

/// Parses a `tool.toml` string into a `ToolManifest`.
///
/// This is a pure transformation — no filesystem access.
///
/// # Errors
///
/// Returns `ManifestError::Parse` if the TOML is malformed or missing
/// required sections.
pub fn parse_tool_manifest(content: &str) -> Result<ToolManifest, ManifestError> {
    let raw: RawToolToml =
        toml::from_str(content).map_err(|e| ManifestError::Parse(e.to_string()))?;

    let id = ToolId::new(&raw.tool.id).map_err(|e| ManifestError::Parse(e.to_string()))?;

    let version: ToolVersion = raw
        .tool
        .version
        .parse()
        .map_err(|e: kami_types::KamiError| ManifestError::Parse(e.to_string()))?;

    Ok(ToolManifest {
        id,
        name: raw.tool.name,
        version,
        wasm: raw.tool.wasm,
        description: raw.mcp.description,
        arguments: raw.mcp.arguments,
        security: raw.security,
        // Populated at install time by hashing the WASM file on disk;
        // not present in the tool.toml source file itself.
        wasm_sha256: None,
        signature: None,
        signer_public_key: None,
    })
}

/// Reads a `tool.toml` file from disk and parses it into a `ToolManifest`.
///
/// # Errors
///
/// Returns `ManifestError::Io` if the file cannot be read.
/// Returns `ManifestError::Parse` if the content is invalid.
pub fn parse_tool_manifest_file(path: &Path) -> Result<ToolManifest, ManifestError> {
    let content = std::fs::read_to_string(path).map_err(|e| ManifestError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    parse_tool_manifest(&content)
}

// ---------------------------------------------------------------------------
// Private TOML deserialization types
// ---------------------------------------------------------------------------

/// Raw TOML structure mirroring the `tool.toml` file format.
///
/// `SecurityConfig` already derives `Deserialize` with `#[serde(flatten)]`
/// on `ResourceLimits`, so it maps directly to the `[security]` section.
#[derive(Debug, serde::Deserialize)]
struct RawToolToml {
    tool: RawToolSection,
    mcp: RawMcpSection,
    #[serde(default)]
    security: SecurityConfig,
}

/// `[tool]` section of tool.toml.
#[derive(Debug, serde::Deserialize)]
struct RawToolSection {
    id: String,
    name: String,
    version: String,
    wasm: String,
}

/// `[mcp]` section of tool.toml.
#[derive(Debug, serde::Deserialize)]
struct RawMcpSection {
    description: String,
    #[serde(default)]
    arguments: Vec<ToolArgument>,
}
