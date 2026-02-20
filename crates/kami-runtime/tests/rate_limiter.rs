//! Tests for `RateLimiter` token-bucket rate limiting.

use kami_runtime::{RateLimitConfig, RateLimiter};
use kami_types::ToolId;

fn tool(name: &str) -> ToolId {
    ToolId::new(name).unwrap_or_else(|_| panic!("bad id"))
}

#[test]
fn allows_within_limit() {
    let config = RateLimitConfig {
        per_tool: 2,
        global: 10,
        ..Default::default()
    };
    let limiter = RateLimiter::new(&config);
    let id = tool("dev.test.a");
    assert!(limiter.check(&id));
    assert!(limiter.check(&id));
}

#[test]
fn denies_over_per_tool_limit() {
    let config = RateLimitConfig {
        per_tool: 1,
        global: 100,
        ..Default::default()
    };
    let limiter = RateLimiter::new(&config);
    let id = tool("dev.test.b");
    assert!(limiter.check(&id));
    assert!(!limiter.check(&id));
}

#[test]
fn denies_over_global_limit() {
    let config = RateLimitConfig {
        per_tool: 100,
        global: 2,
        ..Default::default()
    };
    let limiter = RateLimiter::new(&config);
    assert!(limiter.check(&tool("dev.test.c")));
    assert!(limiter.check(&tool("dev.test.d")));
    assert!(!limiter.check(&tool("dev.test.e")));
}

#[test]
fn unlimited_when_zero() {
    let config = RateLimitConfig {
        per_tool: 0,
        global: 0,
        ..Default::default()
    };
    let limiter = RateLimiter::new(&config);
    for _ in 0..1000 {
        assert!(limiter.check(&tool("dev.test.f")));
    }
}

#[test]
fn global_zero_per_tool_enforced() {
    // global=0 (skip), per_tool=1 should still deny after 1 request
    let config = RateLimitConfig {
        per_tool: 1,
        global: 0,
        ..Default::default()
    };
    let limiter = RateLimiter::new(&config);
    let id = tool("dev.test.g");
    assert!(limiter.check(&id));
    assert!(!limiter.check(&id));
}

#[test]
fn per_tool_zero_global_enforced() {
    // per_tool=0 (skip), global=2 should deny after 2 requests
    let config = RateLimitConfig {
        per_tool: 0,
        global: 2,
        ..Default::default()
    };
    let limiter = RateLimiter::new(&config);
    assert!(limiter.check(&tool("dev.test.h")));
    assert!(limiter.check(&tool("dev.test.i")));
    assert!(!limiter.check(&tool("dev.test.j")));
}

#[test]
fn default_config_values() {
    let config = RateLimitConfig::default();
    assert_eq!(config.per_tool, 100);
    assert_eq!(config.global, 1000);
    assert_eq!(config.window.as_secs(), 60);
}
