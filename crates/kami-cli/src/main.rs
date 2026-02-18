//! KAMI CLI - Orchestrateur WASM/MCP.

use clap::{Parser, Subcommand};

mod commands;
mod input;
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
    /// Scaffold a new tool project.
    Init(commands::init::InitArgs),
    /// Validate a tool.toml manifest without installing.
    Validate(commands::validate::ValidateArgs),
    /// Install a WASM tool into the registry.
    Install(commands::install::InstallArgs),
    /// Uninstall a tool from the registry.
    Uninstall(commands::uninstall::UninstallArgs),
    /// List installed tools.
    List(commands::list::ListArgs),
    /// Inspect a tool's manifest.
    Inspect(commands::inspect::InspectArgs),
    /// Run a WASM component file directly (dev mode).
    Run(commands::run::RunArgs),
    /// Execute a registered tool by ID.
    Exec(commands::exec::ExecArgs),
    /// Start MCP server over stdio.
    Serve(commands::serve::ServeArgs),
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
        Commands::Init(args) => commands::init::execute(args),
        Commands::Validate(args) => commands::validate::execute(args),
        Commands::Install(args) => commands::install::execute(args),
        Commands::Uninstall(args) => commands::uninstall::execute(args),
        Commands::List(args) => commands::list::execute(args),
        Commands::Inspect(args) => commands::inspect::execute(args),
        Commands::Run(args) => commands::run::execute(args),
        Commands::Exec(args) => commands::exec::execute(args),
        Commands::Serve(args) => commands::serve::execute(args),
    }
}
