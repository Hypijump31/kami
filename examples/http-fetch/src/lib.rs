//! HTTP fetch KAMI tool — demonstrates network access.
//!
//! Takes a URL and returns the HTTP response body.
//! Requires `net_allow_list` in `tool.toml` to permit outbound connections.
//!
//! **Note:** This example shows the tool *logic* for URL validation and
//! response formatting. Actual HTTP is performed by the WASI HTTP outgoing
//! handler at runtime — the tool receives the response body from the host.

use kami_guest::kami_tool;
use serde::Deserialize;

kami_tool! {
    name: "dev.kami.http-fetch",
    version: "0.1.0",
    description: "Fetches content from a URL via HTTP GET",
    handler: handle,
}

/// Maximum response size (64 KiB) to avoid unbounded memory usage.
const DEFAULT_MAX_BYTES: u64 = 65_536;

/// Input schema for the http-fetch tool.
#[derive(Deserialize)]
struct Input {
    /// URL to fetch (must start with `http://` or `https://`).
    url: String,
    /// Optional maximum response body size in bytes.
    max_bytes: Option<u64>,
}

/// Validates that the URL has an acceptable scheme.
fn validate_url(url: &str) -> Result<(), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!(
            "unsupported URL scheme: expected http:// or https://, got '{url}'"
        ));
    }
    if url.len() > 2048 {
        return Err("URL exceeds maximum length (2048 bytes)".to_string());
    }
    Ok(())
}

fn handle(input: &str) -> Result<String, String> {
    let args: Input = kami_guest::parse_input(input)?;
    validate_url(&args.url)?;

    let max = args.max_bytes.unwrap_or(DEFAULT_MAX_BYTES);

    // In a real WASI HTTP tool, this would call wasi:http/outgoing-handler.
    // Here we return a structured placeholder showing what would be fetched.
    let response = serde_json::json!({
        "url": args.url,
        "max_bytes": max,
        "status": "ready",
        "note": "Actual HTTP fetch requires wasi:http/outgoing-handler"
    });

    kami_guest::text_result(&response.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_https_url() {
        let result = handle(r#"{"url":"https://example.com"}"#);
        assert!(result.is_ok());
        let body = result.unwrap();
        assert!(body.contains("example.com"));
    }

    #[test]
    fn valid_http_url() {
        let result = handle(r#"{"url":"http://api.local/data"}"#);
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_unsupported_scheme() {
        let result = handle(r#"{"url":"ftp://evil.com/file"}"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("unsupported URL scheme"));
    }

    #[test]
    fn rejects_missing_url() {
        let result = handle(r#"{}"#);
        assert!(result.is_err());
    }

    #[test]
    fn custom_max_bytes() {
        let result = handle(r#"{"url":"https://example.com","max_bytes":1024}"#);
        assert!(result.is_ok());
        let body = result.unwrap();
        assert!(body.contains("1024"));
    }

    #[test]
    fn validate_url_rejects_too_long() {
        let long = format!("https://example.com/{}", "a".repeat(2040));
        assert!(validate_url(&long).is_err());
    }
}
