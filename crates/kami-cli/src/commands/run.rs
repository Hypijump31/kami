//! `kami run` command.

use std::path::Path;

use clap::Args;

use kami_engine::{
    create_engine, create_linker, load_component_from_file, InstanceConfig,
};
use kami_runtime::WasmToolExecutor;
use kami_types::SecurityConfig;

/// Run a WASM component directly from a file.
#[derive(Debug, Args)]
pub struct RunArgs {
    /// Path to the .wasm component file.
    pub wasm_file: String,
    /// JSON input to pass to the tool's `run` function.
    #[arg(short, long, default_value = "{}")]
    pub input: String,
    /// Fuel limit for execution.
    #[arg(short, long, default_value = "1000000")]
    pub fuel: u64,
}

/// Executes the run command using the async runtime.
pub fn execute(args: &RunArgs) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_async(args))
}

async fn run_async(args: &RunArgs) -> anyhow::Result<()> {
    let wasm_path = Path::new(&args.wasm_file);
    if !wasm_path.exists() {
        anyhow::bail!("WASM file not found: {}", args.wasm_file);
    }

    tracing::info!(path = %args.wasm_file, "Loading WASM component");

    // 1. Create engine and linker
    let config = InstanceConfig::default();
    let engine = create_engine(&config)?;
    let linker = create_linker(&engine)?;

    // 2. Load the component
    let component = load_component_from_file(&engine, wasm_path)?;

    // 3. Execute with deny-all security (default)
    let executor = WasmToolExecutor::new(engine, linker);
    let security = SecurityConfig::default();

    tracing::info!(input = %args.input, fuel = args.fuel, "Executing tool");

    let result = executor
        .execute_component(&component, &args.input, &security, args.fuel)
        .await
        .map_err(|e| anyhow::anyhow!("execution failed: {e}"))?;

    // 4. Output result
    if result.success {
        println!("{}", result.content);
    } else {
        eprintln!("[ERROR] {}", result.content);
    }
    tracing::debug!(
        duration_ms = result.duration_ms,
        success = result.success,
        "Execution complete"
    );

    Ok(())
}
