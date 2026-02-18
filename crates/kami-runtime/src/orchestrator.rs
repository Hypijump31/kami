//! Top-level runtime orchestrator.
//!
//! `KamiRuntime` is the main entry point for tool execution.
//! It combines tool resolution, scheduling, and sandboxed execution
//! into a single high-level API.

use std::sync::Arc;

use tracing::{info, warn};
use wasmtime::component::Linker;
use wasmtime::Engine;

use kami_engine::{create_engine, create_linker, HostState, InstanceConfig};
use kami_registry::ToolRepository;
use kami_types::ToolId;

use crate::cache::ComponentCache;
use crate::error::RuntimeError;
use crate::executor::{ExecutionResult, WasmToolExecutor};
use crate::pool::PoolConfig;
use crate::resolver::ToolResolver;
use crate::scheduler::{Scheduler, SchedulerConfig};

/// Configuration for the KAMI runtime.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Component cache size.
    pub cache_size: usize,
    /// Scheduler concurrency limit.
    pub max_concurrent: usize,
    /// Enable epoch interruption for timeout.
    pub epoch_interruption: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            cache_size: 32,
            max_concurrent: 4,
            epoch_interruption: true,
        }
    }
}

impl From<&PoolConfig> for RuntimeConfig {
    fn from(pool: &PoolConfig) -> Self {
        Self {
            cache_size: pool.max_size,
            ..Self::default()
        }
    }
}

/// Top-level runtime orchestrator.
///
/// Provides a simple `execute(tool_id, input)` API that handles:
/// - Tool resolution from the registry
/// - Component compilation and caching
/// - Concurrency control via the scheduler
/// - Sandboxed execution with full isolation
pub struct KamiRuntime {
    executor: WasmToolExecutor,
    resolver: ToolResolver,
    scheduler: Scheduler,
}

impl KamiRuntime {
    /// Creates a new runtime with the given configuration and repository.
    pub fn new(
        config: RuntimeConfig,
        repository: Arc<dyn ToolRepository>,
    ) -> Result<Self, RuntimeError> {
        let instance_config = InstanceConfig {
            epoch_interruption: config.epoch_interruption,
            ..InstanceConfig::default()
        };
        let engine = create_engine(&instance_config)?;
        let linker = create_linker(&engine)?;
        let cache = ComponentCache::new(config.cache_size);
        let scheduler = Scheduler::new(&SchedulerConfig {
            max_concurrent: config.max_concurrent,
        });

        Ok(Self {
            executor: WasmToolExecutor::new(engine.clone(), linker),
            resolver: ToolResolver::new(engine, cache, repository),
            scheduler,
        })
    }

    /// Creates a runtime from an existing engine and linker.
    ///
    /// Useful when the caller needs control over engine configuration.
    pub fn with_engine(
        engine: Engine,
        linker: Linker<HostState>,
        config: RuntimeConfig,
        repository: Arc<dyn ToolRepository>,
    ) -> Self {
        let cache = ComponentCache::new(config.cache_size);
        let scheduler = Scheduler::new(&SchedulerConfig {
            max_concurrent: config.max_concurrent,
        });

        Self {
            executor: WasmToolExecutor::new(engine.clone(), linker),
            resolver: ToolResolver::new(engine, cache, repository),
            scheduler,
        }
    }

    /// Executes a tool by its ID with the given JSON input.
    ///
    /// Full pipeline:
    /// 1. Acquire scheduler permit (concurrency control)
    /// 2. Resolve tool from registry (with caching)
    /// 3. Execute with full sandbox isolation
    pub async fn execute(
        &self,
        tool_id: &ToolId,
        input: &str,
    ) -> Result<ExecutionResult, RuntimeError> {
        info!(%tool_id, "executing tool");

        // 1. Acquire concurrency permit
        let _permit = self.scheduler.acquire().await?;

        // 2. Resolve tool (cache or compile)
        let cached = self.resolver.resolve(tool_id).await?;

        // 3. Execute with isolation
        let result = self
            .executor
            .execute_component(
                &cached.component,
                input,
                &cached.security,
            )
            .await;

        match &result {
            Ok(r) => info!(
                %tool_id,
                success = r.success,
                duration_ms = r.duration_ms,
                fuel_consumed = r.fuel_consumed,
                "execution complete"
            ),
            Err(e) => warn!(%tool_id, error = %e, "execution failed"),
        }

        result
    }

    /// Invalidates the component cache for a specific tool.
    pub async fn invalidate_cache(&self, tool_id: &ToolId) {
        self.resolver.invalidate(tool_id).await;
    }

    /// Returns the underlying executor for direct component execution.
    pub fn executor(&self) -> &WasmToolExecutor {
        &self.executor
    }

    /// Returns the resolver for cache inspection.
    pub fn resolver(&self) -> &ToolResolver {
        &self.resolver
    }

    /// Returns the scheduler for permit inspection.
    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }
}
