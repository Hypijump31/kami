//! Async tool executor.

use std::time::Instant;

use async_trait::async_trait;
use wasmtime::component::{Component, Linker};
use wasmtime::Engine;

use kami_engine::{
    call_tool_run, create_store, instantiate_component, HostState,
};
use kami_sandbox::{build_wasi_ctx, WasiConfig};
use kami_types::SecurityConfig;

use crate::context::ExecutionContext;
use crate::error::RuntimeError;

/// Result of a tool execution.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Output content from the tool.
    pub content: String,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Whether execution succeeded.
    pub success: bool,
}

/// Trait for executing tools asynchronously.
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Executes a tool with the given context.
    async fn execute(&self, ctx: ExecutionContext) -> Result<ExecutionResult, RuntimeError>;
}

/// Concrete executor that runs WASM components through the
/// engine + sandbox pipeline.
pub struct WasmToolExecutor {
    engine: Engine,
    linker: Linker<HostState>,
}

impl WasmToolExecutor {
    /// Creates a new executor with a pre-configured engine and linker.
    pub fn new(engine: Engine, linker: Linker<HostState>) -> Self {
        Self { engine, linker }
    }

    /// Executes a pre-compiled component with the given security config.
    pub async fn execute_component(
        &self,
        component: &Component,
        input: &str,
        security: &SecurityConfig,
        fuel: u64,
    ) -> Result<ExecutionResult, RuntimeError> {
        let start = Instant::now();

        // 1. Build sandboxed WASI context
        let wasi_config = WasiConfig {
            inherit_stdout: true,
            inherit_stderr: true,
            ..WasiConfig::default()
        };
        let wasi_ctx = build_wasi_ctx(security, &wasi_config, None)?;

        // 2. Create store with host state and fuel
        let host_state = HostState::new(wasi_ctx);
        let mut store = create_store(&self.engine, host_state, fuel)?;

        // 3. Instantiate the component
        let instance =
            instantiate_component(&self.linker, &mut store, component).await?;

        // 4. Call the `run` export
        let result = call_tool_run(&mut store, &instance, input).await?;

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(output) => Ok(ExecutionResult {
                content: output,
                duration_ms,
                success: true,
            }),
            Err(error) => Ok(ExecutionResult {
                content: error,
                duration_ms,
                success: false,
            }),
        }
    }

    /// Returns a reference to the engine.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}
