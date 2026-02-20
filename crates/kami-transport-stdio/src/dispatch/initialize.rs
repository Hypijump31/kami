//! Handles the `initialize` MCP method.

use serde_json::Value;

use kami_protocol::mcp::initialize::{
    InitializeParams, InitializeResult, ServerCapabilities, ServerInfo, ToolCapability,
    PROTOCOL_VERSION,
};
use kami_protocol::{error_codes, JsonRpcErrorResponse, JsonRpcResponse, RequestId};

use crate::handler::JsonRpcOutput;

/// Handles the `initialize` request and returns the server capabilities.
pub(crate) fn handle_initialize(id: RequestId, params: &Option<Value>) -> JsonRpcOutput {
    if let Some(p) = params {
        if let Err(e) = serde_json::from_value::<InitializeParams>(p.clone()) {
            return JsonRpcOutput::Error(JsonRpcErrorResponse::error(
                id,
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
        Ok(v) => JsonRpcOutput::Success(JsonRpcResponse::success(id, v)),
        Err(e) => JsonRpcOutput::Error(JsonRpcErrorResponse::error(
            id,
            error_codes::INTERNAL_ERROR,
            e.to_string(),
        )),
    }
}
