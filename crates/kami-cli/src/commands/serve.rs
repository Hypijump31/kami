//! `kami serve` command.
//!
//! Starts the MCP server over stdio, exposing all registered tools
//! via the JSON-RPC protocol.

use std::sync::Arc;

use clap::Args;

use kami_runtime::{KamiRuntime, RuntimeConfig};
use kami_store_sqlite::SqliteToolRepository;
use kami_transport_stdio::{McpHandler, McpServer, StdioTransport};

use crate::output;

/// Start the MCP server over stdio.
#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Maximum concurrent tool executions.
    #[arg(long, default_value = "4")]
    pub concurrency: usize,
    /// Component cache size.
    #[arg(long, default_value = "32")]
    pub cache_size: usize,
    /// Database path (defaults to .kami/registry.db).
    #[arg(long)]
    pub db: Option<String>,
}

/// Executes the serve command.
pub fn execute(args: &ServeArgs) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(serve_async(args))
}

async fn serve_async(args: &ServeArgs) -> anyhow::Result<()> {
    let db_path =
        args.db.clone().unwrap_or_else(output::default_db_path);

    let repo = SqliteToolRepository::open(&db_path)
        .map_err(|e| anyhow::anyhow!("registry error: {e}"))?;
    let repo = Arc::new(repo);

    let config = RuntimeConfig {
        cache_size: args.cache_size,
        max_concurrent: args.concurrency,
        epoch_interruption: true,
    };

    let runtime = Arc::new(
        KamiRuntime::new(config, repo.clone())
            .map_err(|e| anyhow::anyhow!("runtime init error: {e}"))?,
    );

    let handler = McpHandler::new(runtime, repo);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let transport = StdioTransport::new(stdin, stdout);

    let mut server = McpServer::new(transport, handler);

    tracing::info!("KAMI MCP server ready on stdio");

    server
        .run()
        .await
        .map_err(|e| anyhow::anyhow!("server error: {e}"))?;

    Ok(())
}
