//! CRUD integration tests for `SqliteToolRepository`.

use kami_registry::ToolRepository;
use kami_store_sqlite::SqliteToolRepository;
use kami_types::{SecurityConfig, Tool, ToolId, ToolManifest, ToolVersion};

fn sample_tool() -> Tool {
    Tool {
        manifest: ToolManifest {
            id: ToolId::new("dev.test.sample").expect("id"),
            name: "sample".to_string(),
            version: ToolVersion::new(1, 0, 0),
            wasm: "sample.wasm".to_string(),
            description: "A sample tool".to_string(),
            arguments: vec![],
            security: SecurityConfig::default(),
            wasm_sha256: None,
            signature: None,
            signer_public_key: None,
        },
        install_path: "/tools/sample".to_string(),
        enabled: true,
        pinned_version: None,
        updated_at: None,
    }
}

#[tokio::test]
async fn insert_and_find_by_id() {
    let repo = SqliteToolRepository::open_in_memory().expect("open");
    let tool = sample_tool();

    repo.insert(&tool).await.expect("insert");

    let found = repo.find_by_id(&tool.manifest.id).await.expect("find");
    let found = found.expect("should exist");
    assert_eq!(found.manifest.id.as_str(), "dev.test.sample");
    assert_eq!(found.manifest.name, "sample");
    assert!(found.enabled);
}

#[tokio::test]
async fn find_nonexistent_returns_none() {
    let repo = SqliteToolRepository::open_in_memory().expect("open");
    let id = ToolId::new("dev.test.nope").expect("id");
    let found = repo.find_by_id(&id).await.expect("find");
    assert!(found.is_none());
}

#[tokio::test]
async fn insert_duplicate_fails() {
    let repo = SqliteToolRepository::open_in_memory().expect("open");
    let tool = sample_tool();

    repo.insert(&tool).await.expect("first insert");
    let err = repo.insert(&tool).await;
    assert!(err.is_err());
}

#[tokio::test]
async fn delete_existing_tool() {
    let repo = SqliteToolRepository::open_in_memory().expect("open");
    let tool = sample_tool();

    repo.insert(&tool).await.expect("insert");
    let deleted = repo.delete(&tool.manifest.id).await.expect("delete");
    assert!(deleted);

    let found = repo.find_by_id(&tool.manifest.id).await.expect("find");
    assert!(found.is_none());
}

#[tokio::test]
async fn delete_nonexistent_returns_false() {
    let repo = SqliteToolRepository::open_in_memory().expect("open");
    let id = ToolId::new("dev.test.nope").expect("id");
    let deleted = repo.delete(&id).await.expect("delete");
    assert!(!deleted);
}

#[tokio::test]
async fn update_existing_tool() {
    let repo = SqliteToolRepository::open_in_memory().expect("open");
    let mut tool = sample_tool();
    repo.insert(&tool).await.expect("insert");

    tool.manifest.version = ToolVersion::new(2, 0, 0);
    tool.pinned_version = Some("2.0.0".to_string());
    tool.updated_at = Some("1700000000".to_string());
    repo.update(&tool).await.expect("update");

    let found = repo.find_by_id(&tool.manifest.id).await.expect("find");
    let found = found.expect("exists");
    assert_eq!(found.manifest.version, ToolVersion::new(2, 0, 0));
    assert_eq!(found.pinned_version.as_deref(), Some("2.0.0"));
    assert_eq!(found.updated_at.as_deref(), Some("1700000000"));
}

#[tokio::test]
async fn update_nonexistent_returns_not_found() {
    let repo = SqliteToolRepository::open_in_memory().expect("open");
    let tool = sample_tool();
    let err = repo.update(&tool).await;
    assert!(err.is_err());
}
