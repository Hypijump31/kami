//! `kami exec` command.
//!
//! Executes a tool by its registry ID using the full runtime pipeline:
//! resolution, caching, scheduling, and sandboxed execution.

use clap::Args;

use kami_types::{DiagnosticError, ToolId};

use crate::{input, output, shared};

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
pub async fn execute(args: &ExecArgs) -> anyhow::Result<()> {
    let resolved_input = input::resolve_input(&args.input, args.input_file.as_deref())?;

    let repo = shared::open_repository(&args.db)?;
    let tool_id = ToolId::new(&args.tool).map_err(|e| anyhow::anyhow!("invalid tool ID: {e}"))?;

    let runtime = shared::create_runtime(repo, args.concurrency, args.cache_size)?;

    tracing::info!(
        tool = %args.tool,
        input = %resolved_input,
        "Executing tool from registry"
    );

    let result = runtime
        .execute(&tool_id, &resolved_input)
        .await
        .map_err(|e| {
            if let Some(hint) = e.hint() {
                eprintln!("\n  Cause: {hint}");
            }
            if let Some(fix) = e.fix() {
                eprintln!("  Fix:   {fix}\n");
            }
            anyhow::anyhow!("execution failed: {e}")
        })?;

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
