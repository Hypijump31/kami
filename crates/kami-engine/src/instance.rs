//! WASM Component instance lifecycle management.

use wasmtime::{Config, Engine, Store};

use crate::error::EngineError;
use crate::state::HostState;

/// Configuration for creating WASM instances.
#[derive(Debug, Clone)]
pub struct InstanceConfig {
    /// Maximum memory in bytes.
    pub max_memory_bytes: u64,
    /// Fuel limit for execution.
    pub max_fuel: u64,
    /// Enable async support (required for tokio).
    pub async_support: bool,
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024, // 64 MB
            max_fuel: 1_000_000,
            async_support: true,
        }
    }
}

/// Creates a Wasmtime `Engine` configured for KAMI.
///
/// The engine is reusable across all component compilations and
/// should be created once at startup.
pub fn create_engine(config: &InstanceConfig) -> Result<Engine, EngineError> {
    let mut wasm_config = Config::new();
    wasm_config.async_support(config.async_support);
    wasm_config.consume_fuel(true);
    wasm_config.wasm_component_model(true);

    Engine::new(&wasm_config).map_err(|e| EngineError::Config(e.to_string()))
}

/// Creates a new `Store<HostState>` with fuel metering.
pub fn create_store(
    engine: &Engine,
    host_state: HostState,
    fuel: u64,
) -> Result<Store<HostState>, EngineError> {
    let mut store = Store::new(engine, host_state);
    store
        .set_fuel(fuel)
        .map_err(|e| EngineError::Config(e.to_string()))?;
    Ok(store)
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
        let engine = create_engine(&config).unwrap();
        let ctx = WasiCtxBuilder::new().build();
        let state = HostState::new(ctx);
        let store = create_store(&engine, state, 500_000);
        assert!(store.is_ok());
    }
}
