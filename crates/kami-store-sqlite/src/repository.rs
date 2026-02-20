//! SQLite implementation of `ToolRepository` — struct and constructors.

use std::sync::Mutex;

use kami_registry::RepositoryError;
use rusqlite::Connection;

use crate::migrations::run_migrations;

/// SQLite-backed tool repository.
pub struct SqliteToolRepository {
    pub(crate) conn: Mutex<Connection>,
}

impl SqliteToolRepository {
    /// Opens or creates a SQLite database at the given path and runs
    /// migrations.
    pub fn open(path: &str) -> Result<Self, RepositoryError> {
        let conn = Connection::open(path).map_err(|e| RepositoryError::Storage {
            message: e.to_string(),
        })?;
        run_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Opens an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self, RepositoryError> {
        let conn = Connection::open_in_memory().map_err(|e| RepositoryError::Storage {
            message: e.to_string(),
        })?;
        run_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Acquires the connection lock.
    pub(crate) fn lock_conn(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, Connection>, RepositoryError> {
        self.conn.lock().map_err(|e| RepositoryError::Storage {
            message: format!("lock poisoned: {e}"),
        })
    }
}
