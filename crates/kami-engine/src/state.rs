//! Host state for WASM component instances.
//!
//! `HostState` is the `T` in `Store<T>` and must implement `WasiView`.
//! It holds the WASI context, resource table, and per-instance metadata.

use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiView};

/// Per-instance host state passed to `Store<HostState>`.
///
/// Extensible: future phases will add fields for capability tracking,
/// metrics, and host function state.
pub struct HostState {
    /// WASI context controlling I/O permissions.
    wasi_ctx: WasiCtx,
    /// Resource table for Component Model resources.
    resource_table: ResourceTable,
    /// Fuel consumed so far (for reporting).
    fuel_consumed: u64,
}

impl HostState {
    /// Creates a new host state from a pre-built `WasiCtx`.
    pub fn new(wasi_ctx: WasiCtx) -> Self {
        Self {
            wasi_ctx,
            resource_table: ResourceTable::new(),
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
}
