//! Integration tests for `KamiRuntime` orchestrator.

use std::sync::Arc;

use async_trait::async_trait;

use kami_registry::{RepositoryError, ToolQuery, ToolRepository};
use kami_runtime::{KamiRuntime, RuntimeConfig, RuntimeError};
use kami_types::{Tool, ToolId};

// ---------------------------------------------------------------------------
// Minimal mock repository
// ---------------------------------------------------------------------------

struct EmptyRepository;

#[async_trait]
impl ToolRepository for EmptyRepository {
    async fn find_by_id(&self, _id: &ToolId) -> Result<Option<Tool>, RepositoryError> {
        Ok(None)
    }

    async fn find_all(&self, _query: ToolQuery) -> Result<Vec<Tool>, RepositoryError> {
        Ok(vec![])
    }

    async fn insert(&self, _tool: &Tool) -> Result<(), RepositoryError> {
        Err(RepositoryError::Storage {
            message: "read-only".to_string(),
        })
    }

    async fn update(&self, _tool: &Tool) -> Result<(), RepositoryError> {
        Err(RepositoryError::Storage {
            message: "read-only".to_string(),
        })
    }

    async fn delete(&self, _id: &ToolId) -> Result<bool, RepositoryError> {
        Ok(false)
    }
}

fn make_runtime() -> KamiRuntime {
    let config = RuntimeConfig {
        cache_size: 4,
        max_concurrent: 2,
        epoch_interruption: true,
    };
    KamiRuntime::new(config, Arc::new(EmptyRepository)).expect("runtime")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn orchestrator_tool_not_found() {
    let runtime = make_runtime();
    let id = ToolId::new("dev.test.unknown").expect("id");

    let result = runtime.execute(&id, "{}").await;

    assert!(
        matches!(result, Err(RuntimeError::ToolNotFound { .. })),
        "expected ToolNotFound",
    );
}

#[tokio::test]
async fn orchestrator_invalidate_cache_does_not_panic() {
    let runtime = make_runtime();
    let id = ToolId::new("dev.test.cache").expect("id");

    // Invalidating a tool that was never cached should be a no-op.
    runtime.invalidate_cache(&id).await;
}

#[tokio::test]
async fn orchestrator_scheduler_reports_permits() {
    let runtime = make_runtime();
    let scheduler = runtime.scheduler();

    assert_eq!(scheduler.max_concurrent(), 2);
    assert_eq!(scheduler.available_permits(), 2);
}

#[tokio::test]
async fn orchestrator_default_config_is_valid() {
    let config = RuntimeConfig::default();
    let result = KamiRuntime::new(config, Arc::new(EmptyRepository));
    assert!(
        result.is_ok(),
        "default RuntimeConfig should create a valid runtime"
    );
}

#[tokio::test]
async fn orchestrator_metrics_track_failed_execution() {
    let runtime = make_runtime();
    let id = ToolId::new("dev.test.metrics").expect("id");

    let _ = runtime.execute(&id, "{}").await;

    let snap = runtime.metrics().snapshot();
    assert_eq!(snap.total_executions, 1);
    assert_eq!(snap.failed_executions, 1);
    assert_eq!(snap.successful_executions, 0);
    assert_eq!(snap.cache_misses, 1);
    assert_eq!(snap.cache_hits, 0);
}
