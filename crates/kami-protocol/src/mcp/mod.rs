//! MCP (Model Context Protocol) type definitions.

pub mod prompts;
pub mod resources;
pub mod tools;

pub use prompts::*;
pub use resources::*;
pub use tools::*;

/// MCP protocol method names.
pub mod methods {
    pub const INITIALIZE: &str = "initialize";
    pub const TOOLS_LIST: &str = "tools/list";
    pub const TOOLS_CALL: &str = "tools/call";
    pub const PROMPTS_LIST: &str = "prompts/list";
    pub const RESOURCES_LIST: &str = "resources/list";
    pub const RESOURCES_READ: &str = "resources/read";
}
