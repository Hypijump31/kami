//! Memory management and limits for WASM instances.

/// Memory usage statistics for a WASM instance.
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Current memory usage in bytes.
    pub current_bytes: u64,
    /// Peak memory usage in bytes.
    pub peak_bytes: u64,
    /// Memory limit in bytes.
    pub limit_bytes: u64,
}

impl MemoryStats {
    /// Returns the usage as a percentage of the limit.
    pub fn usage_percent(&self) -> f64 {
        if self.limit_bytes == 0 {
            return 0.0;
        }
        (self.current_bytes as f64 / self.limit_bytes as f64) * 100.0
    }
}
