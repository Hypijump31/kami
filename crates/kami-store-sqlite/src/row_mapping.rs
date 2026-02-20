//! Row-to-domain mapping for the SQLite tool repository.
//!
//! Converts raw SQLite rows into `Tool` domain objects and provides
//! the `OptionalExt` helper for query results.

use kami_types::{SecurityConfig, Tool, ToolArgument, ToolId, ToolManifest, ToolVersion};

/// Maps a SQLite row to a `Tool` domain object.
pub(crate) fn row_to_tool(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tool> {
    let id_str: String = row.get(0)?;
    let name: String = row.get(1)?;
    let version_str: String = row.get(2)?;
    let description: String = row.get(3)?;
    let wasm_path: String = row.get(4)?;
    let install_path: String = row.get(5)?;
    let enabled: bool = row.get(6)?;
    let security_json: String = row.get(7)?;
    let args_json: String = row.get(8)?;
    let wasm_sha256: Option<String> = row.get(9)?;
    let pinned_version: Option<String> = row.get(10)?;
    let updated_at: Option<String> = row.get(11)?;
    let signature: Option<String> = row.get(12)?;
    let signer_public_key: Option<String> = row.get(13)?;

    let id = ToolId::new(id_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let version: ToolVersion = version_str.parse().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e))
    })?;

    // Propagate parse errors rather than silently falling back to defaults.
    // A corrupt JSON column should surface as a repository error, not a
    // silent permission downgrade.
    let security: SecurityConfig = serde_json::from_str(&security_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e))
    })?;

    let arguments: Vec<ToolArgument> = serde_json::from_str(&args_json).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, Box::new(e))
    })?;

    Ok(Tool {
        manifest: ToolManifest {
            id,
            name,
            version,
            wasm: wasm_path,
            description,
            arguments,
            security,
            wasm_sha256,
            signature,
            signer_public_key,
        },
        install_path,
        enabled,
        pinned_version,
        updated_at,
    })
}

/// Extension trait for optional query results.
pub(crate) trait OptionalExt<T> {
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
