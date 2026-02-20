//! Database schema migrations for the tool registry.

use kami_registry::RepositoryError;
use rusqlite::Connection;

/// Current schema version.
const SCHEMA_VERSION: u32 = 4;

/// Runs all pending migrations on the database.
pub fn run_migrations(conn: &Connection) -> Result<(), RepositoryError> {
    let current = get_schema_version(conn)?;

    if current < 1 {
        migrate_v1(conn)?;
    }
    if current < 2 {
        migrate_v2(conn)?;
    }
    if current < 3 {
        migrate_v3(conn)?;
    }
    if current < 4 {
        migrate_v4(conn)?;
    }

    set_schema_version(conn, SCHEMA_VERSION)?;
    Ok(())
}

/// Adds the `wasm_sha256` column for WASM integrity verification (v2).
fn migrate_v2(conn: &Connection) -> Result<(), RepositoryError> {
    conn.execute_batch("ALTER TABLE tools ADD COLUMN wasm_sha256 TEXT;")
        .map_err(|e| RepositoryError::Storage {
            message: format!("migration v2 failed: {e}"),
        })
}

/// Adds versioning columns for update & pin support (v3).
fn migrate_v3(conn: &Connection) -> Result<(), RepositoryError> {
    conn.execute_batch(
        "ALTER TABLE tools ADD COLUMN pinned_version TEXT;
         ALTER TABLE tools ADD COLUMN updated_at TEXT;",
    )
    .map_err(|e| RepositoryError::Storage {
        message: format!("migration v3 failed: {e}"),
    })
}

/// Adds cryptographic signature columns for plugin signing (v4).
fn migrate_v4(conn: &Connection) -> Result<(), RepositoryError> {
    conn.execute_batch(
        "ALTER TABLE tools ADD COLUMN signature TEXT;
         ALTER TABLE tools ADD COLUMN signer_public_key TEXT;",
    )
    .map_err(|e| RepositoryError::Storage {
        message: format!("migration v4 failed: {e}"),
    })
}

/// Creates the initial schema (v1).
fn migrate_v1(conn: &Connection) -> Result<(), RepositoryError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS tools (
            id          TEXT PRIMARY KEY NOT NULL,
            name        TEXT NOT NULL,
            version     TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            wasm_path   TEXT NOT NULL,
            install_path TEXT NOT NULL,
            enabled     INTEGER NOT NULL DEFAULT 1,
            security    TEXT NOT NULL DEFAULT '{}',
            arguments   TEXT NOT NULL DEFAULT '[]',
            installed_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_tools_name ON tools(name);
        CREATE INDEX IF NOT EXISTS idx_tools_enabled ON tools(enabled);",
    )
    .map_err(|e| RepositoryError::Storage {
        message: format!("migration v1 failed: {e}"),
    })
}

/// Reads the current schema version from PRAGMA user_version.
fn get_schema_version(conn: &Connection) -> Result<u32, RepositoryError> {
    conn.query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| RepositoryError::Storage {
            message: format!("failed to read schema version: {e}"),
        })
}

/// Sets the schema version via PRAGMA user_version.
fn set_schema_version(conn: &Connection, version: u32) -> Result<(), RepositoryError> {
    conn.pragma_update(None, "user_version", version)
        .map_err(|e| RepositoryError::Storage {
            message: format!("failed to set schema version: {e}"),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_run_on_fresh_db() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        run_migrations(&conn).expect("migrations should succeed");

        let version = get_schema_version(&conn).expect("version");
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn migrations_are_idempotent() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        run_migrations(&conn).expect("first run");
        run_migrations(&conn).expect("second run should also succeed");
    }
}
