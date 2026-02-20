//! `kami run` command.

use std::path::Path;

use clap::Args;

use kami_engine::{create_engine, create_linker, load_component_from_file, InstanceConfig};
use kami_runtime::{ToolExecutor, WasmToolExecutor};
use kami_types::{ResourceLimits, SecurityConfig};

use crate::input;

/// Run a WASM component directly from a file.
#[derive(Debug, Args)]
pub struct RunArgs {
    /// Path to the .wasm component file.
    pub wasm_file: String,
    /// JSON input string to pass to the tool's `run` function.
    #[arg(short, long, default_value = "{}")]
    pub input: String,
    /// Read JSON input from a file (use "-" for stdin).
    #[arg(short = 'f', long)]
    pub input_file: Option<String>,
    /// Fuel limit for execution.
    #[arg(short, long, default_value = "1000000")]
    pub fuel: u64,
    /// Maximum memory in MB.
    #[arg(short, long, default_value = "64")]
    pub max_memory_mb: u32,
    /// Execution timeout in milliseconds.
    #[arg(short, long, default_value = "5000")]
    pub timeout_ms: u64,
}

/// Executes the run command using the async runtime.
pub async fn execute(args: &RunArgs) -> anyhow::Result<()> {
    let wasm_path = Path::new(&args.wasm_file);
    if !wasm_path.exists() {
        anyhow::bail!("WASM file not found: {}", args.wasm_file);
    }

    let resolved_input = input::resolve_input(&args.input, args.input_file.as_deref())?;

    tracing::info!(path = %args.wasm_file, "Loading WASM component");

    // 1. Create engine and linker (epoch interruption for timeout)
    let config = InstanceConfig {
        epoch_interruption: true,
        ..InstanceConfig::default()
    };
    let engine = create_engine(&config)?;
    let linker = create_linker(&engine)?;

    // 2. Load the component
    let component = load_component_from_file(&engine, wasm_path)?;

    // 3. Build security config from CLI args
    let security = SecurityConfig {
        limits: ResourceLimits {
            max_fuel: args.fuel,
            max_memory_mb: args.max_memory_mb,
            max_execution_ms: args.timeout_ms,
        },
        ..SecurityConfig::default()
    };

    // 4. Execute with full isolation
    let executor = WasmToolExecutor::new(engine, linker);

    tracing::info!(
        input = %resolved_input,
        fuel = args.fuel,
        max_memory_mb = args.max_memory_mb,
        timeout_ms = args.timeout_ms,
        "Executing tool"
    );

    let result = executor
        .execute(&component, &resolved_input, &security)
        .await
        .map_err(|e| anyhow::anyhow!("execution failed: {e}"))?;

    // 5. Output result
    if result.success {
        println!("{}", result.content);
    } else {
        eprintln!("[ERROR] {}", result.content);
    }
    tracing::debug!(
        duration_ms = result.duration_ms,
        fuel_consumed = result.fuel_consumed,
        success = result.success,
        "Execution complete"
    );

    Ok(())
}
