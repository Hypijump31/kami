//! MCP server loop over stdio transport.
//!
//! Reads JSON-RPC requests line by line, dispatches them via
//! `McpHandler`, and writes responses back.

use tracing::{debug, error, info, warn};

use kami_protocol::{JsonRpcRequest, error_codes, JsonRpcErrorResponse, RequestId};

use crate::error::TransportError;
use crate::handler::{JsonRpcOutput, McpHandler};
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
    pub fn new(
        transport: StdioTransport<R, W>,
        handler: McpHandler,
    ) -> Self {
        Self { transport, handler }
    }

    /// Runs the server loop until the transport is closed.
    ///
    /// Each line from the transport is parsed as a JSON-RPC request,
    /// dispatched to the handler, and the response written back.
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

            // 2. Parse as JSON-RPC request
            let request = match serde_json::from_str::<JsonRpcRequest>(
                &line,
            ) {
                Ok(req) => req,
                Err(e) => {
                    warn!(error = %e, "failed to parse JSON-RPC request");
                    let error_resp = JsonRpcErrorResponse::error(
                        RequestId::Number(0),
                        error_codes::PARSE_ERROR,
                        format!("parse error: {e}"),
                    );
                    let output = JsonRpcOutput::Error(error_resp);
                    self.write_output(&output).await?;
                    continue;
                }
            };

            debug!(
                method = %request.method,
                id = ?request.id,
                "received request"
            );

            // 3. Dispatch to handler
            let output = self.handler.dispatch(&request).await;

            // 4. Write response
            self.write_output(&output).await?;
        }
    }

    /// Serializes and writes a JSON-RPC output to the transport.
    async fn write_output(
        &mut self,
        output: &JsonRpcOutput,
    ) -> Result<(), TransportError> {
        match output.to_json() {
            Ok(json) => self.transport.write_line(&json).await,
            Err(e) => {
                error!(error = %e, "failed to serialize response");
                Err(e)
            }
        }
    }
}
