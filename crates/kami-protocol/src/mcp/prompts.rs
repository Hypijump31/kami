//! MCP prompts/* method types.

use serde::{Deserialize, Serialize};

/// Request params for `prompts/list`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptsListParams {
    /// Optional cursor for pagination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// A prompt definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptDefinition {
    /// Prompt name.
    pub name: String,
    /// Prompt description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Response for `prompts/list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsListResult {
    /// Available prompts.
    pub prompts: Vec<McpPromptDefinition>,
}
