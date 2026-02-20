//! Handles the `tools/call` MCP method.

use serde_json::Value;

use kami_protocol::mcp::tools::{ToolContent, ToolsCallParams, ToolsCallResult};
use kami_protocol::{error_codes, JsonRpcErrorResponse, JsonRpcResponse, RequestId};
use kami_runtime::KamiRuntime;
use kami_types::ToolId;

use crate::handler::JsonRpcOutput;

/// Handles the `tools/call` request.
pub(crate) async fn handle_tools_call(
    id: RequestId,
    params: &Option<Value>,
    runtime: &KamiRuntime,
) -> JsonRpcOutput {
    // 1. Parse params
    let call_params = match params {
        Some(p) => match serde_json::from_value::<ToolsCallParams>(p.clone()) {
            Ok(cp) => cp,
            Err(e) => {
                return JsonRpcOutput::Error(JsonRpcErrorResponse::error(
                    id,
                    error_codes::INVALID_PARAMS,
                    format!("invalid tools/call params: {e}"),
                ));
            }
        },
        None => {
            return JsonRpcOutput::Error(JsonRpcErrorResponse::error(
                id,
                error_codes::INVALID_PARAMS,
                "tools/call requires params",
            ));
        }
    };

    // 2. Parse tool ID
    let tool_id = match ToolId::new(&call_params.name) {
        Ok(tid) => tid,
        Err(e) => {
            return JsonRpcOutput::Error(JsonRpcErrorResponse::error(
                id,
                error_codes::INVALID_PARAMS,
                format!("invalid tool name: {e}"),
            ));
        }
    };

    // 3. Execute via runtime
    let input = call_params.arguments.to_string();
    tracing::debug!(%tool_id, "executing tool via MCP");

    let (content, is_error) = match runtime.execute(&tool_id, &input).await {
        Ok(result) => (result.content, !result.success),
        Err(e) => (e.to_string(), true),
    };

    let call_result = ToolsCallResult {
        content: vec![ToolContent::Text { text: content }],
        is_error,
    };

    match serde_json::to_value(call_result) {
        Ok(v) => JsonRpcOutput::Success(JsonRpcResponse::success(id, v)),
        Err(e) => JsonRpcOutput::Error(JsonRpcErrorResponse::error(
            id,
            error_codes::INTERNAL_ERROR,
            e.to_string(),
        )),
    }
}
