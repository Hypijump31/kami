//! `kami uninstall` command.
//!
//! Removes a tool from the registry by its ID.

use clap::Args;

use kami_types::ToolId;

use crate::{output, shared};

/// Uninstall a tool from the registry.
#[derive(Debug, Args)]
pub struct UninstallArgs {
    /// Tool ID to remove (e.g. dev.example.fetch-url).
    pub tool: String,
    /// Database path (defaults to .kami/registry.db).
    #[arg(long)]
    pub db: Option<String>,
}

/// Executes the uninstall command.
pub async fn execute(args: &UninstallArgs) -> anyhow::Result<()> {
    let tool_id = ToolId::new(&args.tool).map_err(|e| anyhow::anyhow!("invalid tool ID: {e}"))?;

    let repo = shared::open_repository(&args.db)?;

    // Check if tool exists before deleting
    let existing = repo
        .find_by_id(&tool_id)
        .await
        .map_err(|e| anyhow::anyhow!("registry error: {e}"))?;

    match existing {
        Some(tool) => {
            repo.delete(&tool_id)
                .await
                .map_err(|e| anyhow::anyhow!("delete error: {e}"))?;

            output::print_success(&format!(
                "Uninstalled {} v{}",
                tool.manifest.id, tool.manifest.version
            ));
        }
        None => {
            anyhow::bail!("tool not found: {}", args.tool);
        }
    }

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn uninstall_missing_tool_fails() {
        let dir = tempfile::tempdir().expect("tmp");
        let db = dir.path().join("uni.db").to_str().expect("u").to_string();
        let args = UninstallArgs {
            tool: "dev.test.nope".into(),
            db: Some(db),
        };
        assert!(execute(&args).await.is_err());
    }

    #[tokio::test]
    async fn uninstall_invalid_id_fails() {
        let dir = tempfile::tempdir().expect("tmp");
        let db = dir.path().join("uni2.db").to_str().expect("u").to_string();
        let args = UninstallArgs {
            tool: "bad".into(),
            db: Some(db),
        };
        assert!(execute(&args).await.is_err());
    }

    #[tokio::test]
    async fn uninstall_existing_tool_succeeds() {
        use kami_registry::ToolRepository;
        use kami_store_sqlite::SqliteToolRepository;
        use kami_types::{SecurityConfig, Tool, ToolId, ToolManifest, ToolVersion};
        let dir = tempfile::tempdir().expect("tmp");
        let db = dir.path().join("uni3.db").to_str().expect("u").to_string();
        {
            let repo = SqliteToolRepository::open(&db).expect("open");
            repo.insert(&Tool {
                manifest: ToolManifest {
                    id: ToolId::new("dev.test.del").expect("id"),
                    name: "del".into(),
                    version: ToolVersion::new(1, 0, 0),
                    wasm: "del.wasm".into(),
                    description: "d".into(),
                    arguments: vec![],
                    security: SecurityConfig::default(),
                    wasm_sha256: None,
                    signature: None,
                    signer_public_key: None,
                },
                install_path: "/t".into(),
                enabled: true,
                pinned_version: None,
                updated_at: None,
            })
            .await
            .expect("insert");
        }
        let args = UninstallArgs {
            tool: "dev.test.del".into(),
            db: Some(db),
        };
        assert!(execute(&args).await.is_ok());
    }
}
