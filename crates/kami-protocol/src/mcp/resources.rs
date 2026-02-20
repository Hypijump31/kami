//! MCP resources/* method types.

use serde::{Deserialize, Serialize};

/// Request params for `resources/read`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesReadParams {
    /// URI of the resource to read.
    pub uri: String,
}

/// A resource definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceDefinition {
    /// Resource URI.
    pub uri: String,
    /// Resource name.
    pub name: String,
    /// Resource description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Response for `resources/list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListResult {
    /// Available resources.
    pub resources: Vec<McpResourceDefinition>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resources_read_params_serde() {
        let p = ResourcesReadParams {
            uri: "file:///data.txt".into(),
        };
        let s = serde_json::to_string(&p).expect("ser");
        let back: ResourcesReadParams = serde_json::from_str(&s).expect("de");
        assert_eq!(back.uri, "file:///data.txt");
    }

    #[test]
    fn resource_definition_roundtrip() {
        let def = McpResourceDefinition {
            uri: "file:///x".into(),
            name: "config".into(),
            description: Some("Config file".into()),
            mime_type: Some("text/plain".into()),
        };
        let s = serde_json::to_string(&def).expect("ser");
        let back: McpResourceDefinition = serde_json::from_str(&s).expect("de");
        assert_eq!(back.name, "config");
        assert_eq!(back.mime_type, Some("text/plain".into()));
    }

    #[test]
    fn resources_list_result_roundtrip() {
        let r = ResourcesListResult {
            resources: vec![McpResourceDefinition {
                uri: "x".into(),
                name: "r".into(),
                description: None,
                mime_type: None,
            }],
        };
        let s = serde_json::to_string(&r).expect("ser");
        let back: ResourcesListResult = serde_json::from_str(&s).expect("de");
        assert_eq!(back.resources.len(), 1);
    }

    #[test]
    fn resource_definition_optional_fields_skipped() {
        let def = McpResourceDefinition {
            uri: "x".into(),
            name: "r".into(),
            description: None,
            mime_type: None,
        };
        let s = serde_json::to_string(&def).expect("ser");
        assert!(!s.contains("description"));
        assert!(!s.contains("mime_type"));
    }
}
