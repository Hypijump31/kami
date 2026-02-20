//! Query integration tests for `SqliteToolRepository`.

use kami_registry::{ToolQuery, ToolRepository};
use kami_store_sqlite::SqliteToolRepository;
use kami_types::{SecurityConfig, Tool, ToolArgument, ToolId, ToolManifest, ToolVersion};

#[tokio::test]
async fn find_all_returns_all() {
    let repo = SqliteToolRepository::open_in_memory().expect("open");

    let tool1 = Tool {
        manifest: ToolManifest {
            id: ToolId::new("dev.test.alpha").expect("id"),
            name: "alpha".to_string(),
            version: ToolVersion::new(1, 0, 0),
            wasm: "alpha.wasm".to_string(),
            description: "Alpha".to_string(),
            arguments: vec![],
            security: SecurityConfig::default(),
            wasm_sha256: None,
            signature: None,
            signer_public_key: None,
        },
        install_path: "/tools/alpha".to_string(),
        enabled: true,
        pinned_version: None,
        updated_at: None,
    };
    let tool2 = Tool {
        manifest: ToolManifest {
            id: ToolId::new("dev.test.beta").expect("id"),
            name: "beta".to_string(),
            version: ToolVersion::new(2, 0, 0),
            wasm: "beta.wasm".to_string(),
            description: "Beta".to_string(),
            arguments: vec![],
            security: SecurityConfig::default(),
            wasm_sha256: None,
            signature: None,
            signer_public_key: None,
        },
        install_path: "/tools/beta".to_string(),
        enabled: true,
        pinned_version: None,
        updated_at: None,
    };

    repo.insert(&tool1).await.expect("insert 1");
    repo.insert(&tool2).await.expect("insert 2");

    let all = repo.find_all(ToolQuery::all()).await.expect("find_all");
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn find_all_with_name_filter() {
    let repo = SqliteToolRepository::open_in_memory().expect("open");

    let tool1 = Tool {
        manifest: ToolManifest {
            id: ToolId::new("dev.test.fetch-url").expect("id"),
            name: "fetch-url".to_string(),
            version: ToolVersion::new(1, 0, 0),
            wasm: "f.wasm".to_string(),
            description: "Fetch".to_string(),
            arguments: vec![],
            security: SecurityConfig::default(),
            wasm_sha256: None,
            signature: None,
            signer_public_key: None,
        },
        install_path: "/tools/fetch".to_string(),
        enabled: true,
        pinned_version: None,
        updated_at: None,
    };
    let tool2 = Tool {
        manifest: ToolManifest {
            id: ToolId::new("dev.test.calc").expect("id"),
            name: "calc".to_string(),
            version: ToolVersion::new(1, 0, 0),
            wasm: "c.wasm".to_string(),
            description: "Calculator".to_string(),
            arguments: vec![],
            security: SecurityConfig::default(),
            wasm_sha256: None,
            signature: None,
            signer_public_key: None,
        },
        install_path: "/tools/calc".to_string(),
        enabled: true,
        pinned_version: None,
        updated_at: None,
    };

    repo.insert(&tool1).await.expect("insert");
    repo.insert(&tool2).await.expect("insert");

    let results = repo
        .find_all(ToolQuery::all().with_name("fetch"))
        .await
        .expect("find");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].manifest.name, "fetch-url");
}

#[tokio::test]
async fn find_all_with_arguments() {
    let repo = SqliteToolRepository::open_in_memory().expect("open");

    let tool = Tool {
        manifest: ToolManifest {
            id: ToolId::new("dev.test.with-args").expect("id"),
            name: "with-args".to_string(),
            version: ToolVersion::new(1, 0, 0),
            wasm: "args.wasm".to_string(),
            description: "Tool with arguments".to_string(),
            arguments: vec![ToolArgument {
                name: "url".to_string(),
                arg_type: "string".to_string(),
                description: "URL to fetch".to_string(),
                required: true,
                default: None,
            }],
            security: SecurityConfig::default(),
            wasm_sha256: None,
            signature: None,
            signer_public_key: None,
        },
        install_path: "/tools/with-args".to_string(),
        enabled: true,
        pinned_version: None,
        updated_at: None,
    };

    repo.insert(&tool).await.expect("insert");
    let found = repo
        .find_by_id(&tool.manifest.id)
        .await
        .expect("find")
        .expect("exists");
    assert_eq!(found.manifest.arguments.len(), 1);
    assert_eq!(found.manifest.arguments[0].name, "url");
}
