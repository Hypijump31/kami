//! `kami list` command.
//!
//! Lists installed tools from the SQLite registry.

use clap::Args;

use kami_registry::{ToolQuery, ToolRepository};
use kami_store_sqlite::SqliteToolRepository;

use crate::output;

/// List installed tools.
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Filter by name (substring match).
    #[arg(short, long)]
    pub filter: Option<String>,
    /// Database path (defaults to .kami/registry.db).
    #[arg(long)]
    pub db: Option<String>,
}

/// Executes the list command.
pub fn execute(args: &ListArgs) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(list_async(args))
}

async fn list_async(args: &ListArgs) -> anyhow::Result<()> {
    let db_path =
        args.db.clone().unwrap_or_else(output::default_db_path);

    let repo = SqliteToolRepository::open(&db_path)
        .map_err(|e| anyhow::anyhow!("registry error: {e}"))?;

    let mut query = ToolQuery::all();
    if let Some(ref name) = args.filter {
        query = query.with_name(name);
    }

    let tools = repo
        .find_all(query)
        .await
        .map_err(|e| anyhow::anyhow!("query error: {e}"))?;

    if tools.is_empty() {
        println!("No tools installed.");
        return Ok(());
    }

    println!(
        "{:<35} {:<10} {:<8} DESCRIPTION",
        "ID", "VERSION", "STATUS"
    );
    println!("{}", "-".repeat(80));

    for tool in &tools {
        let status = if tool.enabled { "enabled" } else { "disabled" };
        println!(
            "{:<35} {:<10} {:<8} {}",
            tool.manifest.id,
            tool.manifest.version,
            status,
            tool.manifest.description,
        );
    }

    println!("\n{} tool(s) installed.", tools.len());

    Ok(())
}
