//! `kami serve` command.
//!
//! Starts the MCP server over stdio or HTTP, exposing registered tools
//! via JSON-RPC 2.0.

use std::sync::Arc;

use clap::Args;

use kami_transport_http::HttpServer;
use kami_transport_stdio::{McpHandler, McpServer, StdioTransport};

use crate::shared;

/// Start the MCP server (stdio or HTTP).
#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Transport mode: stdio (default) or http.
    #[arg(long, default_value = "stdio", value_parser = ["stdio", "http"])]
    pub transport: String,
    /// TCP port for HTTP transport.
    #[arg(long, default_value = "3000")]
    pub port: u16,
    /// Bearer token for HTTP transport authentication (optional).
    #[arg(long)]
    pub token: Option<String>,
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
pub async fn execute(args: &ServeArgs) -> anyhow::Result<()> {
    let repo = shared::open_repository(&args.db)?;
    let runtime = Arc::new(shared::create_runtime(
        repo.clone(),
        args.concurrency,
        args.cache_size,
    )?);

    match args.transport.as_str() {
        "http" => {
            let handler = Arc::new(McpHandler::new(runtime.clone(), repo));
            let server = HttpServer::new(handler, args.port, args.token.clone());
            tokio::select! {
                result = server.run() => {
                    result.map_err(|e| anyhow::anyhow!("server error: {e}"))?;
                }
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("shutdown signal received");
                    runtime.shutdown().await;
                }
            }
        }
        _ => {
            let handler = McpHandler::new(runtime.clone(), repo);
            let transport = StdioTransport::new(tokio::io::stdin(), tokio::io::stdout());
            let mut server = McpServer::new(transport, handler);
            tracing::info!("KAMI MCP server ready on stdio");
            tokio::select! {
                result = server.run() => {
                    result.map_err(|e| anyhow::anyhow!("server error: {e}"))?;
                }
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("shutdown signal received");
                    runtime.shutdown().await;
                }
            }
        }
    }

    Ok(())
}
