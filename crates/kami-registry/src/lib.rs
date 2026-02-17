//! # kami-registry
//!
//! Port definitions (abstract traits) for the tool registry.
//! Adapter crates implement these traits.

pub mod query;
pub mod repository;

pub use query::ToolQuery;
pub use repository::{RepositoryError, ToolRepository};
