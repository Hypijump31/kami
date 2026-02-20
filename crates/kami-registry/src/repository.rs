//! Abstract repository trait (port) for tool storage.

use async_trait::async_trait;
use thiserror::Error;

use kami_types::{Tool, ToolId};

use crate::query::ToolQuery;

/// Errors returned by repository implementations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RepositoryError {
    /// The requested tool was not found.
    #[error("tool not found: {id}")]
    NotFound { id: String },
    /// A database or I/O error occurred.
    #[error("storage error: {message}")]
    Storage { message: String },
    /// A conflict (duplicate id, etc.).
    #[error("conflict: {message}")]
    Conflict { message: String },
    /// Stored data is corrupt or cannot be deserialized.
    #[error("data corruption for tool '{tool_id}': {message}")]
    DataCorruption { tool_id: String, message: String },
}

/// Abstract trait for tool persistence.
///
/// Implementations live in adapter crates (e.g., `kami-store-sqlite`).
#[async_trait]
pub trait ToolRepository: Send + Sync {
    /// Finds a tool by its unique ID.
    async fn find_by_id(&self, id: &ToolId) -> Result<Option<Tool>, RepositoryError>;

    /// Finds tools matching a query.
    async fn find_all(&self, query: ToolQuery) -> Result<Vec<Tool>, RepositoryError>;

    /// Inserts a new tool.
    async fn insert(&self, tool: &Tool) -> Result<(), RepositoryError>;

    /// Updates an existing tool. Returns `NotFound` if absent.
    async fn update(&self, tool: &Tool) -> Result<(), RepositoryError>;

    /// Deletes a tool by ID. Returns true if it existed.
    async fn delete(&self, id: &ToolId) -> Result<bool, RepositoryError>;
}
