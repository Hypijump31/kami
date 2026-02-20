//! `kami list` command.
//!
//! Lists installed tools from the SQLite registry.

use clap::Args;

use kami_registry::ToolQuery;

use crate::shared;

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
pub async fn execute(args: &ListArgs) -> anyhow::Result<()> {
    let repo = shared::open_repository(&args.db)?;

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

    println!("{:<35} {:<10} {:<8} DESCRIPTION", "ID", "VERSION", "STATUS");
    println!("{}", "-".repeat(80));

    for tool in &tools {
        let status = if tool.enabled { "enabled" } else { "disabled" };
        println!(
            "{:<35} {:<10} {:<8} {}",
            tool.manifest.id, tool.manifest.version, status, tool.manifest.description,
        );
    }

    println!("\n{} tool(s) installed.", tools.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_empty_registry() {
        let dir = tempfile::tempdir().expect("tmp");
        let db = dir.path().join("list.db").to_str().expect("u").to_string();
        let args = ListArgs {
            filter: None,
            db: Some(db),
        };
        assert!(execute(&args).await.is_ok());
    }

    #[tokio::test]
    async fn list_with_name_filter() {
        let dir = tempfile::tempdir().expect("tmp");
        let db = dir.path().join("list2.db").to_str().expect("u").to_string();
        let args = ListArgs {
            filter: Some("echo".into()),
            db: Some(db),
        };
        assert!(execute(&args).await.is_ok());
    }
}
