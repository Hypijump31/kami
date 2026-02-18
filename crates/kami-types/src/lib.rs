//! # kami-types
//!
//! Domain types for the KAMI orchestrator.
//! This crate contains pure data types with zero external dependencies
//! (except serde for serialization).

pub mod capability;
pub mod error;
pub mod event;
pub mod manifest;
pub mod tool;

// Re-exports for convenience.
pub use capability::{Capability, FsAccess, ResourceLimits, SecurityConfig};
pub use error::{ErrorKind, KamiError};
pub use event::DomainEvent;
pub use manifest::{parse_tool_manifest, parse_tool_manifest_file};
pub use tool::{Tool, ToolArgument, ToolId, ToolManifest, ToolVersion};
