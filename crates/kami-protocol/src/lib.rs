//! # kami-protocol
//!
//! MCP protocol and JSON-RPC 2.0 type definitions.
//! This crate defines the wire format for communication between
//! AI agents and the KAMI orchestrator.

pub mod jsonrpc;
pub mod mcp;
pub mod schema;

pub use jsonrpc::*;
pub use mcp::methods;
