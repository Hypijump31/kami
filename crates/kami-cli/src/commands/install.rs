//! `kami install` command.
//!
//! Parses a tool.toml manifest, validates it, and registers the tool
//! in the SQLite registry.

use std::path::Path;

use clap::Args;

use kami_registry::ToolRepository;
use kami_store_sqlite::SqliteToolRepository;
use kami_types::{parse_tool_manifest_file, Tool};

use crate::output;

/// Install a WASM tool into the registry.
#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Path to the tool directory containing tool.toml.
    pub path: String,
    /// Database path (defaults to .kami/registry.db).
    #[arg(long)]
    pub db: Option<String>,
}

/// Executes the install command.
pub fn execute(args: &InstallArgs) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(install_async(args))
}

async fn install_async(args: &InstallArgs) -> anyhow::Result<()> {
    let tool_path = Path::new(&args.path);

    // Resolve tool.toml location
    let manifest_path = if tool_path.is_dir() {
        tool_path.join("tool.toml")
    } else if tool_path.extension().is_some_and(|e| e == "toml") {
        tool_path.to_path_buf()
    } else {
        anyhow::bail!(
            "expected a directory containing tool.toml or a .toml file"
        );
    };

    if !manifest_path.exists() {
        anyhow::bail!(
            "tool.toml not found: {}",
            manifest_path.display()
        );
    }

    tracing::info!(
        path = %manifest_path.display(),
        "Parsing tool manifest"
    );

    // 1. Parse manifest
    let manifest = parse_tool_manifest_file(&manifest_path)
        .map_err(|e| anyhow::anyhow!("manifest error: {e}"))?;

    // 2. Verify WASM file exists
    let tool_dir =
        manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let wasm_path = tool_dir.join(&manifest.wasm);
    if !wasm_path.exists() {
        anyhow::bail!(
            "WASM file not found: {} (referenced in tool.toml)",
            wasm_path.display()
        );
    }

    // 3. Open registry
    let db_path =
        args.db.clone().unwrap_or_else(output::default_db_path);
    if let Some(parent) = Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let repo = SqliteToolRepository::open(&db_path)
        .map_err(|e| anyhow::anyhow!("registry error: {e}"))?;

    // 4. Check for duplicates
    if let Some(existing) = repo
        .find_by_id(&manifest.id)
        .await
        .map_err(|e| anyhow::anyhow!("registry error: {e}"))?
    {
        anyhow::bail!(
            "tool {} v{} is already installed",
            existing.manifest.id,
            existing.manifest.version,
        );
    }

    // 5. Register the tool
    let install_path = tool_dir
        .canonicalize()
        .unwrap_or_else(|_| tool_dir.to_path_buf())
        .display()
        .to_string();

    let tool = Tool {
        manifest,
        install_path,
        enabled: true,
    };

    repo.insert(&tool)
        .await
        .map_err(|e| anyhow::anyhow!("insert error: {e}"))?;

    output::print_success(&format!(
        "Installed {} v{}",
        tool.manifest.id, tool.manifest.version
    ));

    Ok(())
}
