//! Tool identity and manifest types.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::capability::SecurityConfig;
use crate::error::KamiError;

/// Unique identifier for a tool (reverse-domain notation).
/// Example: `dev.example.fetch-url`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolId(String);

impl ToolId {
    /// Creates a new `ToolId`, validating the format.
    pub fn new(id: impl Into<String>) -> Result<Self, KamiError> {
        let id = id.into();
        if id.is_empty() {
            return Err(KamiError::invalid_input("tool id cannot be empty"));
        }
        // Must contain at least one dot (reverse-domain)
        if !id.contains('.') {
            return Err(KamiError::invalid_input(
                "tool id must use reverse-domain notation (e.g. dev.example.tool)",
            ));
        }
        Ok(Self(id))
    }

    /// Returns the tool id as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ToolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for ToolId {
    type Err = KamiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// Semantic version for a tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ToolVersion {
    /// Creates a new version.
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for ToolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for ToolVersion {
    type Err = KamiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(KamiError::invalid_input(
                "version must be in semver format: MAJOR.MINOR.PATCH",
            ));
        }
        let parse = |p: &str| -> Result<u32, KamiError> {
            p.parse::<u32>().map_err(|_| {
                KamiError::invalid_input(format!("invalid version component: {p}"))
            })
        };
        Ok(Self {
            major: parse(parts[0])?,
            minor: parse(parts[1])?,
            patch: parse(parts[2])?,
        })
    }
}

/// MCP argument definition for a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolArgument {
    /// Argument name.
    pub name: String,
    /// JSON Schema type (string, number, boolean, etc.).
    #[serde(rename = "type")]
    pub arg_type: String,
    /// Human-readable description.
    pub description: String,
    /// Whether this argument is required.
    #[serde(default)]
    pub required: bool,
    /// Default value if not required.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// Complete tool manifest (parsed from tool.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    /// Tool identity.
    pub id: ToolId,
    /// Human-readable name.
    pub name: String,
    /// Tool version.
    pub version: ToolVersion,
    /// Path to the WASM component file.
    pub wasm: String,
    /// MCP description.
    pub description: String,
    /// Tool arguments.
    #[serde(default)]
    pub arguments: Vec<ToolArgument>,
    /// Security configuration.
    pub security: SecurityConfig,
}

/// Installed tool with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// The tool manifest.
    pub manifest: ToolManifest,
    /// Installation path on disk.
    pub install_path: String,
    /// Whether the tool is enabled.
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_tool_id() {
        let id = ToolId::new("dev.example.fetch-url");
        assert!(id.is_ok());
        assert_eq!(id.unwrap().as_str(), "dev.example.fetch-url");
    }

    #[test]
    fn empty_tool_id_rejected() {
        assert!(ToolId::new("").is_err());
    }

    #[test]
    fn tool_id_without_dot_rejected() {
        assert!(ToolId::new("no-dot").is_err());
    }

    #[test]
    fn version_parse() {
        let v: ToolVersion = "1.2.3".parse().unwrap();
        assert_eq!(v, ToolVersion::new(1, 2, 3));
    }

    #[test]
    fn version_display() {
        let v = ToolVersion::new(0, 1, 0);
        assert_eq!(v.to_string(), "0.1.0");
    }

    #[test]
    fn invalid_version_rejected() {
        assert!("1.2".parse::<ToolVersion>().is_err());
        assert!("a.b.c".parse::<ToolVersion>().is_err());
    }
}
