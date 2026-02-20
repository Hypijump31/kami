//! Integration tests for JSON-RPC 2.0 types.

use kami_protocol::{
    error_codes, JsonRpcErrorResponse, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
    RequestId,
};
use serde_json::json;

#[test]
fn request_serialization() {
    let req = JsonRpcRequest::new(RequestId::Number(1), "tools/list", None);
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"method\":\"tools/list\""));
}

#[test]
fn response_roundtrip() {
    let resp = JsonRpcResponse::success(
        RequestId::String("abc".into()),
        serde_json::json!({"tools": []}),
    );
    let json = serde_json::to_string(&resp).unwrap();
    let back: JsonRpcResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, RequestId::String("abc".into()));
}

#[test]
fn error_response() {
    let err = JsonRpcErrorResponse::error(
        RequestId::Number(1),
        error_codes::METHOD_NOT_FOUND,
        "method not found",
    );
    assert_eq!(err.error.code, -32601);
}

#[test]
fn request_serde_roundtrip() {
    let req = JsonRpcRequest::new(RequestId::Number(1), "test", Some(json!({"a": 1})));
    let s = serde_json::to_string(&req).expect("ser");
    let back: JsonRpcRequest = serde_json::from_str(&s).expect("de");
    assert_eq!(back.method, "test");
    assert_eq!(back.id, RequestId::Number(1));
}

#[test]
fn response_success_roundtrip() {
    let resp = JsonRpcResponse::success(RequestId::String("x".into()), json!(42));
    assert_eq!(resp.jsonrpc, "2.0");
    let s = serde_json::to_string(&resp).expect("ser");
    let back: JsonRpcResponse = serde_json::from_str(&s).expect("de");
    assert_eq!(back.result, json!(42));
}

#[test]
fn error_response_structure() {
    let err = JsonRpcErrorResponse::error(RequestId::Number(1), -32600, "invalid");
    assert_eq!(err.error.code, -32600);
    assert_eq!(err.error.message, "invalid");
    assert!(err.error.data.is_none());
}

#[test]
fn notification_deserializes_without_id() {
    let s = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let n: JsonRpcNotification = serde_json::from_str(s).expect("de");
    assert_eq!(n.method, "notifications/initialized");
    assert!(n.params.is_none());
}

#[test]
fn request_id_number_vs_string() {
    assert_ne!(RequestId::Number(1), RequestId::String("1".into()));
    assert_eq!(RequestId::Number(42), RequestId::Number(42));
}
