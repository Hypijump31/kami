//! Async tool executor with full isolation pipeline.
//!
//! Pipeline: validate config → build sandbox → apply limits → execute with timeout.

use std::time::{Duration, Instant};

use async_trait::async_trait;
use tracing::{debug, warn};
use wasmtime::component::{Component, Linker};
use wasmtime::Engine;

use kami_engine::{
    call_tool_run, create_store, instantiate_component, instantiate_tool, set_epoch_deadline,
    HostState,
};
use kami_sandbox::{build_wasi_ctx, validate_security_config, WasiConfig};
use kami_types::SecurityConfig;

use crate::error::RuntimeError;
use crate::types::{ExecutionResult, ToolExecutor};

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
}

#[async_trait]
impl ToolExecutor for WasmToolExecutor {
    /// Executes a component with full isolation pipeline.
    ///
    /// # Errors
    ///
    /// Returns `RuntimeError::Sandbox` if security config is invalid.
    /// Returns `RuntimeError::Engine` if the component fails to execute.
    /// Returns `RuntimeError::Timeout` if execution exceeds the deadline.
    #[tracing::instrument(skip_all, fields(
        max_fuel = security.limits.max_fuel,
        timeout_ms = security.limits.max_execution_ms,
    ))]
    async fn execute(
        &self,
        component: &Component,
        input: &str,
        security: &SecurityConfig,
    ) -> Result<ExecutionResult, RuntimeError> {
        let start = Instant::now();

        // 1. Validate security config
        validate_security_config(security)?;

        let fuel = security.limits.max_fuel;
        let max_memory = security.limits.max_memory_mb as usize * 1024 * 1024;
        let timeout_duration = Duration::from_millis(security.limits.max_execution_ms);

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

        // 3. Create store with memory limits + fuel + HTTP allow-list
        let mut host_state = HostState::with_limits(wasi_ctx, max_memory);
        host_state.set_net_allow_list(security.net_allow_list.clone());
        let mut store = create_store(&self.engine, host_state, fuel)?;

        // 4. Set epoch deadline (1 tick = timeout reached)
        set_epoch_deadline(&mut store, 1);

        // 5. Spawn epoch ticker that increments epoch after timeout
        let engine_clone = self.engine.clone();
        let tick_handle = tokio::spawn(async move {
            tokio::time::sleep(timeout_duration).await;
            engine_clone.increment_epoch();
        });

        // 6. Instantiate and call: try typed API (WIT components), fallback to flat
        let outer_timeout = timeout_duration + Duration::from_millis(500);
        let call_result = tokio::time::timeout(outer_timeout, async {
            match instantiate_tool(&self.linker, &mut store, component).await {
                Ok(tool) => kami_engine::bindings::call_run(&mut store, &tool, input).await,
                Err(_) => {
                    let inst = instantiate_component(&self.linker, &mut store, component).await?;
                    call_tool_run(&mut store, &inst, input).await
                }
            }
        })
        .await;

        tick_handle.abort();

        let duration_ms = start.elapsed().as_millis() as u64;
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
}
