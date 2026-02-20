//! Additional repository tests for uncovered query_builder branches.

use kami_registry::{ToolQuery, ToolRepository};
use kami_store_sqlite::SqliteToolRepository;
use kami_types::{SecurityConfig, Tool, ToolId, ToolManifest, ToolVersion};

fn tool(id: &str, name: &str, enabled: bool) -> Tool {
    Tool {
        manifest: ToolManifest {
            id: ToolId::new(id).expect("id"),
            name: name.to_string(),
            version: ToolVersion::new(1, 0, 0),
            wasm: format!("{name}.wasm"),
            description: format!("{name} tool"),
            arguments: vec![],
            security: SecurityConfig::default(),
            wasm_sha256: Some("abc123".to_string()),
            signature: None,
            signer_public_key: None,
        },
        install_path: format!("/tools/{name}"),
        enabled,
        pinned_version: None,
        updated_at: None,
    }
}

#[tokio::test]
async fn find_all_enabled_only() {
    let repo = SqliteToolRepository::open_in_memory().expect("db");
    repo.insert(&tool("dev.a.one", "one", true))
        .await
        .expect("i");
    repo.insert(&tool("dev.a.two", "two", false))
        .await
        .expect("i");
    repo.insert(&tool("dev.a.three", "three", true))
        .await
        .expect("i");
    let q = ToolQuery {
        enabled_only: true,
        ..Default::default()
    };
    let results = repo.find_all(q).await.expect("find");
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|t| t.enabled));
}

#[tokio::test]
async fn find_all_with_limit() {
    let repo = SqliteToolRepository::open_in_memory().expect("db");
    repo.insert(&tool("dev.b.one", "aa", true))
        .await
        .expect("i");
    repo.insert(&tool("dev.b.two", "bb", true))
        .await
        .expect("i");
    repo.insert(&tool("dev.b.three", "cc", true))
        .await
        .expect("i");
    let q = ToolQuery::all().with_limit(2);
    let results = repo.find_all(q).await.expect("find");
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn find_all_with_offset() {
    let repo = SqliteToolRepository::open_in_memory().expect("db");
    repo.insert(&tool("dev.c.one", "da", true))
        .await
        .expect("i");
    repo.insert(&tool("dev.c.two", "db", true))
        .await
        .expect("i");
    repo.insert(&tool("dev.c.three", "dc", true))
        .await
        .expect("i");
    let q = ToolQuery {
        limit: Some(100),
        offset: Some(1),
        ..Default::default()
    };
    let results = repo.find_all(q).await.expect("find");
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn find_all_limit_and_offset() {
    let repo = SqliteToolRepository::open_in_memory().expect("db");
    repo.insert(&tool("dev.d.one", "ea", true))
        .await
        .expect("i");
    repo.insert(&tool("dev.d.two", "eb", true))
        .await
        .expect("i");
    repo.insert(&tool("dev.d.three", "ec", true))
        .await
        .expect("i");
    let q = ToolQuery {
        limit: Some(1),
        offset: Some(1),
        ..Default::default()
    };
    let results = repo.find_all(q).await.expect("find");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].manifest.name, "eb");
}

#[tokio::test]
async fn open_with_temp_file() {
    let dir = std::env::temp_dir().join("kami_test_store");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("test.db");
    let _ = std::fs::remove_file(&path);
    let repo = SqliteToolRepository::open(path.to_str().expect("p")).expect("open");
    repo.insert(&tool("dev.e.one", "f", true)).await.expect("i");
    let found = repo
        .find_by_id(&ToolId::new("dev.e.one").expect("id"))
        .await
        .expect("find");
    assert!(found.is_some());
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn insert_with_sha256() {
    let repo = SqliteToolRepository::open_in_memory().expect("db");
    repo.insert(&tool("dev.f.hash", "hashed", true))
        .await
        .expect("i");
    let found = repo
        .find_by_id(&ToolId::new("dev.f.hash").expect("id"))
        .await
        .expect("find")
        .expect("exists");
    assert_eq!(found.manifest.wasm_sha256.as_deref(), Some("abc123"));
}
