//! `kami install` command.

use clap::Args;

/// Install a WASM tool into the registry.
#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Path to the tool directory or .wasm file.
    pub path: String,
}

/// Executes the install command.
pub fn execute(args: &InstallArgs) -> anyhow::Result<()> {
    crate::output::print_success(&format!("install: {} (not yet implemented)", args.path));
    Ok(())
}
