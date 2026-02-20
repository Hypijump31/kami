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
use crate::integrity;

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
    pub fn new(engine: Engine, cache: ComponentCache, repository: Arc<dyn ToolRepository>) -> Self {
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
    #[tracing::instrument(skip(self), fields(tool_id = %id))]
    pub async fn resolve(&self, id: &ToolId) -> Result<CachedComponent, RuntimeError> {
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
        let wasm_path = Path::new(&tool.install_path).join(&tool.manifest.wasm);

        if !wasm_path.exists() {
            return Err(RuntimeError::ToolNotFound {
                name: format!("WASM file missing: {}", wasm_path.display()),
            });
        }

        // 4. Verify WASM integrity (skipped if no hash stored at install time)
        integrity::verify_hash(&wasm_path, &tool.manifest.wasm_sha256).map_err(|e| {
            RuntimeError::IntegrityViolation {
                tool_id: id.to_string(),
                detail: e.to_string(),
            }
        })?;

        // 5. Verify Ed25519 signature if stored
        if let (Some(sig), Some(pk)) = (&tool.manifest.signature, &tool.manifest.signer_public_key)
        {
            crate::signature::verify_file_signature(&wasm_path, sig, pk).map_err(|e| {
                RuntimeError::IntegrityViolation {
                    tool_id: id.to_string(),
                    detail: format!("signature verification failed: {e}"),
                }
            })?;
            debug!(%id, "signature verified");
        }

        info!(%id, path = %wasm_path.display(), "compiling component");

        // 5. Compile the component
        let component = load_component_from_file(&self.engine, &wasm_path)?;

        // 6. Cache it
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
