//! Transport-layer error types.

use kami_types::{ErrorKind, KamiError};
use thiserror::Error;

/// Errors from the stdio transport layer.
#[derive(Debug, Error)]
pub enum TransportError {
    /// Failed to read from stdin.
    #[error("read error: {0}")]
    Read(String),
    /// Failed to write to stdout.
    #[error("write error: {0}")]
    Write(String),
    /// Failed to parse incoming JSON message.
    #[error("parse error: {0}")]
    Parse(String),
    /// The connection was closed (EOF on stdin).
    #[error("connection closed")]
    ConnectionClosed,
    /// Handler returned an error during dispatch.
    #[error("dispatch error: {0}")]
    Dispatch(String),
}

impl From<TransportError> for KamiError {
    fn from(e: TransportError) -> Self {
        let kind = match &e {
            TransportError::Parse(_) => ErrorKind::InvalidInput,
            TransportError::ConnectionClosed => ErrorKind::Internal,
            _ => ErrorKind::Internal,
        };
        KamiError::new(kind, e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_converts_to_invalid_input() {
        let err: KamiError = TransportError::Parse("bad json".into()).into();
        assert_eq!(err.kind, ErrorKind::InvalidInput);
    }

    #[test]
    fn connection_closed_converts_to_internal() {
        let err: KamiError = TransportError::ConnectionClosed.into();
        assert_eq!(err.kind, ErrorKind::Internal);
    }

    #[test]
    fn dispatch_error_converts_to_internal() {
        let err: KamiError = TransportError::Dispatch("fail".into()).into();
        assert_eq!(err.kind, ErrorKind::Internal);
    }
}
