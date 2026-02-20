//! Integration tests for `ToolResolver` — error path coverage.
//!
//! Tests the resolver's behavior when tools are missing from the registry,
//! WASM files are absent from disk, or the hash check fails.

use std::sync::Arc;

use async_trait::async_trait;

use kami_engine::{create_engine, create_linker, InstanceConfig};
use kami_registry::{RepositoryError, ToolQuery, ToolRepository};
use kami_runtime::{ComponentCache, RuntimeError, ToolResolver};
use kami_types::{Tool, ToolId};

// ---------------------------------------------------------------------------
// Mock repository
// ---------------------------------------------------------------------------

struct MockRepository {
    tool: Option<Tool>,
}

impl MockRepository {
    fn returning(tool: Option<Tool>) -> Arc<Self> {
        Arc::new(Self { tool })
    }
}

#[async_trait]
impl ToolRepository for MockRepository {
    async fn find_by_id(&self, _id: &ToolId) -> Result<Option<Tool>, RepositoryError> {
        Ok(self.tool.clone())
    }

    async fn find_all(&self, _query: ToolQuery) -> Result<Vec<Tool>, RepositoryError> {
        Ok(self.tool.iter().cloned().collect())
    }

    async fn insert(&self, _tool: &Tool) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn update(&self, _tool: &Tool) -> Result<(), RepositoryError> {
        Ok(())
    }

    async fn delete(&self, _id: &ToolId) -> Result<bool, RepositoryError> {
        Ok(false)
    }
}

fn make_resolver(repo: Arc<dyn ToolRepository>) -> ToolResolver {
    let config = InstanceConfig::default();
    let engine = create_engine(&config).expect("engine");
    let linker = create_linker(&engine).expect("linker");
    let _ = linker; // linker not needed for resolver construction
    let cache = ComponentCache::new(4);
    ToolResolver::new(engine, cache, repo)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn resolve_returns_not_found_for_missing_tool() {
    let repo = MockRepository::returning(None);
    let resolver = make_resolver(repo);
    let id = ToolId::new("dev.test.missing").expect("id");

    let result = resolver.resolve(&id).await;

    assert!(
        matches!(result, Err(RuntimeError::ToolNotFound { .. })),
        "expected ToolNotFound",
    );
}

#[tokio::test]
async fn resolve_returns_not_found_when_wasm_file_missing() {
    use kami_types::{SecurityConfig, ToolManifest, ToolVersion};

    let tool = Tool {
        manifest: ToolManifest {
            id: ToolId::new("dev.test.no-wasm").expect("id"),
            name: "no-wasm".to_string(),
            version: ToolVersion::new(1, 0, 0),
            wasm: "nonexistent.wasm".to_string(),
            description: "Tool with missing WASM".to_string(),
            arguments: vec![],
            security: SecurityConfig::default(),
            wasm_sha256: None,
            signature: None,
            signer_public_key: None,
        },
        install_path: "/nonexistent/path".to_string(),
        enabled: true,
        pinned_version: None,
        updated_at: None,
    };

    let repo = MockRepository::returning(Some(tool));
    let resolver = make_resolver(repo);
    let id = ToolId::new("dev.test.no-wasm").expect("id");

    let result = resolver.resolve(&id).await;

    assert!(
        matches!(result, Err(RuntimeError::ToolNotFound { .. })),
        "expected ToolNotFound for missing WASM",
    );
}

#[tokio::test]
async fn resolve_integrity_violation_on_wrong_hash() {
    use kami_types::{SecurityConfig, ToolManifest, ToolVersion};
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut wasm_file = NamedTempFile::new().expect("tempfile");
    wasm_file
        .write_all(b"\x00asm\x01\x00\x00\x00")
        .expect("write");
    let wasm_path = wasm_file.path().to_path_buf();
    let install_path = wasm_path.parent().expect("parent").display().to_string();
    let wasm_filename = wasm_path
        .file_name()
        .expect("name")
        .to_string_lossy()
        .to_string();

    let tool = Tool {
        manifest: ToolManifest {
            id: ToolId::new("dev.test.integrity").expect("id"),
            name: "integrity".to_string(),
            version: ToolVersion::new(1, 0, 0),
            wasm: wasm_filename,
            description: "Integrity test".to_string(),
            arguments: vec![],
            security: SecurityConfig::default(),
            wasm_sha256: Some("0".repeat(64)),
            signature: None,
            signer_public_key: None,
        },
        install_path,
        enabled: true,
        pinned_version: None,
        updated_at: None,
    };

    let repo = MockRepository::returning(Some(tool));
    let resolver = make_resolver(repo);
    let id = ToolId::new("dev.test.integrity").expect("id");

    let result = resolver.resolve(&id).await;

    assert!(
        matches!(result, Err(RuntimeError::IntegrityViolation { .. })),
        "expected IntegrityViolation",
    );
}
