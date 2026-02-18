//! `kami uninstall` command.
//!
//! Removes a tool from the registry by its ID.

use clap::Args;

use kami_registry::ToolRepository;
use kami_store_sqlite::SqliteToolRepository;
use kami_types::ToolId;

use crate::output;

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
pub fn execute(args: &UninstallArgs) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(uninstall_async(args))
}

async fn uninstall_async(args: &UninstallArgs) -> anyhow::Result<()> {
    let tool_id = ToolId::new(&args.tool)
        .map_err(|e| anyhow::anyhow!("invalid tool ID: {e}"))?;

    let db_path =
        args.db.clone().unwrap_or_else(output::default_db_path);

    let repo = SqliteToolRepository::open(&db_path)
        .map_err(|e| anyhow::anyhow!("registry error: {e}"))?;

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
