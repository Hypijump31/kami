//! MCP (Model Context Protocol) type definitions.

pub mod initialize;
pub mod prompts;
pub mod resources;
pub mod tools;

pub use initialize::*;
pub use prompts::*;
pub use resources::*;
pub use tools::*;

/// MCP protocol method names.
pub mod methods {
    /// Handshake request.
    pub const INITIALIZE: &str = "initialize";
    /// Client notification after successful initialization.
    pub const NOTIFICATIONS_INITIALIZED: &str = "notifications/initialized";
    /// List available tools.
    pub const TOOLS_LIST: &str = "tools/list";
    /// Execute a tool.
    pub const TOOLS_CALL: &str = "tools/call";
    /// List available prompts.
    pub const PROMPTS_LIST: &str = "prompts/list";
    /// List available resources.
    pub const RESOURCES_LIST: &str = "resources/list";
    /// Read a resource.
    pub const RESOURCES_READ: &str = "resources/read";
}
