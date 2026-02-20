//! Host state for WASM component instances.
//!
//! `HostState` is the `T` in `Store<T>` and must implement `WasiView`.
//! It holds the WASI context, resource table, store limits, and
//! per-instance metadata.

use wasmtime::component::ResourceTable;
use wasmtime::{StoreLimits, StoreLimitsBuilder};
use wasmtime_wasi::{WasiCtx, WasiView};

/// Per-instance host state passed to `Store<HostState>`.
///
/// Contains the `StoreLimits` so that `Store::limiter()` can reference it.
/// Extensible: future phases will add fields for capability tracking,
/// metrics, and host function state.
pub struct HostState {
    /// WASI context controlling I/O permissions.
    wasi_ctx: WasiCtx,
    /// Resource table for Component Model resources.
    resource_table: ResourceTable,
    /// Wasmtime resource limiter (memory, tables, instances).
    pub(crate) store_limits: StoreLimits,
    /// Fuel consumed so far (for reporting).
    fuel_consumed: u64,
}

impl HostState {
    /// Creates a new host state from a pre-built `WasiCtx` with default
    /// resource limits (no memory cap).
    pub fn new(wasi_ctx: WasiCtx) -> Self {
        Self {
            wasi_ctx,
            resource_table: ResourceTable::new(),
            store_limits: StoreLimitsBuilder::new().build(),
            fuel_consumed: 0,
        }
    }

    /// Creates a new host state with explicit memory limits.
    ///
    /// `max_memory_bytes` caps each linear memory allocation.
    pub fn with_limits(wasi_ctx: WasiCtx, max_memory_bytes: usize) -> Self {
        let store_limits = StoreLimitsBuilder::new()
            .memory_size(max_memory_bytes)
            .trap_on_grow_failure(true)
            .build();
        Self {
            wasi_ctx,
            resource_table: ResourceTable::new(),
            store_limits,
            fuel_consumed: 0,
        }
    }

    /// Returns fuel consumed so far.
    pub fn fuel_consumed(&self) -> u64 {
        self.fuel_consumed
    }

    /// Records fuel consumption.
    pub fn record_fuel(&mut self, consumed: u64) {
        self.fuel_consumed = consumed;
    }
}

impl WasiView for HostState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime_wasi::WasiCtxBuilder;

    #[test]
    fn host_state_creation() {
        let ctx = WasiCtxBuilder::new().build();
        let state = HostState::new(ctx);
        assert_eq!(state.fuel_consumed(), 0);
    }

    #[test]
    fn host_state_with_memory_limits() {
        let ctx = WasiCtxBuilder::new().build();
        let state = HostState::with_limits(ctx, 32 * 1024 * 1024);
        assert_eq!(state.fuel_consumed(), 0);
    }

    #[test]
    fn record_fuel_updates_consumed() {
        let ctx = WasiCtxBuilder::new().build();
        let mut state = HostState::new(ctx);
        state.record_fuel(500);
        assert_eq!(state.fuel_consumed(), 500);
        state.record_fuel(1000);
        assert_eq!(state.fuel_consumed(), 1000);
    }
}
