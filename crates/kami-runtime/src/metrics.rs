//! Execution metrics for the KAMI runtime.
//!
//! Tracks key counters using lock-free atomics for zero-overhead recording
//! from concurrent async tasks. Use [`MetricsSnapshot`] for human-readable output.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Shared, thread-safe execution metrics collected by [`super::KamiRuntime`].
///
/// All fields are `AtomicU64` — incrementing from any async task is safe and fast.
#[derive(Debug, Default)]
pub struct ExecutionMetrics {
    /// Total number of tool executions attempted.
    pub total_executions: AtomicU64,
    /// Number of executions that completed successfully.
    pub successful_executions: AtomicU64,
    /// Number of executions that returned an error.
    pub failed_executions: AtomicU64,
    /// Cumulative fuel consumed across all successful executions.
    pub total_fuel_consumed: AtomicU64,
    /// Number of times a compiled component was found in the cache.
    pub cache_hits: AtomicU64,
    /// Number of times a component had to be compiled from scratch.
    pub cache_misses: AtomicU64,
}

/// A point-in-time snapshot of [`ExecutionMetrics`].
///
/// Use [`ExecutionMetrics::snapshot`] to obtain a copyable view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricsSnapshot {
    /// Total executions attempted.
    pub total_executions: u64,
    /// Successful executions.
    pub successful_executions: u64,
    /// Failed executions.
    pub failed_executions: u64,
    /// Cumulative fuel consumed.
    pub total_fuel_consumed: u64,
    /// Cache hits (compiled component reused).
    pub cache_hits: u64,
    /// Cache misses (component compiled fresh).
    pub cache_misses: u64,
}

impl ExecutionMetrics {
    /// Creates a new zeroed metrics instance wrapped in an [`Arc`].
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Records one attempted execution.
    pub fn record_attempt(&self) {
        self.total_executions.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a successful execution with the given fuel consumed.
    pub fn record_success(&self, fuel_consumed: u64) {
        self.successful_executions.fetch_add(1, Ordering::Relaxed);
        self.total_fuel_consumed
            .fetch_add(fuel_consumed, Ordering::Relaxed);
    }

    /// Records a failed execution.
    pub fn record_failure(&self) {
        self.failed_executions.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a cache hit (component was reused from the cache).
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a cache miss (component was compiled and stored).
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns a point-in-time snapshot of all counters.
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_executions: self.total_executions.load(Ordering::Relaxed),
            successful_executions: self.successful_executions.load(Ordering::Relaxed),
            failed_executions: self.failed_executions.load(Ordering::Relaxed),
            total_fuel_consumed: self.total_fuel_consumed.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
        }
    }
}
