//! # kami-types
//!
//! Domain types for the KAMI orchestrator.
//! Zero external dependencies except `serde` for serialization.
//! No filesystem I/O — `tool.toml` parsing is in `kami-config`.

pub mod capability;
pub mod error;
pub mod event;
pub mod manifest;
pub mod tool;
pub mod version;

// Re-exports for convenience.
pub use capability::{Capability, FsAccess, ResourceLimits, SecurityConfig};
pub use error::{DiagnosticError, ErrorKind, KamiError};
pub use event::DomainEvent;
pub use tool::{Tool, ToolArgument, ToolId, ToolManifest, ToolVersion};
