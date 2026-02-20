//! `kami status` command.
//!
//! Displays tool registry statistics and runtime configuration.

use clap::Args;

use kami_registry::ToolQuery;

use crate::output;
use crate::shared;

/// Show KAMI runtime status and installed tool statistics.
#[derive(Debug, Args)]
pub struct StatusArgs {
    /// Database path (defaults to .kami/registry.db).
    #[arg(long)]
    pub db: Option<String>,
    /// Maximum concurrent tool executions (for display).
    #[arg(long, default_value = "4")]
    pub concurrency: usize,
    /// Component cache size in slots (for display).
    #[arg(long, default_value = "32")]
    pub cache_size: usize,
}

/// Executes the status command.
pub async fn execute(args: &StatusArgs) -> anyhow::Result<()> {
    let repo = shared::open_repository(&args.db)?;
    let tools = repo
        .find_all(ToolQuery::all())
        .await
        .map_err(|e| anyhow::anyhow!("registry query: {e}"))?;

    let enabled = tools.iter().filter(|t| t.enabled).count();
    let disabled = tools.len() - enabled;
    let db_path = args.db.clone().unwrap_or_else(output::default_db_path);

    println!("KAMI Runtime Status");
    println!("{}", "─".repeat(40));
    println!("  Version    : {}", env!("CARGO_PKG_VERSION"));
    println!("  Database   : {db_path}");
    println!();
    println!("Tool Registry");
    println!("  Total    : {}", tools.len());
    println!("  Enabled  : {enabled}");
    println!("  Disabled : {disabled}");
    println!();
    println!("Runtime Configuration");
    println!(
        "  Concurrency : {} max parallel executions",
        args.concurrency
    );
    println!("  Cache       : {} component slots", args.cache_size);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_args_defaults() {
        let args = StatusArgs {
            db: None,
            concurrency: 4,
            cache_size: 32,
        };
        assert_eq!(args.concurrency, 4);
        assert_eq!(args.cache_size, 32);
        assert!(args.db.is_none());
    }

    #[tokio::test]
    async fn status_empty_registry() {
        let dir = tempfile::tempdir().expect("tmp");
        let db = dir.path().join("st.db").to_str().expect("u").to_string();
        let args = StatusArgs {
            db: Some(db),
            concurrency: 2,
            cache_size: 8,
        };
        assert!(execute(&args).await.is_ok());
    }
}
