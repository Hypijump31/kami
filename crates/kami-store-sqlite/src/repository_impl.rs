//! `ToolRepository` trait implementation for `SqliteToolRepository`.

use async_trait::async_trait;
use kami_registry::{RepositoryError, ToolQuery, ToolRepository};
use kami_types::{Tool, ToolId};
use rusqlite::params;

use crate::query_builder::build_find_all_query;
use crate::repository::SqliteToolRepository;
use crate::row_mapping::{row_to_tool, OptionalExt};

/// Column list shared across all SELECT and UPDATE queries.
pub(crate) const COLS: &str = "\
    id, name, version, description, wasm_path, \
    install_path, enabled, security, arguments, wasm_sha256, \
    pinned_version, updated_at, signature, signer_public_key";

/// Maps a `rusqlite::Error` to a `RepositoryError::Storage`.
fn map_sqlite_err(e: rusqlite::Error) -> RepositoryError {
    RepositoryError::Storage {
        message: e.to_string(),
    }
}

#[async_trait]
impl ToolRepository for SqliteToolRepository {
    async fn find_by_id(&self, id: &ToolId) -> Result<Option<Tool>, RepositoryError> {
        let conn = self.lock_conn()?;
        let sql = format!("SELECT {COLS} FROM tools WHERE id = ?1");
        let mut stmt = conn.prepare(&sql).map_err(map_sqlite_err)?;
        let result = stmt
            .query_row(params![id.as_str()], row_to_tool)
            .optional()
            .map_err(map_sqlite_err)?;
        Ok(result)
    }

    async fn find_all(&self, query: ToolQuery) -> Result<Vec<Tool>, RepositoryError> {
        let conn = self.lock_conn()?;
        let (sql, param_values) = build_find_all_query(&query);
        let mut stmt = conn.prepare(&sql).map_err(map_sqlite_err)?;
        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let tools = stmt
            .query_map(params_refs.as_slice(), row_to_tool)
            .map_err(map_sqlite_err)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(map_sqlite_err)?;
        Ok(tools)
    }

    async fn insert(&self, tool: &Tool) -> Result<(), RepositoryError> {
        let conn = self.lock_conn()?;
        let m = &tool.manifest;
        let security_json =
            serde_json::to_string(&m.security).map_err(|e| RepositoryError::Storage {
                message: format!("serialize security: {e}"),
            })?;
        let args_json =
            serde_json::to_string(&m.arguments).map_err(|e| RepositoryError::Storage {
                message: format!("serialize arguments: {e}"),
            })?;
        conn.execute(
            "INSERT INTO tools (id, name, version, description, wasm_path, \
             install_path, enabled, security, arguments, wasm_sha256, \
             signature, signer_public_key) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                m.id.as_str(),
                m.name,
                m.version.to_string(),
                m.description,
                m.wasm,
                tool.install_path,
                tool.enabled as i32,
                security_json,
                args_json,
                m.wasm_sha256,
                m.signature,
                m.signer_public_key,
            ],
        )
        .map_err(|e| {
            if let rusqlite::Error::SqliteFailure(ref err, _) = e {
                if err.code == rusqlite::ErrorCode::ConstraintViolation {
                    return RepositoryError::Conflict {
                        message: format!("tool already exists: {}", m.id),
                    };
                }
            }
            RepositoryError::Storage {
                message: e.to_string(),
            }
        })?;
        Ok(())
    }

    async fn update(&self, tool: &Tool) -> Result<(), RepositoryError> {
        let conn = self.lock_conn()?;
        let m = &tool.manifest;
        let security_json =
            serde_json::to_string(&m.security).map_err(|e| RepositoryError::Storage {
                message: format!("serialize security: {e}"),
            })?;
        let args_json =
            serde_json::to_string(&m.arguments).map_err(|e| RepositoryError::Storage {
                message: format!("serialize arguments: {e}"),
            })?;
        let affected = conn
            .execute(
                "UPDATE tools SET name=?2, version=?3, description=?4, wasm_path=?5, \
                 install_path=?6, enabled=?7, security=?8, arguments=?9, wasm_sha256=?10, \
                 pinned_version=?11, updated_at=?12, signature=?13, \
                 signer_public_key=?14 WHERE id=?1",
                params![
                    m.id.as_str(),
                    m.name,
                    m.version.to_string(),
                    m.description,
                    m.wasm,
                    tool.install_path,
                    tool.enabled as i32,
                    security_json,
                    args_json,
                    m.wasm_sha256,
                    tool.pinned_version,
                    tool.updated_at,
                    m.signature,
                    m.signer_public_key,
                ],
            )
            .map_err(map_sqlite_err)?;
        if affected == 0 {
            return Err(RepositoryError::NotFound {
                id: m.id.to_string(),
            });
        }
        Ok(())
    }

    async fn delete(&self, id: &ToolId) -> Result<bool, RepositoryError> {
        let conn = self.lock_conn()?;
        let affected = conn
            .execute("DELETE FROM tools WHERE id = ?1", params![id.as_str()])
            .map_err(map_sqlite_err)?;
        Ok(affected > 0)
    }
}
