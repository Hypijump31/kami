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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompts_list_params_default() {
        let p = PromptsListParams::default();
        assert!(p.cursor.is_none());
    }

    #[test]
    fn prompt_definition_serde_roundtrip() {
        let def = McpPromptDefinition {
            name: "summarize".into(),
            description: Some("Summarize text".into()),
        };
        let s = serde_json::to_string(&def).expect("ser");
        let back: McpPromptDefinition = serde_json::from_str(&s).expect("de");
        assert_eq!(back.name, "summarize");
        assert!(back.description.is_some());
    }

    #[test]
    fn prompts_list_result_roundtrip() {
        let r = PromptsListResult {
            prompts: vec![McpPromptDefinition {
                name: "p".into(),
                description: None,
            }],
        };
        let s = serde_json::to_string(&r).expect("ser");
        let back: PromptsListResult = serde_json::from_str(&s).expect("de");
        assert_eq!(back.prompts.len(), 1);
    }
}
