//! Compiled component cache for warm-start optimization.
//!
//! Caches pre-compiled `wasmtime::component::Component` instances keyed
//! by `ToolId`. Compilation is expensive; instantiation is cheap.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use wasmtime::component::Component;

use kami_types::{SecurityConfig, ToolId};

/// A cached compiled component with its security config.
#[derive(Clone)]
pub struct CachedComponent {
    /// Pre-compiled wasmtime component.
    pub component: Component,
    /// Security config from the tool manifest.
    pub security: SecurityConfig,
    /// WASM file path for cache invalidation.
    pub wasm_path: String,
}

/// Thread-safe cache for compiled WASM components.
///
/// Components are keyed by `ToolId` and can be shared across
/// concurrent executions. Each execution gets its own `Store`
/// but reuses the compiled component.
#[derive(Clone)]
pub struct ComponentCache {
    entries: Arc<RwLock<HashMap<String, CachedComponent>>>,
    max_size: usize,
}

impl ComponentCache {
    /// Creates a new cache with the given maximum size.
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_size,
        }
    }

    /// Gets a cached component by tool ID.
    pub async fn get(&self, id: &ToolId) -> Option<CachedComponent> {
        let entries = self.entries.read().await;
        entries.get(id.as_str()).cloned()
    }

    /// Inserts a compiled component into the cache.
    ///
    /// If the cache is full, the oldest entry is evicted (simple FIFO).
    pub async fn insert(&self, id: &ToolId, entry: CachedComponent) {
        let mut entries = self.entries.write().await;

        // Evict if at capacity (simple strategy: remove first key)
        if entries.len() >= self.max_size
            && !entries.contains_key(id.as_str())
        {
            if let Some(key) = entries.keys().next().cloned() {
                entries.remove(&key);
            }
        }

        entries.insert(id.as_str().to_string(), entry);
    }

    /// Removes a cached component.
    pub async fn invalidate(&self, id: &ToolId) {
        let mut entries = self.entries.write().await;
        entries.remove(id.as_str());
    }

    /// Clears the entire cache.
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }

    /// Returns the number of cached components.
    pub async fn len(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }

    /// Returns true if the cache is empty.
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kami_engine::{create_engine, load_component, InstanceConfig};

    const MINIMAL_WAT: &str = r#"(component
      (core module $m (func (export "x")))
      (core instance $i (instantiate $m))
    )"#;

    fn make_component() -> Component {
        let config = InstanceConfig::default();
        let engine = create_engine(&config).expect("engine");
        load_component(&engine, MINIMAL_WAT.as_bytes())
            .expect("component")
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
            let id =
                ToolId::new(&format!("dev.test.t{i}")).expect("id");
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

        // Cache should have at most 2 entries
        assert_eq!(cache.len().await, 2);
    }
}
