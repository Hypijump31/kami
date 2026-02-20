//! # kami-transport-stdio
//!
//! Stdio transport adapter for MCP JSON-RPC communication.
//!
//! Provides line-delimited JSON transport over stdin/stdout,
//! MCP method dispatch, and a server loop that ties them together.

mod dispatch;
pub mod error;
pub mod handler;
pub mod server;
pub mod transport;

pub use error::TransportError;
pub use handler::McpHandler;
pub use server::McpServer;
pub use transport::StdioTransport;
