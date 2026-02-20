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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_percent_zero_limit_returns_zero() {
        let stats = MemoryStats {
            current_bytes: 1024,
            peak_bytes: 2048,
            limit_bytes: 0,
        };
        assert!((stats.usage_percent() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn usage_percent_computes_correctly() {
        let stats = MemoryStats {
            current_bytes: 50,
            peak_bytes: 75,
            limit_bytes: 100,
        };
        assert!((stats.usage_percent() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn default_stats_are_zeroed() {
        let stats = MemoryStats::default();
        assert_eq!(stats.current_bytes, 0);
        assert_eq!(stats.limit_bytes, 0);
        assert!((stats.usage_percent() - 0.0).abs() < f64::EPSILON);
    }
}
