//! Handles the `tools/list` MCP method.

use serde_json::Value;

use kami_protocol::mcp::tools::{McpToolDefinition, ToolsListResult};
use kami_protocol::{error_codes, JsonRpcErrorResponse, JsonRpcResponse, RequestId};
use kami_registry::{ToolQuery, ToolRepository};
use kami_types::ToolArgument;

use crate::handler::JsonRpcOutput;

/// Handles the `tools/list` request.
pub(crate) async fn handle_tools_list(
    id: RequestId,
    repository: &dyn ToolRepository,
) -> JsonRpcOutput {
    let tools = match repository.find_all(ToolQuery::all()).await {
        Ok(t) => t,
        Err(e) => {
            return JsonRpcOutput::Error(JsonRpcErrorResponse::error(
                id,
                error_codes::INTERNAL_ERROR,
                format!("registry error: {e}"),
            ));
        }
    };

    let definitions: Vec<McpToolDefinition> = tools
        .iter()
        .filter(|t| t.enabled)
        .map(|t| {
            let m = &t.manifest;
            McpToolDefinition {
                name: m.id.to_string(),
                description: Some(m.description.clone()),
                input_schema: build_input_schema(&m.arguments),
            }
        })
        .collect();

    let result = ToolsListResult {
        tools: definitions,
        next_cursor: None,
    };

    match serde_json::to_value(result) {
        Ok(v) => JsonRpcOutput::Success(JsonRpcResponse::success(id, v)),
        Err(e) => JsonRpcOutput::Error(JsonRpcErrorResponse::error(
            id,
            error_codes::INTERNAL_ERROR,
            e.to_string(),
        )),
    }
}

/// Builds a JSON Schema `inputSchema` from tool arguments.
pub(crate) fn build_input_schema(arguments: &[ToolArgument]) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for arg in arguments {
        let mut prop = serde_json::Map::new();
        prop.insert("type".to_string(), Value::String(arg.arg_type.clone()));
        prop.insert(
            "description".to_string(),
            Value::String(arg.description.clone()),
        );
        properties.insert(arg.name.clone(), Value::Object(prop));
        if arg.required {
            required.push(Value::String(arg.name.clone()));
        }
    }

    let mut schema = serde_json::Map::new();
    schema.insert("type".to_string(), Value::String("object".to_string()));
    schema.insert("properties".to_string(), Value::Object(properties));
    if !required.is_empty() {
        schema.insert("required".to_string(), Value::Array(required));
    }

    Value::Object(schema)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kami_types::ToolArgument;

    #[test]
    fn build_input_schema_empty_args() {
        let schema = build_input_schema(&[]);
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].as_object().expect("obj").is_empty());
        assert!(schema.get("required").is_none());
    }

    #[test]
    fn build_input_schema_with_args() {
        let args = vec![
            ToolArgument {
                name: "url".to_string(),
                arg_type: "string".to_string(),
                description: "The URL".to_string(),
                required: true,
                default: None,
            },
            ToolArgument {
                name: "timeout".to_string(),
                arg_type: "number".to_string(),
                description: "Timeout in ms".to_string(),
                required: false,
                default: Some("5000".to_string()),
            },
        ];
        let schema = build_input_schema(&args);
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"]["url"]["type"], "string");
        assert_eq!(schema["properties"]["timeout"]["type"], "number");
        let req = schema["required"].as_array().expect("arr");
        assert_eq!(req.len(), 1);
        assert_eq!(req[0], "url");
    }
}
