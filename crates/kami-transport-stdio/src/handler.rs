//! MCP method dispatch handler.
//!
//! Routes incoming JSON-RPC requests to the appropriate MCP method
//! handler (initialize, tools/list, tools/call) and builds responses.

use std::sync::Arc;

use serde_json::Value;
use tracing::{debug, warn};

use kami_protocol::mcp::initialize::{
    InitializeParams, InitializeResult, ServerCapabilities, ServerInfo,
    ToolCapability, PROTOCOL_VERSION,
};
use kami_protocol::mcp::methods;
use kami_protocol::mcp::tools::{
    McpToolDefinition, ToolContent, ToolsCallParams, ToolsCallResult,
    ToolsListResult,
};
use kami_protocol::{
    JsonRpcErrorResponse, JsonRpcRequest, JsonRpcResponse, RequestId,
    error_codes,
};
use kami_registry::ToolRepository;
use kami_runtime::KamiRuntime;
use kami_types::ToolId;

use crate::error::TransportError;

/// Handles MCP method dispatch over JSON-RPC.
///
/// Combines a `KamiRuntime` (for tool execution) and a
/// `ToolRepository` (for tool listing) to serve MCP requests.
pub struct McpHandler {
    runtime: Arc<KamiRuntime>,
    repository: Arc<dyn ToolRepository>,
}

/// Enum representing either a success or error JSON-RPC response.
#[derive(Debug)]
pub enum JsonRpcOutput {
    /// Success response.
    Success(JsonRpcResponse),
    /// Error response.
    Error(JsonRpcErrorResponse),
}

impl JsonRpcOutput {
    /// Serializes the output to a JSON string.
    pub fn to_json(&self) -> Result<String, TransportError> {
        match self {
            Self::Success(r) => serde_json::to_string(r),
            Self::Error(r) => serde_json::to_string(r),
        }
        .map_err(|e| TransportError::Write(e.to_string()))
    }
}

impl McpHandler {
    /// Creates a new handler with the given runtime and repository.
    pub fn new(
        runtime: Arc<KamiRuntime>,
        repository: Arc<dyn ToolRepository>,
    ) -> Self {
        Self {
            runtime,
            repository,
        }
    }

    /// Dispatches a JSON-RPC request to the appropriate method handler.
    pub async fn dispatch(
        &self,
        request: &JsonRpcRequest,
    ) -> JsonRpcOutput {
        debug!(method = %request.method, "dispatching MCP request");

        match request.method.as_str() {
            methods::INITIALIZE => {
                self.handle_initialize(&request.id, &request.params)
            }
            methods::TOOLS_LIST => {
                self.handle_tools_list(&request.id).await
            }
            methods::TOOLS_CALL => {
                self.handle_tools_call(&request.id, &request.params)
                    .await
            }
            other => {
                warn!(method = other, "unknown MCP method");
                JsonRpcOutput::Error(JsonRpcErrorResponse::error(
                    request.id.clone(),
                    error_codes::METHOD_NOT_FOUND,
                    format!("unknown method: {other}"),
                ))
            }
        }
    }

    /// Handles the `initialize` method.
    fn handle_initialize(
        &self,
        id: &RequestId,
        params: &Option<Value>,
    ) -> JsonRpcOutput {
        // Parse params (optional validation)
        if let Some(p) = params {
            if let Err(e) = serde_json::from_value::<InitializeParams>(
                p.clone(),
            ) {
                return JsonRpcOutput::Error(JsonRpcErrorResponse::error(
                    id.clone(),
                    error_codes::INVALID_PARAMS,
                    format!("invalid initialize params: {e}"),
                ));
            }
        }

        let result = InitializeResult {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolCapability {}),
            },
            server_info: ServerInfo {
                name: "kami".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        match serde_json::to_value(result) {
            Ok(v) => JsonRpcOutput::Success(JsonRpcResponse::success(
                id.clone(),
                v,
            )),
            Err(e) => JsonRpcOutput::Error(JsonRpcErrorResponse::error(
                id.clone(),
                error_codes::INTERNAL_ERROR,
                e.to_string(),
            )),
        }
    }

    /// Handles the `tools/list` method.
    async fn handle_tools_list(
        &self,
        id: &RequestId,
    ) -> JsonRpcOutput {
        let query = kami_registry::ToolQuery::all();

        let tools = match self.repository.find_all(query).await {
            Ok(tools) => tools,
            Err(e) => {
                return JsonRpcOutput::Error(
                    JsonRpcErrorResponse::error(
                        id.clone(),
                        error_codes::INTERNAL_ERROR,
                        format!("registry error: {e}"),
                    ),
                );
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
            Ok(v) => JsonRpcOutput::Success(JsonRpcResponse::success(
                id.clone(),
                v,
            )),
            Err(e) => JsonRpcOutput::Error(JsonRpcErrorResponse::error(
                id.clone(),
                error_codes::INTERNAL_ERROR,
                e.to_string(),
            )),
        }
    }

    /// Handles the `tools/call` method.
    async fn handle_tools_call(
        &self,
        id: &RequestId,
        params: &Option<Value>,
    ) -> JsonRpcOutput {
        // 1. Parse params
        let call_params = match params {
            Some(p) => {
                match serde_json::from_value::<ToolsCallParams>(p.clone())
                {
                    Ok(cp) => cp,
                    Err(e) => {
                        return JsonRpcOutput::Error(
                            JsonRpcErrorResponse::error(
                                id.clone(),
                                error_codes::INVALID_PARAMS,
                                format!("invalid tools/call params: {e}"),
                            ),
                        );
                    }
                }
            }
            None => {
                return JsonRpcOutput::Error(
                    JsonRpcErrorResponse::error(
                        id.clone(),
                        error_codes::INVALID_PARAMS,
                        "tools/call requires params",
                    ),
                );
            }
        };

        // 2. Parse tool ID
        let tool_id = match ToolId::new(&call_params.name) {
            Ok(tid) => tid,
            Err(e) => {
                return JsonRpcOutput::Error(
                    JsonRpcErrorResponse::error(
                        id.clone(),
                        error_codes::INVALID_PARAMS,
                        format!("invalid tool name: {e}"),
                    ),
                );
            }
        };

        // 3. Serialize arguments as JSON string input
        let input = call_params.arguments.to_string();

        // 4. Execute via runtime
        debug!(%tool_id, "executing tool via MCP");

        match self.runtime.execute(&tool_id, &input).await {
            Ok(result) => {
                let call_result = ToolsCallResult {
                    content: vec![ToolContent::Text {
                        text: result.content,
                    }],
                    is_error: !result.success,
                };
                match serde_json::to_value(call_result) {
                    Ok(v) => JsonRpcOutput::Success(
                        JsonRpcResponse::success(id.clone(), v),
                    ),
                    Err(e) => JsonRpcOutput::Error(
                        JsonRpcErrorResponse::error(
                            id.clone(),
                            error_codes::INTERNAL_ERROR,
                            e.to_string(),
                        ),
                    ),
                }
            }
            Err(e) => {
                let call_result = ToolsCallResult {
                    content: vec![ToolContent::Text {
                        text: e.to_string(),
                    }],
                    is_error: true,
                };
                match serde_json::to_value(call_result) {
                    Ok(v) => JsonRpcOutput::Success(
                        JsonRpcResponse::success(id.clone(), v),
                    ),
                    Err(ser_e) => JsonRpcOutput::Error(
                        JsonRpcErrorResponse::error(
                            id.clone(),
                            error_codes::INTERNAL_ERROR,
                            ser_e.to_string(),
                        ),
                    ),
                }
            }
        }
    }
}

/// Builds a JSON Schema `inputSchema` from tool arguments.
fn build_input_schema(
    arguments: &[kami_types::ToolArgument],
) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for arg in arguments {
        let mut prop = serde_json::Map::new();
        prop.insert(
            "type".to_string(),
            Value::String(arg.arg_type.clone()),
        );
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
    schema.insert(
        "type".to_string(),
        Value::String("object".to_string()),
    );
    schema.insert(
        "properties".to_string(),
        Value::Object(properties),
    );
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
        assert!(schema["properties"]["url"]["type"] == "string");
        assert!(schema["properties"]["timeout"]["type"] == "number");
        let req = schema["required"].as_array().expect("arr");
        assert_eq!(req.len(), 1);
        assert_eq!(req[0], "url");
    }
}
