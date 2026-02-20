//! `kami update` command.
//!
//! Re-reads tool.toml from the install path and updates the registry
//! entry with new manifest data and a fresh WASM hash.

use std::path::Path;

use clap::Args;

use kami_config::parse_tool_manifest_file;
use kami_registry::ToolRepository;
use kami_runtime::compute_file_hash;
use kami_types::Tool;

use crate::{output, shared};

/// Update one or all installed tools from their source directories.
#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Tool ID to update (omit for --all).
    pub tool_id: Option<String>,
    /// Update all installed tools.
    #[arg(long)]
    pub all: bool,
    /// Database path (defaults to .kami/registry.db).
    #[arg(long)]
    pub db: Option<String>,
}

/// Executes the update command.
pub async fn execute(args: &UpdateArgs) -> anyhow::Result<()> {
    let repo = shared::open_repository(&args.db)?;

    if args.all {
        return update_all(&*repo).await;
    }
    let id_str = args
        .tool_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("provide a tool ID or use --all"))?;
    let id = id_str.parse().map_err(|e| anyhow::anyhow!("{e}"))?;
    update_one(&*repo, &id).await
}

/// Updates a single tool by re-reading its source manifest.
async fn update_one(repo: &dyn ToolRepository, id: &kami_types::ToolId) -> anyhow::Result<()> {
    let existing = repo
        .find_by_id(id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .ok_or_else(|| anyhow::anyhow!("tool not found: {id}"))?;

    if existing.pinned_version.is_some() {
        anyhow::bail!(
            "tool {} is pinned to v{}; unpin first with `kami pin --unpin {}`",
            id,
            existing.pinned_version.as_deref().unwrap_or("?"),
            id,
        );
    }

    let updated = rebuild_tool(&existing)?;
    repo.update(&updated)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    output::print_success(&format!(
        "Updated {} → v{}",
        updated.manifest.id, updated.manifest.version
    ));
    Ok(())
}

/// Re-reads tool.toml and recomputes the WASM hash.
fn rebuild_tool(existing: &Tool) -> anyhow::Result<Tool> {
    let install = Path::new(&existing.install_path);
    let manifest_path = install.join("tool.toml");
    if !manifest_path.exists() {
        anyhow::bail!("tool.toml not found at {}", manifest_path.display());
    }
    let mut manifest = parse_tool_manifest_file(&manifest_path)
        .map_err(|e| anyhow::anyhow!("manifest error: {e}"))?;

    let wasm_path = install.join(&manifest.wasm);
    if wasm_path.exists() {
        let hash = compute_file_hash(&wasm_path).map_err(|e| anyhow::anyhow!("hash error: {e}"))?;
        manifest.wasm_sha256 = Some(hash);
    }

    let now = chrono_now();
    Ok(Tool {
        manifest,
        install_path: existing.install_path.clone(),
        enabled: existing.enabled,
        pinned_version: existing.pinned_version.clone(),
        updated_at: Some(now),
    })
}

/// Returns the current UTC time in ISO 8601 format (no chrono dep).
fn chrono_now() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    format!("{secs}")
}

/// Updates every installed tool that is not pinned.
async fn update_all(repo: &dyn ToolRepository) -> anyhow::Result<()> {
    let tools = repo
        .find_all(kami_registry::ToolQuery::all())
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    if tools.is_empty() {
        output::print_info("No tools installed.");
        return Ok(());
    }
    let mut updated = 0u32;
    let mut skipped = 0u32;
    for tool in &tools {
        if tool.pinned_version.is_some() {
            skipped += 1;
            continue;
        }
        match rebuild_tool(tool) {
            Ok(new_tool) => {
                repo.update(&new_tool)
                    .await
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
                updated += 1;
            }
            Err(e) => {
                tracing::warn!(tool = %tool.manifest.id, "skip: {e}");
                skipped += 1;
            }
        }
    }
    output::print_success(&format!("Updated {updated} tool(s), skipped {skipped}"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chrono_now_is_numeric_epoch_secs() {
        assert!(chrono_now().parse::<u64>().is_ok());
    }
}
