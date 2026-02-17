//! KAMI CLI - Orchestrateur WASM/MCP.

use clap::{Parser, Subcommand};

mod commands;
#[allow(dead_code)]
mod output;

/// KAMI - Secure WASM tool orchestrator for AI agents.
#[derive(Debug, Parser)]
#[command(name = "kami", version, about)]
struct Cli {
    /// Configuration file path.
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Verbosity level (-v, -vv, -vvv).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Install a WASM tool.
    Install(commands::install::InstallArgs),
    /// Run a WASM component file.
    Run(commands::run::RunArgs),
    /// List installed tools.
    List(commands::list::ListArgs),
    /// Inspect a tool's manifest.
    Inspect(commands::inspect::InspectArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing.
    let filter = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    tracing::debug!("KAMI starting with config: {:?}", cli.config);

    match &cli.command {
        Commands::Install(args) => commands::install::execute(args),
        Commands::Run(args) => commands::run::execute(args),
        Commands::List(args) => commands::list::execute(args),
        Commands::Inspect(args) => commands::inspect::execute(args),
    }
}
