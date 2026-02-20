//! `kami install` command.
//!
//! Installs tools from local paths, URLs, or GitHub shorthand.

use std::path::{Path, PathBuf};

use clap::Args;
use kami_config::parse_tool_manifest_file;
use kami_runtime::compute_file_hash;
use kami_types::Tool;

use crate::{commands::download, output, shared};

/// Install a WASM tool into the registry.
#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Source: local path, URL, or GitHub shorthand (owner/repo@tag).
    pub source: String,
    /// Database path (defaults to .kami/registry.db).
    #[arg(long)]
    pub db: Option<String>,
}

/// Executes the install command.
pub async fn execute(args: &InstallArgs) -> anyhow::Result<()> {
    let tool_dir = resolve_source(&args.source).await?;
    install_from_dir(&tool_dir, &args.db).await
}

/// Resolves the source to a local directory (downloading if remote).
async fn resolve_source(source: &str) -> anyhow::Result<PathBuf> {
    if download::is_url(source) {
        download_to_plugins(source).await
    } else if download::is_github_shorthand(source) {
        let url = download::github_release_url(source);
        download_to_plugins(&url).await
    } else {
        Ok(PathBuf::from(source))
    }
}

/// Downloads a remote archive and stores it in the plugins directory.
async fn download_to_plugins(url: &str) -> anyhow::Result<PathBuf> {
    let tmp_name = format!("_downloading_{}", std::process::id());
    let tmp_dir = shared::plugins_dir().join(&tmp_name);
    download::download_and_extract(url, &tmp_dir).await?;

    let manifest = parse_tool_manifest_file(&tmp_dir.join("tool.toml")).map_err(|e| {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        anyhow::anyhow!("downloaded archive has no valid tool.toml: {e}")
    })?;

    let final_dir = shared::plugins_dir().join(manifest.id.as_str());
    if final_dir.exists() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        anyhow::bail!("plugin {} already downloaded", manifest.id);
    }
    std::fs::rename(&tmp_dir, &final_dir)?;
    output::print_info(&format!("Plugin stored in {}", final_dir.display()));
    Ok(final_dir)
}

/// Installs a tool from a local directory containing tool.toml.
async fn install_from_dir(path: &Path, db: &Option<String>) -> anyhow::Result<()> {
    let manifest_path = if path.is_dir() {
        path.join("tool.toml")
    } else if path.extension().is_some_and(|e| e == "toml") {
        path.to_path_buf()
    } else {
        anyhow::bail!("expected a directory or .toml file");
    };
    if !manifest_path.exists() {
        anyhow::bail!("tool.toml not found: {}", manifest_path.display());
    }
    tracing::info!(path = %manifest_path.display(), "Parsing tool manifest");

    let mut manifest = parse_tool_manifest_file(&manifest_path)
        .map_err(|e| anyhow::anyhow!("manifest error: {e}"))?;

    let tool_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let wasm_path = tool_dir.join(&manifest.wasm);
    if !wasm_path.exists() {
        anyhow::bail!("WASM file not found: {}", wasm_path.display());
    }

    let wasm_hash = compute_file_hash(&wasm_path)
        .map_err(|e| anyhow::anyhow!("failed to hash WASM file: {e}"))?;
    tracing::info!(hash = %wasm_hash, "SHA-256 computed");
    manifest.wasm_sha256 = Some(wasm_hash);

    let repo = shared::open_repository(db)?;
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

    let install_path = tool_dir
        .canonicalize()
        .unwrap_or_else(|_| tool_dir.to_path_buf())
        .display()
        .to_string();

    let tool = Tool {
        manifest,
        install_path,
        enabled: true,
        pinned_version: None,
        updated_at: None,
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
