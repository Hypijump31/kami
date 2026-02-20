//! ABI helpers for guest WASM modules.
//!
//! Provides the low-level interface between guest tool code and the
//! KAMI host. Guest tools use these helpers to parse input, build
//! results, and interact with host functions.

use serde::de::DeserializeOwned;
use serde::Serialize;

/// Parses JSON input string into a typed struct.
///
/// Returns `Err(String)` with a human-readable message on failure.
///
/// # Example
/// ```ignore
/// #[derive(serde::Deserialize)]
/// struct MyInput { url: String }
///
/// let input: MyInput = kami_guest::abi::parse_input(raw_json)?;
/// ```
pub fn parse_input<T: DeserializeOwned>(input: &str) -> Result<T, String> {
    serde_json::from_str(input).map_err(|e| format!("invalid input: {e}"))
}

/// Serializes a value into a JSON result string.
///
/// Returns `Err(String)` if serialization fails.
pub fn to_output<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| format!("serialization error: {e}"))
}

/// Builds a simple text result.
pub fn text_result(text: &str) -> Result<String, String> {
    Ok(serde_json::json!({ "text": text }).to_string())
}

/// Builds an error result.
pub fn error_result(message: &str) -> String {
    serde_json::json!({ "error": message }).to_string()
}

/// Tool metadata for the `describe` export.
#[derive(Debug, Clone, Serialize)]
pub struct ToolMetadata {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// Tool version.
    pub version: String,
}

impl ToolMetadata {
    /// Serializes metadata to JSON for the `describe` ABI call.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestInput {
        url: String,
        count: u32,
    }

    #[test]
    fn parse_valid_input() {
        let json = r#"{"url":"https://example.com","count":5}"#;
        let input: TestInput = parse_input(json).expect("parse");
        assert_eq!(input.url, "https://example.com");
        assert_eq!(input.count, 5);
    }

    #[test]
    fn parse_invalid_input_returns_error() {
        let result = parse_input::<TestInput>("not json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid input"));
    }

    #[test]
    fn to_output_serializes() {
        let data = serde_json::json!({"result": 42});
        let json = to_output(&data).expect("serialize");
        assert!(json.contains("42"));
    }

    #[test]
    fn text_result_wraps_string() {
        let result = text_result("hello").expect("ok");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("parse");
        assert_eq!(parsed["text"], "hello");
    }

    #[test]
    fn error_result_wraps_message() {
        let result = error_result("something failed");
        let parsed: serde_json::Value = serde_json::from_str(&result).expect("parse");
        assert_eq!(parsed["error"], "something failed");
    }

    #[test]
    fn tool_metadata_to_json() {
        let meta = ToolMetadata {
            name: "test-tool".to_string(),
            description: "A test".to_string(),
            version: "1.0.0".to_string(),
        };
        let json = meta.to_json();
        assert!(json.contains("test-tool"));
        assert!(json.contains("1.0.0"));
    }
}
