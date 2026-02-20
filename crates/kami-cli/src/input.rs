//! Input resolution for CLI commands.
//!
//! Supports three input sources:
//! - Inline JSON string via `--input` / `-i`
//! - JSON file via `--input-file` / `-f`
//! - Stdin via `--input-file -`

use std::io::Read;
use std::path::Path;

/// Resolves the final JSON input string from CLI arguments.
///
/// Priority: `--input-file` takes precedence over `--input`.
/// If `input_file` is `Some("-")`, reads from stdin.
/// If `input_file` is `Some(path)`, reads from that file.
/// Otherwise, returns the `input` string as-is.
pub fn resolve_input(input: &str, input_file: Option<&str>) -> anyhow::Result<String> {
    match input_file {
        Some("-") => read_from_stdin(),
        Some(path) => read_from_file(path),
        None => Ok(input.to_string()),
    }
}

/// Reads JSON content from a file path.
fn read_from_file(path: &str) -> anyhow::Result<String> {
    let file_path = Path::new(path);
    if !file_path.exists() {
        anyhow::bail!("input file not found: {path}");
    }
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| anyhow::anyhow!("failed to read input file: {e}"))?;
    let trimmed = content.trim().to_string();
    validate_json(&trimmed)?;
    Ok(trimmed)
}

/// Reads JSON content from stdin.
fn read_from_stdin() -> anyhow::Result<String> {
    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .map_err(|e| anyhow::anyhow!("failed to read stdin: {e}"))?;
    let trimmed = buffer.trim().to_string();
    validate_json(&trimmed)?;
    Ok(trimmed)
}

/// Validates that the input is valid JSON.
fn validate_json(input: &str) -> anyhow::Result<()> {
    serde_json::from_str::<serde_json::Value>(input)
        .map_err(|e| anyhow::anyhow!("invalid JSON input: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_inline_input() {
        let result = resolve_input("{\"key\":\"value\"}", None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "{\"key\":\"value\"}");
    }

    #[test]
    fn resolve_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("input.json");
        std::fs::write(&file_path, r#"{"hello": "world"}"#).unwrap();

        let result = resolve_input("{}", Some(file_path.to_str().unwrap()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), r#"{"hello": "world"}"#);
    }

    #[test]
    fn resolve_file_not_found() {
        let result = resolve_input("{}", Some("/nonexistent/file.json"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"));
    }

    #[test]
    fn resolve_invalid_json_in_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("bad.json");
        std::fs::write(&file_path, "not valid json").unwrap();

        let result = resolve_input("{}", Some(file_path.to_str().unwrap()));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid JSON"));
    }

    #[test]
    fn resolve_file_with_whitespace() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("spaced.json");
        std::fs::write(&file_path, "  {\"a\": 1}  \n").unwrap();

        let result = resolve_input("{}", Some(file_path.to_str().unwrap()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "{\"a\": 1}");
    }

    #[test]
    fn resolve_inline_fallback_when_no_file() {
        let result = resolve_input("{\"default\":true}", None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "{\"default\":true}");
    }
}
