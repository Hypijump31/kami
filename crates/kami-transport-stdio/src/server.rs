//! MCP server loop over stdio transport.
//!
//! Reads JSON-RPC requests line by line, dispatches them via
//! `McpHandler`, and writes responses back. JSON-RPC notifications
//! (which have no `id` field) are handled silently without a response.

use tracing::{debug, error, info, warn};

use kami_mcp::{JsonRpcOutput, McpHandler};
use kami_protocol::{
    error_codes, JsonRpcErrorResponse, JsonRpcNotification, JsonRpcRequest, RequestId,
};

use crate::error::TransportError;
use crate::transport::StdioTransport;

/// MCP server that reads from a transport and dispatches to a handler.
pub struct McpServer<R, W> {
    transport: StdioTransport<R, W>,
    handler: McpHandler,
}

impl<R, W> McpServer<R, W>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    /// Creates a new server with the given transport and handler.
    pub fn new(transport: StdioTransport<R, W>, handler: McpHandler) -> Self {
        Self { transport, handler }
    }

    /// Runs the server loop until the transport is closed.
    ///
    /// Each incoming line is parsed as either a JSON-RPC request
    /// (response required) or a notification (silently handled).
    pub async fn run(&mut self) -> Result<(), TransportError> {
        info!("MCP server starting on stdio");

        loop {
            // 1. Read next line
            let line = match self.transport.read_line().await? {
                Some(line) if line.is_empty() => continue,
                Some(line) => line,
                None => {
                    info!("stdin closed, shutting down");
                    return Ok(());
                }
            };

            // 2. Try parsing as a JSON-RPC request (has an `id`)
            match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(request) => {
                    debug!(
                        method = %request.method,
                        id = ?request.id,
                        "received request"
                    );
                    let output = self.handler.dispatch(&request).await;
                    self.write_output(&output).await?;
                }
                Err(_) => {
                    // 3. Fall back: try as notification (no `id` field)
                    match serde_json::from_str::<JsonRpcNotification>(&line) {
                        Ok(notif) => {
                            debug!(
                                method = %notif.method,
                                "received notification"
                            );
                            self.handler.handle_notification(&notif);
                            // Notifications must not receive a response.
                        }
                        Err(e) => {
                            warn!(error = %e, "failed to parse JSON-RPC message");
                            let err = JsonRpcErrorResponse::error(
                                RequestId::Number(0),
                                error_codes::PARSE_ERROR,
                                format!("parse error: {e}"),
                            );
                            self.write_output(&JsonRpcOutput::Error(err)).await?;
                        }
                    }
                }
            }
        }
    }

    /// Serializes and writes a JSON-RPC output to the transport.
    async fn write_output(&mut self, output: &JsonRpcOutput) -> Result<(), TransportError> {
        match output.to_json() {
            Ok(json) => self.transport.write_line(&json).await,
            Err(e) => {
                error!(error = %e, "failed to serialize response");
                Err(TransportError::Write(e.to_string()))
            }
        }
    }
}
