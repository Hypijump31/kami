//! # kami-transport-stdio
//!
//! Stdio transport adapter for MCP JSON-RPC communication.
//!
//! Provides line-delimited JSON transport over stdin/stdout,
//! and a server loop that ties it together.

pub mod error;
pub mod server;
pub mod transport;

pub use error::TransportError;
// McpHandler lives in kami-mcp (APPLICATION layer); re-exported for convenience.
pub use kami_mcp::{JsonRpcOutput, McpHandler};
pub use server::McpServer;
pub use transport::StdioTransport;
