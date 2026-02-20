//! Integration tests for ComponentCache.

use kami_engine::{create_engine, load_component, InstanceConfig};
use kami_runtime::{CachedComponent, ComponentCache};
use kami_types::{SecurityConfig, ToolId};
use wasmtime::component::Component;

const MINIMAL_WAT: &str = r#"(component
  (core module $m (func (export "x")))
  (core instance $i (instantiate $m))
)"#;

fn make_component() -> Component {
    let config = InstanceConfig::default();
    let engine = create_engine(&config).expect("engine");
    load_component(&engine, MINIMAL_WAT.as_bytes()).expect("component")
}

#[tokio::test]
async fn insert_and_get() {
    let cache = ComponentCache::new(10);
    let id = ToolId::new("dev.test.tool").expect("id");
    let component = make_component();

    cache
        .insert(
            &id,
            CachedComponent {
                component,
                security: SecurityConfig::default(),
                wasm_path: "test.wasm".to_string(),
            },
        )
        .await;

    assert!(cache.get(&id).await.is_some());
    assert_eq!(cache.len().await, 1);
}

#[tokio::test]
async fn get_missing_returns_none() {
    let cache = ComponentCache::new(10);
    let id = ToolId::new("dev.test.nope").expect("id");
    assert!(cache.get(&id).await.is_none());
}

#[tokio::test]
async fn invalidate_removes_entry() {
    let cache = ComponentCache::new(10);
    let id = ToolId::new("dev.test.tool").expect("id");

    cache
        .insert(
            &id,
            CachedComponent {
                component: make_component(),
                security: SecurityConfig::default(),
                wasm_path: "test.wasm".to_string(),
            },
        )
        .await;

    cache.invalidate(&id).await;
    assert!(cache.get(&id).await.is_none());
}

#[tokio::test]
async fn eviction_at_capacity() {
    let cache = ComponentCache::new(2);

    for i in 0..3 {
        let id = ToolId::new(format!("dev.test.t{i}")).expect("id");
        cache
            .insert(
                &id,
                CachedComponent {
                    component: make_component(),
                    security: SecurityConfig::default(),
                    wasm_path: format!("t{i}.wasm"),
                },
            )
            .await;
    }

    assert_eq!(cache.len().await, 2);
}

#[tokio::test]
async fn lru_evicts_least_recently_used() {
    let cache = ComponentCache::new(2);
    let t0 = ToolId::new("dev.test.t0").expect("id");
    let t1 = ToolId::new("dev.test.t1").expect("id");
    let t2 = ToolId::new("dev.test.t2").expect("id");

    let entry = |path: &str| CachedComponent {
        component: make_component(),
        security: SecurityConfig::default(),
        wasm_path: path.to_string(),
    };

    cache.insert(&t0, entry("t0.wasm")).await;
    cache.insert(&t1, entry("t1.wasm")).await;

    // Access t0 so it becomes most-recently-used
    cache.get(&t0).await;

    // Insert t2 — should evict t1 (least recently used), not t0
    cache.insert(&t2, entry("t2.wasm")).await;

    assert!(cache.get(&t0).await.is_some(), "t0 should survive");
    assert!(cache.get(&t1).await.is_none(), "t1 should be evicted");
    assert!(cache.get(&t2).await.is_some(), "t2 should exist");
}

#[tokio::test]
async fn clear_empties_cache_and_is_empty() {
    let cache = ComponentCache::new(4);
    let id = ToolId::new("dev.test.clr").expect("id");
    let entry = CachedComponent {
        component: make_component(),
        security: SecurityConfig::default(),
        wasm_path: "clr.wasm".to_string(),
    };
    cache.insert(&id, entry).await;
    assert!(!cache.is_empty().await);
    cache.clear().await;
    assert!(cache.is_empty().await);
    assert_eq!(cache.len().await, 0);
}

#[tokio::test]
async fn reinserting_same_key_keeps_count_at_one() {
    let cache = ComponentCache::new(4);
    let id = ToolId::new("dev.test.ri").expect("id");
    let mk = || CachedComponent {
        component: make_component(),
        security: SecurityConfig::default(),
        wasm_path: "ri.wasm".to_string(),
    };
    cache.insert(&id, mk()).await;
    cache.insert(&id, mk()).await;
    assert_eq!(cache.len().await, 1);
}
