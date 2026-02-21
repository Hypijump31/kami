//! # kami-mcp
//!
//! MCP method dispatch handler (APPLICATION layer).
//!
//! Provides `McpHandler` and `JsonRpcOutput` for routing JSON-RPC requests
//! to the appropriate MCP method implementations.

mod dispatch;
pub mod handler;

pub use handler::{JsonRpcOutput, McpHandler};
