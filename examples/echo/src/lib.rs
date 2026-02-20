//! Echo KAMI tool — returns the input unchanged.
//!
//! Demonstrates the simplest possible tool: pass input through verbatim.

use kami_guest::kami_tool;

kami_tool! {
    name: "dev.kami.echo",
    version: "0.1.0",
    description: "Echoes back the JSON input unchanged",
    handler: handle,
}

fn handle(input: &str) -> Result<String, String> {
    // Validate that the input is valid JSON, then return it unmodified.
    let _: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| format!("invalid JSON input: {e}"))?;
    Ok(input.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echoes_object() {
        let input = r#"{"key":"value"}"#;
        let result = handle(input).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn echoes_array() {
        let input = r#"[1,2,3]"#;
        let result = handle(input).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn rejects_invalid_json() {
        let result = handle("not json");
        assert!(result.is_err());
    }
}
