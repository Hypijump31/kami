//! Compiled component cache with LRU eviction.
//!
//! Caches pre-compiled `wasmtime::component::Component` instances keyed
//! by `ToolId`. Compilation is expensive; instantiation is cheap.
//! Uses a `Mutex` since LRU tracking requires mutable access on reads.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
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

/// Thread-safe LRU cache for compiled WASM components.
///
/// Components are keyed by `ToolId` and can be shared across
/// concurrent executions. Uses LRU eviction when the cache
/// reaches its maximum capacity.
#[derive(Clone)]
pub struct ComponentCache {
    inner: Arc<Mutex<LruInner>>,
}

/// Internal LRU state: `order` front = least recently used.
struct LruInner {
    entries: HashMap<String, CachedComponent>,
    order: Vec<String>,
    max_size: usize,
}

impl ComponentCache {
    /// Creates a new cache with the given maximum size.
    pub fn new(max_size: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LruInner {
                entries: HashMap::new(),
                order: Vec::with_capacity(max_size),
                max_size,
            })),
        }
    }

    /// Gets a cached component by tool ID, marking it as recently used.
    pub async fn get(&self, id: &ToolId) -> Option<CachedComponent> {
        let mut inner = self.inner.lock().await;
        let key = id.as_str();
        if inner.entries.contains_key(key) {
            inner.touch(key);
            inner.entries.get(key).cloned()
        } else {
            None
        }
    }

    /// Inserts a compiled component into the cache.
    ///
    /// If the cache is full, the least recently used entry is evicted.
    pub async fn insert(&self, id: &ToolId, entry: CachedComponent) {
        let mut inner = self.inner.lock().await;
        let key = id.as_str().to_string();
        if inner.entries.contains_key(&key) {
            inner.touch(&key);
        } else {
            if inner.entries.len() >= inner.max_size {
                inner.evict_lru();
            }
            inner.order.push(key.clone());
        }
        inner.entries.insert(key, entry);
    }

    /// Removes a cached component.
    pub async fn invalidate(&self, id: &ToolId) {
        let mut inner = self.inner.lock().await;
        let key = id.as_str();
        inner.entries.remove(key);
        inner.order.retain(|k| k != key);
    }

    /// Clears the entire cache.
    pub async fn clear(&self) {
        let mut inner = self.inner.lock().await;
        inner.entries.clear();
        inner.order.clear();
    }

    /// Returns the number of cached components.
    pub async fn len(&self) -> usize {
        let inner = self.inner.lock().await;
        inner.entries.len()
    }

    /// Returns true if the cache is empty.
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

impl LruInner {
    /// Moves `key` to the back (most recently used).
    fn touch(&mut self, key: &str) {
        self.order.retain(|k| k != key);
        self.order.push(key.to_string());
    }

    /// Evicts the least recently used entry (front of the order vec).
    fn evict_lru(&mut self) {
        if let Some(lru_key) = self.order.first().cloned() {
            self.order.remove(0);
            self.entries.remove(&lru_key);
        }
    }
}
