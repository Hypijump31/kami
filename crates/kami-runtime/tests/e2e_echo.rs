//! End-to-end test: real WASM component through the full KAMI runtime.
//!
//! Verifies: resolve → compile → instantiate → execute → result.

use std::sync::Arc;

use kami_config::parse_tool_manifest;
use kami_registry::ToolRepository;
use kami_runtime::{compute_file_hash, KamiRuntime, RuntimeConfig};
use kami_store_sqlite::SqliteToolRepository;
use kami_types::Tool;

/// Returns the echo-tool fixture directory.
fn echo_tool_dir() -> std::path::PathBuf {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    crate_root
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .join("tests")
        .join("fixtures")
        .join("echo-tool")
}

/// Registers the echo tool in an in-memory SQLite repository.
async fn register_echo(repo: &SqliteToolRepository) -> kami_types::ToolId {
    let dir = echo_tool_dir();
    let content = std::fs::read_to_string(dir.join("tool.toml")).expect("read tool.toml");
    let mut manifest = parse_tool_manifest(&content).expect("parse manifest");

    let wasm_path = dir.join(&manifest.wasm);
    assert!(wasm_path.exists(), "echo_tool.wasm missing");

    let hash = compute_file_hash(&wasm_path).expect("hash");
    manifest.wasm_sha256 = Some(hash);

    let id = manifest.id.clone();
    let tool = Tool {
        manifest,
        install_path: dir.display().to_string(),
        enabled: true,
        pinned_version: None,
        updated_at: None,
    };

    repo.insert(&tool).await.expect("insert");
    id
}

#[tokio::test]
async fn echo_tool_returns_input() {
    let repo = Arc::new(SqliteToolRepository::open_in_memory().expect("repo"));
    let tool_id = register_echo(&repo).await;
    let rt = KamiRuntime::new(RuntimeConfig::default(), repo).expect("rt");

    let input = r#"{"message":"hello from KAMI"}"#;
    let result = rt.execute(&tool_id, input).await.expect("exec");

    assert!(result.success, "echo should succeed");
    assert_eq!(result.content, input);
    assert!(result.fuel_consumed > 0);
    assert!(result.duration_ms < 10_000);
}

#[tokio::test]
async fn echo_tool_with_empty_input() {
    let repo = Arc::new(SqliteToolRepository::open_in_memory().expect("repo"));
    let tool_id = register_echo(&repo).await;
    let rt = KamiRuntime::new(RuntimeConfig::default(), repo).expect("rt");

    let result = rt.execute(&tool_id, "{}").await.expect("exec");

    assert!(result.success);
    assert_eq!(result.content, "{}");
}
