//! Line-delimited JSON transport over stdin/stdout.
//!
//! Each JSON-RPC message is a single line terminated by `\n`.
//! This follows the MCP stdio transport specification.

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::trace;

use crate::error::TransportError;

/// Reads JSON-RPC messages from stdin, writes responses to stdout.
///
/// Uses line-delimited JSON: one complete JSON object per line.
/// This struct is generic over reader/writer for testability.
pub struct StdioTransport<R, W> {
    reader: BufReader<R>,
    writer: W,
}

impl<R, W> StdioTransport<R, W>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    /// Creates a new transport with the given reader and writer.
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader: BufReader::new(reader),
            writer,
        }
    }

    /// Reads the next line from the input stream.
    ///
    /// Returns `None` on EOF (connection closed).
    pub async fn read_line(
        &mut self,
    ) -> Result<Option<String>, TransportError> {
        let mut line = String::new();
        let bytes_read = self
            .reader
            .read_line(&mut line)
            .await
            .map_err(|e| TransportError::Read(e.to_string()))?;

        if bytes_read == 0 {
            return Ok(None);
        }

        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            return Ok(Some(String::new()));
        }

        trace!(len = trimmed.len(), "read message");
        Ok(Some(trimmed))
    }

    /// Writes a JSON-RPC response line to the output stream.
    pub async fn write_line(
        &mut self,
        message: &str,
    ) -> Result<(), TransportError> {
        trace!(len = message.len(), "writing message");

        self.writer
            .write_all(message.as_bytes())
            .await
            .map_err(|e| TransportError::Write(e.to_string()))?;

        self.writer
            .write_all(b"\n")
            .await
            .map_err(|e| TransportError::Write(e.to_string()))?;

        self.writer
            .flush()
            .await
            .map_err(|e| TransportError::Write(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[tokio::test]
    async fn read_single_line() {
        let input = b"{\"jsonrpc\":\"2.0\"}\n";
        let reader = Cursor::new(input.to_vec());
        let writer = Vec::new();
        let mut transport = StdioTransport::new(reader, writer);

        let line = transport.read_line().await.expect("read");
        assert_eq!(line, Some("{\"jsonrpc\":\"2.0\"}".to_string()));
    }

    #[tokio::test]
    async fn read_eof_returns_none() {
        let reader = Cursor::new(Vec::<u8>::new());
        let writer = Vec::new();
        let mut transport = StdioTransport::new(reader, writer);

        let line = transport.read_line().await.expect("read");
        assert_eq!(line, None);
    }

    #[tokio::test]
    async fn write_appends_newline() {
        let reader = Cursor::new(Vec::<u8>::new());
        let writer = Vec::new();
        let mut transport = StdioTransport::new(reader, writer);

        transport.write_line("{\"ok\":true}").await.expect("write");

        let output =
            String::from_utf8(transport.writer.clone()).expect("utf8");
        assert_eq!(output, "{\"ok\":true}\n");
    }

    #[tokio::test]
    async fn read_multiple_lines() {
        let input = b"line1\nline2\nline3\n";
        let reader = Cursor::new(input.to_vec());
        let writer = Vec::new();
        let mut transport = StdioTransport::new(reader, writer);

        let l1 = transport.read_line().await.expect("r1");
        let l2 = transport.read_line().await.expect("r2");
        let l3 = transport.read_line().await.expect("r3");
        let l4 = transport.read_line().await.expect("r4");

        assert_eq!(l1, Some("line1".to_string()));
        assert_eq!(l2, Some("line2".to_string()));
        assert_eq!(l3, Some("line3".to_string()));
        assert_eq!(l4, None);
    }
}
