//! `kami inspect` command.
//!
//! Displays detailed information about an installed tool.

use clap::Args;

use kami_types::ToolId;

use crate::{output, shared};

/// Inspect a tool's manifest and capabilities.
#[derive(Debug, Args)]
pub struct InspectArgs {
    /// Tool ID (reverse-domain notation).
    pub tool: String,
    /// Database path (defaults to .kami/registry.db).
    #[arg(long)]
    pub db: Option<String>,
}

/// Executes the inspect command.
pub async fn execute(args: &InspectArgs) -> anyhow::Result<()> {
    let repo = shared::open_repository(&args.db)?;

    let id = ToolId::new(&args.tool).map_err(|e| anyhow::anyhow!("invalid tool ID: {e}"))?;

    let tool = repo
        .find_by_id(&id)
        .await
        .map_err(|e| anyhow::anyhow!("query error: {e}"))?;

    let tool = match tool {
        Some(t) => t,
        None => {
            output::print_error(&format!("tool not found: {}", args.tool));
            return Ok(());
        }
    };

    let m = &tool.manifest;
    let sec = &m.security;

    println!("Tool: {} v{}", m.id, m.version);
    println!("Name: {}", m.name);
    println!("Description: {}", m.description);
    println!("WASM: {}", m.wasm);
    println!("Install path: {}", tool.install_path);
    println!(
        "Status: {}",
        if tool.enabled { "enabled" } else { "disabled" }
    );

    println!("\nSecurity:");
    println!("  Filesystem: {:?}", sec.fs_access);
    if sec.net_allow_list.is_empty() {
        println!("  Network: deny-all");
    } else {
        println!("  Network: {}", sec.net_allow_list.join(", "));
    }
    if sec.env_allow_list.is_empty() {
        println!("  Env vars: deny-all");
    } else {
        println!("  Env vars: {}", sec.env_allow_list.join(", "));
    }

    println!("\nResource limits:");
    println!("  Memory: {} MB", sec.limits.max_memory_mb);
    println!("  Timeout: {} ms", sec.limits.max_execution_ms);
    println!("  Fuel: {}", sec.limits.max_fuel);

    if !m.arguments.is_empty() {
        println!("\nArguments:");
        for arg in &m.arguments {
            let req = if arg.required { "required" } else { "optional" };
            println!(
                "  {} ({}, {}): {}",
                arg.name, arg.arg_type, req, arg.description
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn inspect_missing_tool() {
        let dir = tempfile::tempdir().expect("tmp");
        let db = dir.path().join("insp.db").to_str().expect("u").to_string();
        let args = InspectArgs {
            tool: "dev.test.missing".into(),
            db: Some(db),
        };
        // Returns Ok but prints "tool not found"
        assert!(execute(&args).await.is_ok());
    }

    #[tokio::test]
    async fn inspect_invalid_id() {
        let dir = tempfile::tempdir().expect("tmp");
        let db = dir.path().join("insp2.db").to_str().expect("u").to_string();
        let args = InspectArgs {
            tool: "bad".into(),
            db: Some(db),
        };
        assert!(execute(&args).await.is_err());
    }

    #[tokio::test]
    async fn inspect_existing_tool() {
        use kami_types::*;
        let dir = tempfile::tempdir().expect("tmp");
        let db_path = dir.path().join("insp3.db");
        let db = db_path.to_str().expect("u").to_string();
        let repo = crate::shared::open_repository(&Some(db.clone())).expect("r");
        let tool = Tool {
            manifest: ToolManifest {
                id: ToolId::new("dev.t.x").expect("id"),
                name: "x".into(),
                version: ToolVersion::new(1, 0, 0),
                wasm: "x.wasm".into(),
                description: "x tool".into(),
                arguments: vec![],
                security: SecurityConfig::default(),
                wasm_sha256: None,
                signature: None,
                signer_public_key: None,
            },
            install_path: "/x".into(),
            enabled: true,
            pinned_version: None,
            updated_at: None,
        };
        repo.insert(&tool).await.expect("ins");
        let args = InspectArgs {
            tool: "dev.t.x".into(),
            db: Some(db),
        };
        assert!(execute(&args).await.is_ok());
    }
}
