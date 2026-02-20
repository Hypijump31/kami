//! Bearer token authentication for MCP HTTP requests.

use axum::http::{header, HeaderMap, StatusCode};

/// Validates the `Authorization: Bearer <token>` header.
///
/// # Errors
///
/// Returns `StatusCode::UNAUTHORIZED` if the header is absent or the
/// token does not match `expected`.
pub(crate) fn validate_bearer(headers: &HeaderMap, expected: &str) -> Result<(), StatusCode> {
    let provided = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    if provided == Some(expected) {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn valid_bearer_passes() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer secret123"),
        );
        assert!(validate_bearer(&headers, "secret123").is_ok());
    }

    #[test]
    fn wrong_token_rejected() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer wrong"),
        );
        assert!(validate_bearer(&headers, "secret123").is_err());
    }

    #[test]
    fn missing_header_rejected() {
        let headers = HeaderMap::new();
        assert!(validate_bearer(&headers, "secret123").is_err());
    }

    #[test]
    fn basic_auth_scheme_rejected() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Basic secret123"),
        );
        assert!(validate_bearer(&headers, "secret123").is_err());
    }
}
