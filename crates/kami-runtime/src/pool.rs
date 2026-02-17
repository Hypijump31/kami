//! Instance pool for warm-start optimization.

/// Configuration for the instance pool.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of cached instances.
    pub max_size: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self { max_size: 5 }
    }
}
