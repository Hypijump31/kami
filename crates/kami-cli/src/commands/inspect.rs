//! `kami inspect` command.
//!
//! Displays detailed information about an installed tool.

use clap::Args;

use kami_registry::ToolRepository;
use kami_store_sqlite::SqliteToolRepository;
use kami_types::ToolId;

use crate::output;

/// Inspect a tool's manifest and capabilities.
#[derive(Debug, Args)]
pub struct InspectArgs {
    /// Tool ID (reverse-domain notation).
    pub tool: String,
    /// Database path (defaults to .kami/registry.db).
    #[arg(long)]
    pub db: Option<String>,
}

/// Executes the inspect command.
pub fn execute(args: &InspectArgs) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(inspect_async(args))
}

async fn inspect_async(args: &InspectArgs) -> anyhow::Result<()> {
    let db_path =
        args.db.clone().unwrap_or_else(output::default_db_path);

    let repo = SqliteToolRepository::open(&db_path)
        .map_err(|e| anyhow::anyhow!("registry error: {e}"))?;

    let id = ToolId::new(&args.tool)
        .map_err(|e| anyhow::anyhow!("invalid tool ID: {e}"))?;

    let tool = repo
        .find_by_id(&id)
        .await
        .map_err(|e| anyhow::anyhow!("query error: {e}"))?;

    let tool = match tool {
        Some(t) => t,
        None => {
            output::print_error(&format!(
                "tool not found: {}",
                args.tool
            ));
            return Ok(());
        }
    };

    let m = &tool.manifest;
    let sec = &m.security;

    println!("Tool: {} v{}", m.id, m.version);
    println!("Name: {}", m.name);
    println!("Description: {}", m.description);
    println!("WASM: {}", m.wasm);
    println!("Install path: {}", tool.install_path);
    println!(
        "Status: {}",
        if tool.enabled { "enabled" } else { "disabled" }
    );

    println!("\nSecurity:");
    println!("  Filesystem: {:?}", sec.fs_access);
    if sec.net_allow_list.is_empty() {
        println!("  Network: deny-all");
    } else {
        println!("  Network: {}", sec.net_allow_list.join(", "));
    }
    if sec.env_allow_list.is_empty() {
        println!("  Env vars: deny-all");
    } else {
        println!("  Env vars: {}", sec.env_allow_list.join(", "));
    }

    println!("\nResource limits:");
    println!("  Memory: {} MB", sec.limits.max_memory_mb);
    println!("  Timeout: {} ms", sec.limits.max_execution_ms);
    println!("  Fuel: {}", sec.limits.max_fuel);

    if !m.arguments.is_empty() {
        println!("\nArguments:");
        for arg in &m.arguments {
            let req =
                if arg.required { "required" } else { "optional" };
            println!(
                "  {} ({}, {}): {}",
                arg.name, arg.arg_type, req, arg.description
            );
        }
    }

    Ok(())
}
