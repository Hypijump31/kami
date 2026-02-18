//! WASM Component instance lifecycle management.

use wasmtime::{Config, Engine, Store};

use crate::error::EngineError;
use crate::state::HostState;

/// Configuration for creating WASM instances.
#[derive(Debug, Clone)]
pub struct InstanceConfig {
    /// Maximum memory in bytes per linear memory.
    pub max_memory_bytes: u64,
    /// Fuel limit for execution.
    pub max_fuel: u64,
    /// Enable async support (required for tokio).
    pub async_support: bool,
    /// Enable epoch-based interruption for timeout enforcement.
    pub epoch_interruption: bool,
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024, // 64 MB
            max_fuel: 1_000_000,
            async_support: true,
            epoch_interruption: false,
        }
    }
}

/// Creates a Wasmtime `Engine` configured for KAMI.
///
/// The engine is reusable across all component compilations and
/// should be created once at startup. Enables fuel metering,
/// Component Model, and epoch interruption.
pub fn create_engine(config: &InstanceConfig) -> Result<Engine, EngineError> {
    let mut wasm_config = Config::new();
    wasm_config.async_support(config.async_support);
    wasm_config.consume_fuel(true);
    wasm_config.wasm_component_model(true);
    if config.epoch_interruption {
        wasm_config.epoch_interruption(true);
    }

    Engine::new(&wasm_config).map_err(|e| EngineError::Config(e.to_string()))
}

/// Creates a new `Store<HostState>` with fuel metering and resource limits.
///
/// Connects `StoreLimits` from `HostState` via `store.limiter()` and sets
/// the fuel budget for instruction-level metering.
pub fn create_store(
    engine: &Engine,
    host_state: HostState,
    fuel: u64,
) -> Result<Store<HostState>, EngineError> {
    let mut store = Store::new(engine, host_state);

    // Connect resource limiter (memory, tables)
    store.limiter(|state| &mut state.store_limits);

    // Set fuel budget
    store
        .set_fuel(fuel)
        .map_err(|e| EngineError::Config(e.to_string()))?;

    Ok(store)
}

/// Sets an epoch deadline on a store for timeout enforcement.
///
/// The store will trap when the engine's epoch counter exceeds
/// `ticks_beyond_current`. Use `Engine::increment_epoch()` from
/// a separate tokio task to trigger the deadline after a timeout.
pub fn set_epoch_deadline(store: &mut Store<HostState>, ticks: u64) {
    store.epoch_deadline_trap();
    store.set_epoch_deadline(ticks);
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime_wasi::WasiCtxBuilder;

    #[test]
    fn create_engine_with_defaults() {
        let config = InstanceConfig::default();
        let engine = create_engine(&config);
        assert!(engine.is_ok());
    }

    #[test]
    fn create_store_with_fuel() {
        let config = InstanceConfig::default();
        let engine = create_engine(&config).expect("engine creation");
        let ctx = WasiCtxBuilder::new().build();
        let state = HostState::new(ctx);
        let store = create_store(&engine, state, 500_000);
        assert!(store.is_ok());
    }

    #[test]
    fn store_with_memory_limits() {
        let config = InstanceConfig::default();
        let engine = create_engine(&config).expect("engine creation");
        let ctx = WasiCtxBuilder::new().build();
        let state = HostState::with_limits(ctx, 16 * 1024 * 1024);
        let store = create_store(&engine, state, 500_000);
        assert!(store.is_ok());
    }

    #[test]
    fn epoch_deadline_can_be_set() {
        let config = InstanceConfig::default();
        let engine = create_engine(&config).expect("engine creation");
        let ctx = WasiCtxBuilder::new().build();
        let state = HostState::new(ctx);
        let mut store =
            create_store(&engine, state, 500_000).expect("store creation");
        set_epoch_deadline(&mut store, 1);
    }
}
