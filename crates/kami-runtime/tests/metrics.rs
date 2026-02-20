//! Tests for `ExecutionMetrics` and `MetricsSnapshot`.

use kami_runtime::{ExecutionMetrics, MetricsSnapshot};

fn zeroed_snapshot() -> MetricsSnapshot {
    MetricsSnapshot {
        total_executions: 0,
        successful_executions: 0,
        failed_executions: 0,
        total_fuel_consumed: 0,
        cache_hits: 0,
        cache_misses: 0,
    }
}

#[test]
fn default_metrics_are_zero() {
    let m = ExecutionMetrics::default();
    assert_eq!(m.snapshot(), zeroed_snapshot());
}

#[test]
fn record_attempt_increments_total() {
    let m = ExecutionMetrics::default();
    m.record_attempt();
    m.record_attempt();
    assert_eq!(m.snapshot().total_executions, 2);
}

#[test]
fn record_success_increments_success_and_fuel() {
    let m = ExecutionMetrics::default();
    m.record_success(500);
    m.record_success(300);
    let s = m.snapshot();
    assert_eq!(s.successful_executions, 2);
    assert_eq!(s.total_fuel_consumed, 800);
}

#[test]
fn record_failure_increments_failed() {
    let m = ExecutionMetrics::default();
    m.record_failure();
    assert_eq!(m.snapshot().failed_executions, 1);
}

#[test]
fn record_cache_hit_and_miss() {
    let m = ExecutionMetrics::default();
    m.record_cache_hit();
    m.record_cache_miss();
    m.record_cache_hit();
    let s = m.snapshot();
    assert_eq!(s.cache_hits, 2);
    assert_eq!(s.cache_misses, 1);
}

#[test]
fn new_shared_returns_arc_with_defaults() {
    let m = ExecutionMetrics::new_shared();
    assert_eq!(m.snapshot(), zeroed_snapshot());
}
