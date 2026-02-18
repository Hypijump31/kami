//! `kami exec` command.
//!
//! Executes a tool by its registry ID using the full runtime pipeline:
//! resolution, caching, scheduling, and sandboxed execution.

use std::sync::Arc;

use clap::Args;

use kami_runtime::{KamiRuntime, RuntimeConfig};
use kami_store_sqlite::SqliteToolRepository;
use kami_types::ToolId;

use crate::input;
use crate::output;

/// Execute a registered tool by its ID.
#[derive(Debug, Args)]
pub struct ExecArgs {
    /// Tool ID (reverse-domain notation, e.g. dev.example.fetch-url).
    pub tool: String,
    /// JSON input string to pass to the tool's `run` function.
    #[arg(short, long, default_value = "{}")]
    pub input: String,
    /// Read JSON input from a file (use "-" for stdin).
    #[arg(short = 'f', long)]
    pub input_file: Option<String>,
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

/// Executes the exec command.
pub fn execute(args: &ExecArgs) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(exec_async(args))
}

async fn exec_async(args: &ExecArgs) -> anyhow::Result<()> {
    let resolved_input =
        input::resolve_input(&args.input, args.input_file.as_deref())?;

    let db_path =
        args.db.clone().unwrap_or_else(output::default_db_path);

    let repo = SqliteToolRepository::open(&db_path)
        .map_err(|e| anyhow::anyhow!("registry error: {e}"))?;

    let tool_id = ToolId::new(&args.tool)
        .map_err(|e| anyhow::anyhow!("invalid tool ID: {e}"))?;

    let config = RuntimeConfig {
        cache_size: args.cache_size,
        max_concurrent: args.concurrency,
        epoch_interruption: true,
    };

    let runtime = KamiRuntime::new(config, Arc::new(repo))
        .map_err(|e| anyhow::anyhow!("runtime init error: {e}"))?;

    tracing::info!(
        tool = %args.tool,
        input = %resolved_input,
        "Executing tool from registry"
    );

    let result = runtime
        .execute(&tool_id, &resolved_input)
        .await
        .map_err(|e| anyhow::anyhow!("execution failed: {e}"))?;

    if result.success {
        println!("{}", result.content);
    } else {
        output::print_error(&result.content);
    }

    tracing::debug!(
        duration_ms = result.duration_ms,
        fuel_consumed = result.fuel_consumed,
        success = result.success,
        "Execution complete"
    );

    Ok(())
}
