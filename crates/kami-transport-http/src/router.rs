//! Axum router for the MCP HTTP/JSON-RPC transport.
//! Routes: `POST /mcp` (requests), `GET /health` (liveness), `GET /health/ready` (readiness).

use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};

use kami_mcp::McpHandler;
use kami_protocol::{error_codes, JsonRpcNotification, JsonRpcRequest};

use crate::auth;

/// Shared state threaded through all axum handlers.
#[derive(Clone)]
pub struct AppState {
    /// The MCP request dispatcher.
    pub handler: Arc<McpHandler>,
    /// Optional Bearer token (None = no authentication required).
    pub token: Option<String>,
}

/// Builds the axum `Router` with all MCP routes.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/mcp", post(handle_mcp))
        .route("/health", get(handle_health))
        .route("/health/ready", get(handle_ready))
        .with_state(state)
}

async fn handle_health() -> impl IntoResponse {
    Json(json!({"status": "ok", "service": "kami"}))
}

/// Readiness probe — returns `200 OK` once the server is accepting requests.
async fn handle_ready() -> impl IntoResponse {
    Json(json!({"status": "ready", "service": "kami"}))
}

async fn handle_mcp(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    if let Some(ref token) = state.token {
        if auth::validate_bearer(&headers, token).is_err() {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "unauthorized"})),
            )
                .into_response();
        }
    }

    let json_val: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => return json_rpc_error(StatusCode::OK, error_codes::PARSE_ERROR, "Parse error"),
    };

    let has_id = json_val.get("id").is_some_and(|v| !v.is_null());
    if !has_id {
        if let Ok(notif) = serde_json::from_value::<JsonRpcNotification>(json_val) {
            state.handler.handle_notification(&notif);
        }
        return StatusCode::NO_CONTENT.into_response();
    }

    let request: JsonRpcRequest = match serde_json::from_value(json_val) {
        Ok(r) => r,
        Err(e) => {
            return json_rpc_error(
                StatusCode::OK,
                error_codes::INVALID_REQUEST,
                &format!("Invalid request: {e}"),
            )
        }
    };

    let output = state.handler.dispatch(&request).await;
    match output.to_json() {
        Ok(json_str) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            json_str,
        )
            .into_response(),
        Err(e) => json_rpc_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            error_codes::INTERNAL_ERROR,
            &e.to_string(),
        ),
    }
}

/// Produces a JSON-RPC error response without a request ID (id: null).
fn json_rpc_error(status: StatusCode, code: i32, message: &str) -> axum::response::Response {
    let body = json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": { "code": code, "message": message }
    });
    (status, Json(body)).into_response()
}
