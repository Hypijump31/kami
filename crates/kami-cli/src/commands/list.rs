//! `kami list` command.

use clap::Args;

/// List installed tools.
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Filter by name.
    #[arg(short, long)]
    pub filter: Option<String>,
}

/// Executes the list command.
pub fn execute(_args: &ListArgs) -> anyhow::Result<()> {
    tracing::info!("list command not yet implemented");
    Ok(())
}
