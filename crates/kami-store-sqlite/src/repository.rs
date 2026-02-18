//! SQLite implementation of `ToolRepository`.

use std::sync::Mutex;

use async_trait::async_trait;
use kami_registry::{RepositoryError, ToolQuery, ToolRepository};
use kami_types::{
    SecurityConfig, Tool, ToolArgument, ToolId, ToolManifest, ToolVersion,
};
use rusqlite::{params, Connection};

use crate::migrations::run_migrations;

/// SQLite-backed tool repository.
pub struct SqliteToolRepository {
    conn: Mutex<Connection>,
}

impl SqliteToolRepository {
    /// Opens or creates a SQLite database at the given path and runs
    /// migrations.
    pub fn open(path: &str) -> Result<Self, RepositoryError> {
        let conn = Connection::open(path).map_err(|e| {
            RepositoryError::Storage {
                message: e.to_string(),
            }
        })?;
        run_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Opens an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self, RepositoryError> {
        let conn = Connection::open_in_memory().map_err(|e| {
            RepositoryError::Storage {
                message: e.to_string(),
            }
        })?;
        run_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Acquires the connection lock.
    fn lock_conn(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, Connection>, RepositoryError> {
        self.conn.lock().map_err(|e| RepositoryError::Storage {
            message: format!("lock poisoned: {e}"),
        })
    }
}

#[async_trait]
impl ToolRepository for SqliteToolRepository {
    async fn find_by_id(
        &self,
        id: &ToolId,
    ) -> Result<Option<Tool>, RepositoryError> {
        let conn = self.lock_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, version, description, wasm_path,
                        install_path, enabled, security, arguments
                 FROM tools WHERE id = ?1",
            )
            .map_err(|e| RepositoryError::Storage {
                message: e.to_string(),
            })?;

        let result = stmt
            .query_row(params![id.as_str()], row_to_tool)
            .optional()
            .map_err(|e| RepositoryError::Storage {
                message: e.to_string(),
            })?;

        Ok(result)
    }

    async fn find_all(
        &self,
        query: ToolQuery,
    ) -> Result<Vec<Tool>, RepositoryError> {
        let conn = self.lock_conn()?;

        let mut sql = String::from(
            "SELECT id, name, version, description, wasm_path,
                    install_path, enabled, security, arguments
             FROM tools WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> =
            Vec::new();

        if query.enabled_only {
            sql.push_str(" AND enabled = 1");
        }
        if let Some(ref name) = query.name_filter {
            sql.push_str(" AND name LIKE ?");
            param_values.push(Box::new(format!("%{name}%")));
        }

        sql.push_str(" ORDER BY name ASC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }
        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {offset}"));
        }

        let mut stmt = conn.prepare(&sql).map_err(|e| {
            RepositoryError::Storage {
                message: e.to_string(),
            }
        })?;

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let tools = stmt
            .query_map(params_refs.as_slice(), row_to_tool)
            .map_err(|e| RepositoryError::Storage {
                message: e.to_string(),
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| RepositoryError::Storage {
                message: e.to_string(),
            })?;

        Ok(tools)
    }

    async fn insert(
        &self,
        tool: &Tool,
    ) -> Result<(), RepositoryError> {
        let conn = self.lock_conn()?;
        let m = &tool.manifest;

        let security_json =
            serde_json::to_string(&m.security).map_err(|e| {
                RepositoryError::Storage {
                    message: format!("serialize security: {e}"),
                }
            })?;
        let args_json =
            serde_json::to_string(&m.arguments).map_err(|e| {
                RepositoryError::Storage {
                    message: format!("serialize arguments: {e}"),
                }
            })?;

        conn.execute(
            "INSERT INTO tools (id, name, version, description, wasm_path,
                               install_path, enabled, security, arguments)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
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

    async fn delete(
        &self,
        id: &ToolId,
    ) -> Result<bool, RepositoryError> {
        let conn = self.lock_conn()?;
        let affected = conn
            .execute(
                "DELETE FROM tools WHERE id = ?1",
                params![id.as_str()],
            )
            .map_err(|e| RepositoryError::Storage {
                message: e.to_string(),
            })?;
        Ok(affected > 0)
    }
}

/// Maps a SQLite row to a `Tool` domain object.
fn row_to_tool(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tool> {
    let id_str: String = row.get(0)?;
    let name: String = row.get(1)?;
    let version_str: String = row.get(2)?;
    let description: String = row.get(3)?;
    let wasm_path: String = row.get(4)?;
    let install_path: String = row.get(5)?;
    let enabled: bool = row.get(6)?;
    let security_json: String = row.get(7)?;
    let args_json: String = row.get(8)?;

    let id = ToolId::new(id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        )
    })?;

    let version: ToolVersion = version_str.parse().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            Box::new(e),
        )
    })?;

    let security: SecurityConfig =
        serde_json::from_str(&security_json).unwrap_or_default();
    let arguments: Vec<ToolArgument> =
        serde_json::from_str(&args_json).unwrap_or_default();

    Ok(Tool {
        manifest: ToolManifest {
            id,
            name,
            version,
            wasm: wasm_path,
            description,
            arguments,
            security,
        },
        install_path,
        enabled,
    })
}

/// Extension trait for optional query results.
trait OptionalExt<T> {
    /// Converts a "no rows" error into `Ok(None)`.
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tool() -> Tool {
        Tool {
            manifest: ToolManifest {
                id: ToolId::new("dev.test.sample").expect("id"),
                name: "sample".to_string(),
                version: ToolVersion::new(1, 0, 0),
                wasm: "sample.wasm".to_string(),
                description: "A sample tool".to_string(),
                arguments: vec![],
                security: SecurityConfig::default(),
            },
            install_path: "/tools/sample".to_string(),
            enabled: true,
        }
    }

    #[tokio::test]
    async fn insert_and_find_by_id() {
        let repo =
            SqliteToolRepository::open_in_memory().expect("open");
        let tool = sample_tool();

        repo.insert(&tool).await.expect("insert");

        let found = repo
            .find_by_id(&tool.manifest.id)
            .await
            .expect("find");

        let found = found.expect("should exist");
        assert_eq!(found.manifest.id.as_str(), "dev.test.sample");
        assert_eq!(found.manifest.name, "sample");
        assert!(found.enabled);
    }

    #[tokio::test]
    async fn find_nonexistent_returns_none() {
        let repo =
            SqliteToolRepository::open_in_memory().expect("open");
        let id = ToolId::new("dev.test.nope").expect("id");
        let found = repo.find_by_id(&id).await.expect("find");
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn insert_duplicate_fails() {
        let repo =
            SqliteToolRepository::open_in_memory().expect("open");
        let tool = sample_tool();

        repo.insert(&tool).await.expect("first insert");
        let err = repo.insert(&tool).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn delete_existing_tool() {
        let repo =
            SqliteToolRepository::open_in_memory().expect("open");
        let tool = sample_tool();

        repo.insert(&tool).await.expect("insert");
        let deleted =
            repo.delete(&tool.manifest.id).await.expect("delete");
        assert!(deleted);

        let found = repo
            .find_by_id(&tool.manifest.id)
            .await
            .expect("find");
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn delete_nonexistent_returns_false() {
        let repo =
            SqliteToolRepository::open_in_memory().expect("open");
        let id = ToolId::new("dev.test.nope").expect("id");
        let deleted = repo.delete(&id).await.expect("delete");
        assert!(!deleted);
    }

    #[tokio::test]
    async fn find_all_returns_all() {
        let repo =
            SqliteToolRepository::open_in_memory().expect("open");

        let tool1 = Tool {
            manifest: ToolManifest {
                id: ToolId::new("dev.test.alpha").expect("id"),
                name: "alpha".to_string(),
                version: ToolVersion::new(1, 0, 0),
                wasm: "alpha.wasm".to_string(),
                description: "Alpha".to_string(),
                arguments: vec![],
                security: SecurityConfig::default(),
            },
            install_path: "/tools/alpha".to_string(),
            enabled: true,
        };
        let tool2 = Tool {
            manifest: ToolManifest {
                id: ToolId::new("dev.test.beta").expect("id"),
                name: "beta".to_string(),
                version: ToolVersion::new(2, 0, 0),
                wasm: "beta.wasm".to_string(),
                description: "Beta".to_string(),
                arguments: vec![],
                security: SecurityConfig::default(),
            },
            install_path: "/tools/beta".to_string(),
            enabled: true,
        };

        repo.insert(&tool1).await.expect("insert 1");
        repo.insert(&tool2).await.expect("insert 2");

        let all =
            repo.find_all(ToolQuery::all()).await.expect("find_all");
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn find_all_with_name_filter() {
        let repo =
            SqliteToolRepository::open_in_memory().expect("open");

        let tool1 = Tool {
            manifest: ToolManifest {
                id: ToolId::new("dev.test.fetch-url").expect("id"),
                name: "fetch-url".to_string(),
                version: ToolVersion::new(1, 0, 0),
                wasm: "f.wasm".to_string(),
                description: "Fetch".to_string(),
                arguments: vec![],
                security: SecurityConfig::default(),
            },
            install_path: "/tools/fetch".to_string(),
            enabled: true,
        };
        let tool2 = Tool {
            manifest: ToolManifest {
                id: ToolId::new("dev.test.calc").expect("id"),
                name: "calc".to_string(),
                version: ToolVersion::new(1, 0, 0),
                wasm: "c.wasm".to_string(),
                description: "Calculator".to_string(),
                arguments: vec![],
                security: SecurityConfig::default(),
            },
            install_path: "/tools/calc".to_string(),
            enabled: true,
        };

        repo.insert(&tool1).await.expect("insert");
        repo.insert(&tool2).await.expect("insert");

        let results = repo
            .find_all(ToolQuery::all().with_name("fetch"))
            .await
            .expect("find");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].manifest.name, "fetch-url");
    }
}
