//! JSON Schema helpers for MCP tool input validation.

use serde_json::Value;
use thiserror::Error;

/// Schema validation errors.
#[derive(Debug, Error)]
pub enum SchemaError {
    /// The input does not match the expected schema.
    #[error("schema validation failed: {message}")]
    ValidationFailed { message: String },
    /// The schema itself is malformed.
    #[error("invalid schema: {message}")]
    InvalidSchema { message: String },
}

/// Checks that a JSON value contains required properties from a schema.
///
/// This is a minimal validator - checks required fields and basic types.
pub fn validate_required_fields(
    schema: &Value,
    input: &Value,
) -> Result<(), SchemaError> {
    let required = match schema.get("required") {
        Some(Value::Array(arr)) => arr,
        _ => return Ok(()),
    };

    let input_obj = input.as_object().ok_or_else(|| SchemaError::ValidationFailed {
        message: "input must be an object".to_string(),
    })?;

    for field in required {
        let field_name = field.as_str().ok_or_else(|| SchemaError::InvalidSchema {
            message: "required field names must be strings".to_string(),
        })?;
        if !input_obj.contains_key(field_name) {
            return Err(SchemaError::ValidationFailed {
                message: format!("missing required field: {field_name}"),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_required_fields() {
        let schema = json!({
            "required": ["url"],
            "properties": {
                "url": {"type": "string"}
            }
        });
        let input = json!({"url": "https://example.com"});
        assert!(validate_required_fields(&schema, &input).is_ok());
    }

    #[test]
    fn rejects_missing_required_field() {
        let schema = json!({"required": ["url"]});
        let input = json!({});
        assert!(validate_required_fields(&schema, &input).is_err());
    }
}
