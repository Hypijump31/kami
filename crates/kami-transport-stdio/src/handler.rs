//! Stdio message handler for MCP protocol.

use thiserror::Error;

/// Errors from the transport layer.
#[derive(Debug, Error)]
pub enum TransportError {
    /// Failed to read from stdin.
    #[error("read error: {0}")]
    Read(String),
    /// Failed to write to stdout.
    #[error("write error: {0}")]
    Write(String),
    /// Failed to parse incoming message.
    #[error("parse error: {0}")]
    Parse(String),
}
