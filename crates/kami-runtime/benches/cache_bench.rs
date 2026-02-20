//! Benchmark for `ComponentCache` LRU operations.
//!
//! Measures insert, get (hit/miss), and eviction under load using a real
//! (minimal) WASM component for realistic cache entry sizes.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kami_runtime::cache::{CachedComponent, ComponentCache};
use kami_types::{SecurityConfig, ToolId};
use wasmtime::{component::Component, Config, Engine};

/// Minimal valid WASM component (empty component, ~8 bytes).
const MINIMAL_WAT: &str = "(component)";

/// Creates a `CachedComponent` wrapping a real compiled component.
fn make_entry(engine: &Engine, path: &str) -> CachedComponent {
    let component = Component::new(engine, MINIMAL_WAT).expect("bench: compile minimal");
    CachedComponent {
        component,
        security: SecurityConfig::default(),
        wasm_path: path.to_string(),
    }
}

/// Shared engine (component model enabled).
fn bench_engine() -> Engine {
    let mut config = Config::new();
    config.wasm_component_model(true);
    Engine::new(&config).expect("bench: engine")
}

fn bench_cache_insert(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("bench: tokio");
    let engine = bench_engine();

    c.bench_function("cache_insert_100", |b| {
        b.iter(|| {
            rt.block_on(async {
                let cache = ComponentCache::new(100);
                for i in 0..100u32 {
                    let id_str = format!("dev.test.tool-{i}");
                    let id = ToolId::new(&id_str).expect("bench: id");
                    cache
                        .insert(&id, make_entry(&engine, &format!("t{i}.wasm")))
                        .await;
                }
                black_box(cache.len().await);
            });
        });
    });
}

fn bench_cache_get_hit(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("bench: tokio");
    let engine = bench_engine();
    let cache = rt.block_on(async {
        let c = ComponentCache::new(100);
        for i in 0..100u32 {
            let id_str = format!("dev.test.tool-{i}");
            let id = ToolId::new(&id_str).expect("bench: id");
            c.insert(&id, make_entry(&engine, &format!("t{i}.wasm")))
                .await;
        }
        c
    });

    c.bench_function("cache_get_hit", |b| {
        b.iter(|| {
            rt.block_on(async {
                let id = ToolId::new("dev.test.tool-50").expect("bench: id");
                black_box(cache.get(&id).await);
            });
        });
    });
}

fn bench_cache_get_miss(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("bench: tokio");
    let cache = ComponentCache::new(100);

    c.bench_function("cache_get_miss", |b| {
        b.iter(|| {
            rt.block_on(async {
                let id = ToolId::new("dev.test.none").expect("bench: id");
                black_box(cache.get(&id).await);
            });
        });
    });
}

fn bench_cache_eviction(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("bench: tokio");
    let engine = bench_engine();

    c.bench_function("cache_evict_lru_10", |b| {
        b.iter(|| {
            rt.block_on(async {
                let cache = ComponentCache::new(10);
                for i in 0..20u32 {
                    let id_str = format!("dev.test.tool-{i}");
                    let id = ToolId::new(&id_str).expect("bench: id");
                    cache
                        .insert(&id, make_entry(&engine, &format!("t{i}.wasm")))
                        .await;
                }
                black_box(cache.len().await);
            });
        });
    });
}

criterion_group!(
    benches,
    bench_cache_insert,
    bench_cache_get_hit,
    bench_cache_get_miss,
    bench_cache_eviction,
);
criterion_main!(benches);
