//! `kami inspect` command.

use clap::Args;

/// Inspect a tool's manifest and capabilities.
#[derive(Debug, Args)]
pub struct InspectArgs {
    /// Tool name or ID.
    pub tool: String,
}

/// Executes the inspect command.
pub fn execute(_args: &InspectArgs) -> anyhow::Result<()> {
    tracing::info!("inspect command not yet implemented");
    Ok(())
}
