//! HTTP/JSON-RPC transport adapter for KAMI.
//! Exposes MCP over `POST /mcp` with optional Bearer token authentication.

pub mod auth;
mod error;
pub mod router;
pub mod server;

pub use error::HttpTransportError;
pub use router::{build_router, AppState};
pub use server::HttpServer;
