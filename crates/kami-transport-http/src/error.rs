//! Error types for the HTTP/MCP transport layer.

use thiserror::Error;

/// Errors that can occur in the HTTP transport.
#[derive(Debug, Error)]
pub enum HttpTransportError {
    /// Failed to bind to the TCP address.
    #[error("failed to bind on {addr}: {source}")]
    Bind {
        /// The address string.
        addr: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The HTTP server encountered an I/O error while serving.
    #[error("server error: {0}")]
    Serve(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_error_displays_address() {
        let err = HttpTransportError::Bind {
            addr: "127.0.0.1:8080".into(),
            source: std::io::Error::new(std::io::ErrorKind::AddrInUse, "in use"),
        };
        let msg = err.to_string();
        assert!(msg.contains("127.0.0.1:8080"));
    }

    #[test]
    fn serve_error_displays_message() {
        let err = HttpTransportError::Serve("connection reset".into());
        assert!(err.to_string().contains("connection reset"));
    }
}
