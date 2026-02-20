//! External scheduler tests for priority and drain coverage.

use kami_runtime::{Priority, Scheduler, SchedulerConfig};

#[test]
fn priority_ordering() {
    assert!(Priority::High > Priority::Normal);
    assert!(Priority::Normal > Priority::Low);
}

#[test]
fn priority_default_is_normal() {
    assert_eq!(Priority::default(), Priority::Normal);
}

#[test]
fn scheduler_max_concurrent_accessor() {
    let s = Scheduler::new(&SchedulerConfig { max_concurrent: 8 });
    assert_eq!(s.max_concurrent(), 8);
}

#[test]
fn scheduler_config_default() {
    let config = SchedulerConfig::default();
    assert_eq!(config.max_concurrent, 4);
}

#[tokio::test]
async fn scheduler_drain_returns_when_idle() {
    let s = Scheduler::new(&SchedulerConfig { max_concurrent: 2 });
    // No in-flight tasks, drain should return immediately
    s.drain().await;
    assert_eq!(s.available_permits(), 2);
}

#[tokio::test]
async fn scheduler_drain_waits_for_in_flight() {
    let s = Scheduler::new(&SchedulerConfig { max_concurrent: 2 });
    let p = s.acquire().await.expect("permit");
    assert_eq!(s.available_permits(), 1);

    let s_clone = s.clone();
    let handle = tokio::spawn(async move {
        s_clone.drain().await;
    });

    // Release the permit — drain should now be able to acquire all
    drop(p);
    handle.await.expect("drain completes");
    assert_eq!(s.available_permits(), 2);
}
