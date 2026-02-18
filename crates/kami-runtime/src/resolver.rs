//! Tool resolution from registry to compiled component.
//!
//! Resolves a `ToolId` by looking it up in the registry, loading the
//! WASM file, compiling it, and caching the result.

use std::path::Path;
use std::sync::Arc;

use tracing::{debug, info};
use wasmtime::Engine;

use kami_engine::load_component_from_file;
use kami_registry::ToolRepository;
use kami_types::ToolId;

use crate::cache::{CachedComponent, ComponentCache};
use crate::error::RuntimeError;

/// Resolves tools from the registry and compiles their WASM components.
///
/// Uses `ComponentCache` to avoid recompilation on subsequent calls.
pub struct ToolResolver {
    engine: Engine,
    cache: ComponentCache,
    repository: Arc<dyn ToolRepository>,
}

impl ToolResolver {
    /// Creates a new resolver with the given engine, cache, and repository.
    pub fn new(
        engine: Engine,
        cache: ComponentCache,
        repository: Arc<dyn ToolRepository>,
    ) -> Self {
        Self {
            engine,
            cache,
            repository,
        }
    }

    /// Resolves a tool ID to a compiled component.
    ///
    /// Returns the cached component if available, otherwise loads from
    /// the registry, compiles, and caches.
    pub async fn resolve(
        &self,
        id: &ToolId,
    ) -> Result<CachedComponent, RuntimeError> {
        // 1. Check cache first
        if let Some(cached) = self.cache.get(id).await {
            debug!(%id, "cache hit");
            return Ok(cached);
        }

        // 2. Look up in registry
        let tool = self
            .repository
            .find_by_id(id)
            .await
            .map_err(|e| RuntimeError::ToolNotFound {
                name: format!("{id}: {e}"),
            })?
            .ok_or_else(|| RuntimeError::ToolNotFound {
                name: id.to_string(),
            })?;

        // 3. Resolve WASM file path
        let wasm_path =
            Path::new(&tool.install_path).join(&tool.manifest.wasm);

        if !wasm_path.exists() {
            return Err(RuntimeError::ToolNotFound {
                name: format!(
                    "WASM file missing: {}",
                    wasm_path.display()
                ),
            });
        }

        info!(%id, path = %wasm_path.display(), "compiling component");

        // 4. Compile the component
        let component =
            load_component_from_file(&self.engine, &wasm_path)?;

        // 5. Cache it
        let cached = CachedComponent {
            component,
            security: tool.manifest.security.clone(),
            wasm_path: wasm_path.display().to_string(),
        };
        self.cache.insert(id, cached.clone()).await;

        Ok(cached)
    }

    /// Invalidates the cache for a specific tool.
    pub async fn invalidate(&self, id: &ToolId) {
        self.cache.invalidate(id).await;
    }

    /// Returns a reference to the component cache.
    pub fn cache(&self) -> &ComponentCache {
        &self.cache
    }
}
