//! Async tool executor with full isolation pipeline.
//!
//! Pipeline: validate config → build sandbox → apply limits → execute with timeout.

use std::time::{Duration, Instant};

use async_trait::async_trait;
use tracing::{debug, warn};
use wasmtime::component::{Component, Linker};
use wasmtime::Engine;

use kami_engine::{
    call_tool_run, create_store, instantiate_component, set_epoch_deadline,
    HostState,
};
use kami_sandbox::{build_wasi_ctx, validate_security_config, WasiConfig};
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
    /// Fuel consumed during execution.
    pub fuel_consumed: u64,
}

/// Trait for executing tools asynchronously.
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Executes a tool with the given context.
    async fn execute(
        &self,
        ctx: ExecutionContext,
    ) -> Result<ExecutionResult, RuntimeError>;
}

/// Concrete executor that runs WASM components through the
/// engine + sandbox pipeline with full isolation enforcement.
pub struct WasmToolExecutor {
    engine: Engine,
    linker: Linker<HostState>,
}

impl WasmToolExecutor {
    /// Creates a new executor with a pre-configured engine and linker.
    pub fn new(engine: Engine, linker: Linker<HostState>) -> Self {
        Self { engine, linker }
    }

    /// Executes a component with full isolation pipeline.
    ///
    /// Pipeline steps:
    /// 1. Validate security config
    /// 2. Build sandboxed WASI context
    /// 3. Create store with memory limits + fuel
    /// 4. Set epoch deadline for timeout
    /// 5. Spawn epoch ticker task
    /// 6. Instantiate and call with tokio::time::timeout
    pub async fn execute_component(
        &self,
        component: &Component,
        input: &str,
        security: &SecurityConfig,
    ) -> Result<ExecutionResult, RuntimeError> {
        let start = Instant::now();

        // 1. Validate security config
        validate_security_config(security)?;

        let fuel = security.limits.max_fuel;
        let max_memory =
            security.limits.max_memory_mb as usize * 1024 * 1024;
        let timeout_duration =
            Duration::from_millis(security.limits.max_execution_ms);

        debug!(
            fuel,
            max_memory_mb = security.limits.max_memory_mb,
            timeout_ms = security.limits.max_execution_ms,
            "starting isolated execution"
        );

        // 2. Build sandboxed WASI context
        let wasi_config = WasiConfig {
            inherit_stdout: true,
            inherit_stderr: true,
            ..WasiConfig::default()
        };
        let wasi_ctx = build_wasi_ctx(security, &wasi_config, None)?;

        // 3. Create store with memory limits + fuel
        let host_state = HostState::with_limits(wasi_ctx, max_memory);
        let mut store = create_store(&self.engine, host_state, fuel)?;

        // 4. Set epoch deadline (1 tick = timeout reached)
        set_epoch_deadline(&mut store, 1);

        // 5. Spawn epoch ticker that increments epoch after timeout
        let engine_clone = self.engine.clone();
        let tick_handle = tokio::spawn(async move {
            tokio::time::sleep(timeout_duration).await;
            engine_clone.increment_epoch();
        });

        // 6. Instantiate and call (store is borrowed, not moved)
        let outer_timeout = timeout_duration + Duration::from_millis(500);
        let call_result = tokio::time::timeout(outer_timeout, async {
            let instance =
                instantiate_component(&self.linker, &mut store, component)
                    .await?;
            call_tool_run(&mut store, &instance, input).await
        })
        .await;

        // Cancel the epoch ticker
        tick_handle.abort();

        let duration_ms = start.elapsed().as_millis() as u64;

        // Compute fuel consumed
        let fuel_remaining = store.get_fuel().unwrap_or(0);
        let fuel_consumed = fuel.saturating_sub(fuel_remaining);

        match call_result {
            Ok(Ok(Ok(output))) => Ok(ExecutionResult {
                content: output,
                duration_ms,
                success: true,
                fuel_consumed,
            }),
            Ok(Ok(Err(error))) => Ok(ExecutionResult {
                content: error,
                duration_ms,
                success: false,
                fuel_consumed,
            }),
            Ok(Err(engine_err)) => {
                warn!(?engine_err, "engine error during execution");
                Err(engine_err.into())
            }
            Err(_elapsed) => {
                warn!(
                    timeout_ms = security.limits.max_execution_ms,
                    "execution timed out"
                );
                Err(RuntimeError::Timeout {
                    timeout_ms: security.limits.max_execution_ms,
                })
            }
        }
    }

    /// Returns a reference to the engine.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}
