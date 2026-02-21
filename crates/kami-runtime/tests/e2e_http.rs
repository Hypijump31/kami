//! End-to-end tests: http-fetch-tool WASM component with WASI HTTP outgoing.
//!
//! Verifies network allow-list enforcement through the full runtime pipeline.

use std::sync::Arc;

use kami_registry::ToolRepository;
use kami_runtime::{compute_file_hash, KamiRuntime, RuntimeConfig};
use kami_store_sqlite::SqliteToolRepository;
use kami_types::{ResourceLimits, SecurityConfig, Tool, ToolId, ToolManifest, ToolVersion};

/// Returns the path to the compiled http_fetch_tool.wasm.
fn wasm_path() -> std::path::PathBuf {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");
    root.join("tests/fixtures/http-fetch-tool/target/wasm32-wasip2/debug/http_fetch_tool.wasm")
}

/// Registers http-fetch-tool with the given `net_allow_list`.
async fn register_http_fetch(repo: &SqliteToolRepository, net_allow_list: Vec<String>) -> ToolId {
    let path = wasm_path();
    assert!(
        path.exists(),
        "missing: {}\nRun `cargo build --target wasm32-wasip2` in tests/fixtures/http-fetch-tool",
        path.display()
    );
    let hash = compute_file_hash(&path).expect("hash");
    let install_path = path.parent().expect("parent").display().to_string();
    let id = ToolId::new("dev.kami.http-fetch").expect("id");
    let tool = Tool {
        manifest: ToolManifest {
            id: id.clone(),
            name: "http-fetch".to_string(),
            version: ToolVersion::new(0, 1, 0),
            wasm: "http_fetch_tool.wasm".to_string(),
            description: "Fetches a URL via WASI HTTP".to_string(),
            arguments: vec![],
            security: SecurityConfig {
                net_allow_list,
                limits: ResourceLimits {
                    max_fuel: 50_000_000,
                    max_execution_ms: 10_000,
                    ..ResourceLimits::default()
                },
                ..SecurityConfig::default()
            },
            wasm_sha256: Some(hash),
            signature: None,
            signer_public_key: None,
        },
        install_path,
        enabled: true,
        pinned_version: None,
        updated_at: None,
    };
    repo.insert(&tool).await.expect("insert");
    id
}

/// Spawns a minimal HTTP/1.1 server on 127.0.0.1:0.  Returns the bound port.
async fn spawn_http_server() -> u16 {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf).await;
                let body = b"kami-test-response";
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\nkami-test-response",
                    body.len(),
                );
                let _ = stream.write_all(response.as_bytes()).await;
            });
        }
    });
    port
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn http_fetch_blocked_by_empty_allow_list() {
    let repo = Arc::new(SqliteToolRepository::open_in_memory().expect("repo"));
    let id = register_http_fetch(&repo, vec![]).await;
    let rt = KamiRuntime::new(RuntimeConfig::default(), repo).expect("rt");

    let result = rt
        .execute(&id, r#"{"url":"https://example.com"}"#)
        .await
        .expect("no runtime error");

    assert!(!result.success, "empty allow-list must block HTTP");
    assert!(
        result.content.contains("denied") || result.content.contains("refused"),
        "unexpected error message: {}",
        result.content,
    );
}

#[tokio::test]
async fn http_fetch_blocked_for_unlisted_host() {
    let repo = Arc::new(SqliteToolRepository::open_in_memory().expect("repo"));
    let id = register_http_fetch(&repo, vec!["allowed.example.com".to_string()]).await;
    let rt = KamiRuntime::new(RuntimeConfig::default(), repo).expect("rt");

    let result = rt
        .execute(&id, r#"{"url":"https://other.example.com"}"#)
        .await
        .expect("no runtime error");

    assert!(!result.success, "unlisted host must be blocked");
}

#[tokio::test]
async fn http_fetch_allowed_for_listed_host() {
    let port = spawn_http_server().await;
    let repo = Arc::new(SqliteToolRepository::open_in_memory().expect("repo"));
    let id = register_http_fetch(&repo, vec!["127.0.0.1".to_string()]).await;
    let rt = KamiRuntime::new(RuntimeConfig::default(), repo).expect("rt");

    let url = format!("http://127.0.0.1:{port}/");
    let result = rt
        .execute(&id, &format!(r#"{{"url":"{url}"}}"#))
        .await
        .expect("no runtime error");

    assert!(
        result.success,
        "listed host must be allowed; error: {}",
        result.content
    );
    assert!(
        result.content.contains("kami-test-response"),
        "expected body in response, got: {}",
        result.content,
    );
}

#[tokio::test]
async fn http_fetch_wildcard_allow_list() {
    let port = spawn_http_server().await;
    let repo = Arc::new(SqliteToolRepository::open_in_memory().expect("repo"));
    // Wildcard does NOT match 127.0.0.1 (IP, not a domain)
    let id = register_http_fetch(&repo, vec!["*.localhost".to_string()]).await;
    let rt = KamiRuntime::new(RuntimeConfig::default(), repo).expect("rt");

    let url = format!("http://127.0.0.1:{port}/");
    let result = rt
        .execute(&id, &format!(r#"{{"url":"{url}"}}"#))
        .await
        .expect("no runtime error");

    assert!(!result.success, "IP must not match *.localhost wildcard");
}
