//! MCP tools/* method types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request params for `tools/list`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolsListParams {
    /// Optional cursor for pagination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// A single tool definition in the MCP response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    /// Tool name.
    pub name: String,
    /// Tool description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema for input parameters.
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Response for `tools/list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListResult {
    /// Available tools.
    pub tools: Vec<McpToolDefinition>,
    /// Pagination cursor for next page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Request params for `tools/call`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCallParams {
    /// Name of the tool to invoke.
    pub name: String,
    /// Arguments to pass.
    #[serde(default)]
    pub arguments: Value,
}

/// Content item in a tool call response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ToolContent {
    /// Text content.
    Text { text: String },
    /// Image content (base64).
    Image { data: String, mime_type: String },
}

/// Response for `tools/call`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCallResult {
    /// Content items returned by the tool.
    pub content: Vec<ToolContent>,
    /// Whether the tool call resulted in an error.
    #[serde(default, rename = "isError")]
    pub is_error: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tools_list_params_default() {
        let p = ToolsListParams::default();
        assert!(p.cursor.is_none());
    }

    #[test]
    fn tool_definition_serde_roundtrip() {
        let def = McpToolDefinition {
            name: "my-tool".into(),
            description: Some("A tool".into()),
            input_schema: json!({"type": "object"}),
        };
        let s = serde_json::to_string(&def).expect("ser");
        assert!(s.contains("inputSchema"));
        let back: McpToolDefinition = serde_json::from_str(&s).expect("de");
        assert_eq!(back.name, "my-tool");
    }

    #[test]
    fn tools_list_result_with_tools() {
        let res = ToolsListResult {
            tools: vec![McpToolDefinition {
                name: "t".into(),
                description: None,
                input_schema: json!({}),
            }],
            next_cursor: None,
        };
        let s = serde_json::to_string(&res).expect("ser");
        let back: ToolsListResult = serde_json::from_str(&s).expect("de");
        assert_eq!(back.tools.len(), 1);
    }

    #[test]
    fn tools_call_params_serde() {
        let j = r#"{"name":"echo","arguments":{"x":1}}"#;
        let p: ToolsCallParams = serde_json::from_str(j).expect("de");
        assert_eq!(p.name, "echo");
        assert_eq!(p.arguments["x"], 1);
    }

    #[test]
    fn tool_content_text_variant() {
        let c = ToolContent::Text {
            text: "hello".into(),
        };
        let s = serde_json::to_string(&c).expect("ser");
        assert!(s.contains("\"type\":\"text\""));
    }

    #[test]
    fn tools_call_result_roundtrip() {
        let r = ToolsCallResult {
            content: vec![ToolContent::Text { text: "ok".into() }],
            is_error: false,
        };
        let s = serde_json::to_string(&r).expect("ser");
        let back: ToolsCallResult = serde_json::from_str(&s).expect("de");
        assert!(!back.is_error);
        assert_eq!(back.content.len(), 1);
    }
}
