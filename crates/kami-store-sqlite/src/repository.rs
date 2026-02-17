//! SQLite implementation of `ToolRepository`.

use std::sync::Mutex;

use async_trait::async_trait;
use kami_registry::{RepositoryError, ToolQuery, ToolRepository};
use kami_types::{Tool, ToolId};

/// SQLite-backed tool repository.
pub struct SqliteToolRepository {
    conn: Mutex<rusqlite::Connection>,
}

impl SqliteToolRepository {
    /// Opens or creates a SQLite database at the given path.
    pub fn open(path: &str) -> Result<Self, RepositoryError> {
        let conn = rusqlite::Connection::open(path).map_err(|e| RepositoryError::Storage {
            message: e.to_string(),
        })?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

#[async_trait]
impl ToolRepository for SqliteToolRepository {
    async fn find_by_id(&self, _id: &ToolId) -> Result<Option<Tool>, RepositoryError> {
        let _conn = self.conn.lock().map_err(|e| RepositoryError::Storage {
            message: e.to_string(),
        })?;
        // TODO: Phase 3 implementation
        Ok(None)
    }

    async fn find_all(&self, _query: ToolQuery) -> Result<Vec<Tool>, RepositoryError> {
        let _conn = self.conn.lock().map_err(|e| RepositoryError::Storage {
            message: e.to_string(),
        })?;
        // TODO: Phase 3 implementation
        Ok(Vec::new())
    }

    async fn insert(&self, _tool: &Tool) -> Result<(), RepositoryError> {
        let _conn = self.conn.lock().map_err(|e| RepositoryError::Storage {
            message: e.to_string(),
        })?;
        // TODO: Phase 3 implementation
        Ok(())
    }

    async fn delete(&self, _id: &ToolId) -> Result<bool, RepositoryError> {
        let _conn = self.conn.lock().map_err(|e| RepositoryError::Storage {
            message: e.to_string(),
        })?;
        // TODO: Phase 3 implementation
        Ok(false)
    }
}
