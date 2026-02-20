//! Property-based tests for JSON-RPC deserialization.
//!
//! Ensures the parser never panics on arbitrary JSON and that valid
//! requests successfully round-trip through serde.

use kami_protocol::jsonrpc::JsonRpcRequest;
use proptest::prelude::*;

proptest! {
    /// Arbitrary JSON strings never cause a panic.
    #[test]
    fn no_panic_on_arbitrary_json(input in "\\PC{0,256}") {
        let _ = serde_json::from_str::<JsonRpcRequest>(&input);
    }

    /// Well-formed JSON-RPC requests round-trip through serde.
    #[test]
    fn valid_request_roundtrips(
        method in "[a-z/]{1,32}",
        id in any::<i64>(),
    ) {
        let json = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
        });
        let parsed: Result<JsonRpcRequest, _> = serde_json::from_value(json);
        prop_assert!(parsed.is_ok(), "rejected valid request");

        let req = parsed.expect("test: already checked");
        let serialized = serde_json::to_string(&req);
        prop_assert!(serialized.is_ok());

        let reparsed: Result<JsonRpcRequest, _> =
            serde_json::from_str(&serialized.expect("test: already checked"));
        prop_assert!(reparsed.is_ok());
    }

    /// Missing "jsonrpc" field causes parse failure.
    #[test]
    fn missing_jsonrpc_field_fails(method in "[a-z]{2,16}", id in 1i64..1000) {
        let json = format!(r#"{{"id":{id},"method":"{method}"}}"#);
        let result = serde_json::from_str::<JsonRpcRequest>(&json);
        prop_assert!(result.is_err());
    }

    /// Missing "method" field causes parse failure.
    #[test]
    fn missing_method_field_fails(id in 1i64..1000) {
        let json = format!(r#"{{"jsonrpc":"2.0","id":{id}}}"#);
        let result = serde_json::from_str::<JsonRpcRequest>(&json);
        prop_assert!(result.is_err());
    }

    /// Missing "id" field causes parse failure.
    #[test]
    fn missing_id_field_fails(method in "[a-z]{2,16}") {
        let json = format!(r#"{{"jsonrpc":"2.0","method":"{method}"}}"#);
        let result = serde_json::from_str::<JsonRpcRequest>(&json);
        prop_assert!(result.is_err());
    }
}
