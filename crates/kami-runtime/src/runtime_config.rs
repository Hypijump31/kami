//! Configuration for the KAMI runtime.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = RuntimeConfig::default();
        assert_eq!(cfg.cache_size, 32);
        assert_eq!(cfg.max_concurrent, 4);
        assert!(cfg.epoch_interruption);
    }

    #[test]
    fn config_is_cloneable() {
        let cfg = RuntimeConfig {
            cache_size: 64,
            max_concurrent: 8,
            epoch_interruption: false,
        };
        let copy = cfg.clone();
        assert_eq!(copy.cache_size, 64);
        assert!(!copy.epoch_interruption);
    }
}
