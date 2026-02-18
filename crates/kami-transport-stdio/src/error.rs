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
