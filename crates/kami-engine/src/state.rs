//! Host state for WASM component instances.
//!
//! `HostState` is the `T` in `Store<T>` and must implement both `WasiView`
//! and `WasiHttpView`. Holds WASI context, HTTP context, resource table,
//! store limits, and the `net_allow_list` used to enforce outgoing HTTP access.

use hyper::Request;
use wasmtime::component::ResourceTable;
use wasmtime::{StoreLimits, StoreLimitsBuilder};
use wasmtime_wasi::{WasiCtx, WasiView};
use wasmtime_wasi_http::bindings::http::types::ErrorCode;
use wasmtime_wasi_http::body::HyperOutgoingBody;
use wasmtime_wasi_http::types::{HostFutureIncomingResponse, OutgoingRequestConfig};
use wasmtime_wasi_http::{HttpResult, WasiHttpCtx, WasiHttpView};

/// Per-instance host state passed to `Store<HostState>`.
///
/// Contains `StoreLimits` so that `Store::limiter()` can reference it.
/// Contains `WasiHttpCtx` for WASI HTTP outgoing support.
/// Contains `net_allow_list` for per-request network enforcement.
pub struct HostState {
    /// WASI context controlling I/O permissions.
    wasi_ctx: WasiCtx,
    /// Resource table for Component Model resources (shared with HTTP).
    resource_table: ResourceTable,
    /// Wasmtime resource limiter (memory, tables, instances).
    pub(crate) store_limits: StoreLimits,
    /// Fuel consumed so far (for reporting).
    fuel_consumed: u64,
    /// WASI HTTP context for outgoing HTTP requests.
    http_ctx: WasiHttpCtx,
    /// Network allow list enforced in `send_request`.
    net_allow_list: Vec<String>,
}

impl HostState {
    /// Creates a new host state with default resource limits.
    pub fn new(wasi_ctx: WasiCtx) -> Self {
        Self {
            wasi_ctx,
            resource_table: ResourceTable::new(),
            store_limits: StoreLimitsBuilder::new().build(),
            fuel_consumed: 0,
            http_ctx: WasiHttpCtx::new(),
            net_allow_list: Vec::new(),
        }
    }

    /// Creates a new host state with explicit memory limits.
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
            http_ctx: WasiHttpCtx::new(),
            net_allow_list: Vec::new(),
        }
    }

    /// Sets the network allow list used to filter outgoing HTTP requests.
    ///
    /// An empty list means all HTTP outgoing is denied (deny-all default).
    pub fn set_net_allow_list(&mut self, allow_list: Vec<String>) {
        self.net_allow_list = allow_list;
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

impl WasiHttpView for HostState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http_ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }

    /// Enforces `net_allow_list` before forwarding HTTP requests.
    ///
    /// Deny-all when `net_allow_list` is empty. Checks host against patterns
    /// (exact or wildcard `*.example.com`) when the list is non-empty.
    fn send_request(
        &mut self,
        request: Request<HyperOutgoingBody>,
        config: OutgoingRequestConfig,
    ) -> HttpResult<HostFutureIncomingResponse> {
        let host = request
            .uri()
            .host()
            .map_or(String::new(), str::to_string);
        if !is_http_host_allowed(&host, &self.net_allow_list) {
            tracing::warn!(host = %host, "outgoing HTTP denied by net_allow_list");
            return Err(ErrorCode::ConnectionRefused.into());
        }
        Ok(wasmtime_wasi_http::types::default_send_request(
            request, config,
        ))
    }
}

/// Checks if an HTTP host matches the allow list (exact or wildcard).
///
/// Returns `false` when the list is empty (deny-all default).
fn is_http_host_allowed(host: &str, allow_list: &[String]) -> bool {
    if allow_list.is_empty() {
        return false;
    }
    allow_list.iter().any(|pattern| {
        if let Some(suffix) = pattern.strip_prefix("*.") {
            host == suffix || host.ends_with(&format!(".{suffix}"))
        } else {
            host == pattern
        }
    })
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
    }

    #[test]
    fn deny_all_when_allow_list_empty() {
        assert!(!is_http_host_allowed("api.example.com", &[]));
    }

    #[test]
    fn allow_exact_host() {
        let list = vec!["api.example.com".to_string()];
        assert!(is_http_host_allowed("api.example.com", &list));
        assert!(!is_http_host_allowed("other.example.com", &list));
    }

    #[test]
    fn allow_wildcard_host() {
        let list = vec!["*.example.com".to_string()];
        assert!(is_http_host_allowed("api.example.com", &list));
        assert!(is_http_host_allowed("sub.example.com", &list));
        assert!(!is_http_host_allowed("evil.com", &list));
    }
}
