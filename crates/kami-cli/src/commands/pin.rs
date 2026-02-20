//! `kami pin` command.
//!
//! Pins a tool to its current version (or a specified version),
//! preventing `kami update` from changing it. Use `--unpin` to remove.

use clap::Args;

use crate::{output, shared};

/// Pin or unpin a tool version.
#[derive(Debug, Args)]
pub struct PinArgs {
    /// Tool ID to pin/unpin.
    pub tool_id: String,
    /// Version to pin to (defaults to the currently installed version).
    pub version: Option<String>,
    /// Remove the version pin.
    #[arg(long)]
    pub unpin: bool,
    /// Database path (defaults to .kami/registry.db).
    #[arg(long)]
    pub db: Option<String>,
}

/// Executes the pin command.
pub async fn execute(args: &PinArgs) -> anyhow::Result<()> {
    let repo = shared::open_repository(&args.db)?;
    let id: kami_types::ToolId = args.tool_id.parse().map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut tool = repo
        .find_by_id(&id)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .ok_or_else(|| anyhow::anyhow!("tool not found: {id}"))?;

    if args.unpin {
        tool.pinned_version = None;
        repo.update(&tool)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        output::print_success(&format!("Unpinned {id}"));
        return Ok(());
    }

    let pin_ver = args
        .version
        .clone()
        .unwrap_or_else(|| tool.manifest.version.to_string());

    tool.pinned_version = Some(pin_ver.clone());
    repo.update(&tool)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    output::print_success(&format!("Pinned {id} to v{pin_ver}"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_args_default_unpin_is_false() {
        let args = PinArgs {
            tool_id: "dev.test.x".to_string(),
            version: None,
            unpin: false,
            db: None,
        };
        assert!(!args.unpin);
        assert!(args.version.is_none());
    }

    #[tokio::test]
    async fn pin_missing_tool_fails() {
        let dir = tempfile::tempdir().expect("tmp");
        let db = dir.path().join("pin.db").to_str().expect("u").to_string();
        let args = PinArgs {
            tool_id: "dev.test.nope".into(),
            version: None,
            unpin: false,
            db: Some(db),
        };
        assert!(execute(&args).await.is_err());
    }

    #[tokio::test]
    async fn pin_and_unpin_tool() {
        use kami_types::*;
        let dir = tempfile::tempdir().expect("tmp");
        let db = dir.path().join("pin2.db").to_str().expect("u").to_string();
        let repo = crate::shared::open_repository(&Some(db.clone())).expect("r");
        let tool = Tool {
            manifest: ToolManifest {
                id: ToolId::new("dev.t.pn").expect("id"),
                name: "pn".into(),
                version: ToolVersion::new(1, 0, 0),
                wasm: "pn.wasm".into(),
                description: "pin test".into(),
                arguments: vec![],
                security: SecurityConfig::default(),
                wasm_sha256: None,
                signature: None,
                signer_public_key: None,
            },
            install_path: "/pn".into(),
            enabled: true,
            pinned_version: None,
            updated_at: None,
        };
        repo.insert(&tool).await.expect("ins");
        let pin_args = PinArgs {
            tool_id: "dev.t.pn".into(),
            version: Some("1.0.0".into()),
            unpin: false,
            db: Some(db.clone()),
        };
        execute(&pin_args).await.expect("pin");
        let unpin = PinArgs {
            tool_id: "dev.t.pn".into(),
            version: None,
            unpin: true,
            db: Some(db),
        };
        execute(&unpin).await.expect("unpin");
    }
}
