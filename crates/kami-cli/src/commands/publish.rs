//! `kami publish` command.
//!
//! Generates a registry index entry from a local tool and prints
//! instructions for submitting it to the community registry (Homebrew model).

use std::path::Path;

use clap::Args;
use kami_config::parse_tool_manifest_file;
use kami_runtime::compute_file_hash;

use crate::output;

const REGISTRY_REPO: &str = "kami-tools/registry";

/// Publish a tool to the community registry.
#[derive(Debug, Args)]
pub struct PublishArgs {
    /// Path to the tool directory containing tool.toml.
    #[arg(default_value = ".")]
    pub path: String,
    /// GitHub source shorthand (owner/repo or owner/repo@tag).
    #[arg(long)]
    pub source: Option<String>,
    /// Only print the JSON entry without instructions.
    #[arg(long)]
    pub json: bool,
}

/// A registry index entry (matches `index.json` schema).
#[derive(Debug, serde::Serialize)]
struct RegistryEntry {
    id: String,
    name: String,
    version: String,
    description: String,
    source: String,
    wasm_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signer_public_key: Option<String>,
}

/// Executes the publish command.
pub fn execute(args: &PublishArgs) -> anyhow::Result<()> {
    let tool_path = Path::new(&args.path);
    let manifest_path = if tool_path.is_dir() {
        tool_path.join("tool.toml")
    } else {
        tool_path.to_path_buf()
    };
    if !manifest_path.exists() {
        anyhow::bail!("tool.toml not found: {}", manifest_path.display());
    }

    let manifest = parse_tool_manifest_file(&manifest_path)
        .map_err(|e| anyhow::anyhow!("manifest error: {e}"))?;

    let tool_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let wasm_path = tool_dir.join(&manifest.wasm);
    if !wasm_path.exists() {
        anyhow::bail!("WASM file not found: {}", wasm_path.display());
    }

    let wasm_hash =
        compute_file_hash(&wasm_path).map_err(|e| anyhow::anyhow!("failed to hash WASM: {e}"))?;

    let source = args
        .source
        .clone()
        .unwrap_or_else(|| format!("<owner>/{}", manifest.name));

    let entry = RegistryEntry {
        id: manifest.id.to_string(),
        name: manifest.name.clone(),
        version: manifest.version.to_string(),
        description: manifest.description.clone(),
        source: source.clone(),
        wasm_sha256: wasm_hash,
        signature: manifest.signature.clone(),
        signer_public_key: manifest.signer_public_key.clone(),
    };

    let json = serde_json::to_string_pretty(&entry)
        .map_err(|e| anyhow::anyhow!("JSON serialize error: {e}"))?;

    if args.json {
        println!("{json}");
        return Ok(());
    }

    print_publish_instructions(&json, &entry, &source);
    Ok(())
}

/// Prints step-by-step instructions for submitting to the registry.
fn print_publish_instructions(json: &str, entry: &RegistryEntry, source: &str) {
    output::print_success("Registry entry generated!\n");
    println!("Add the following to index.json in {REGISTRY_REPO}:\n");
    println!("{json}\n");
    println!("Steps to publish:");
    println!(
        "  1. Create a GitHub release with plugin.zip (tool.toml + {}.wasm)",
        entry.name
    );
    println!("  2. Tag the release: {source}");
    println!("  3. Fork https://github.com/{REGISTRY_REPO}");
    println!("  4. Add the entry above to index.json, open a PR");
    println!("\nInstall: kami install {source}");
    println!("Search:  kami search {}", entry.name);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_entry_serializes_to_json() {
        let entry = RegistryEntry {
            id: "dev.test.echo".to_string(),
            name: "echo".to_string(),
            version: "1.0.0".to_string(),
            description: "Test tool".to_string(),
            source: "kami-tools/echo@v1.0.0".to_string(),
            wasm_sha256: "abc123".to_string(),
            signature: None,
            signer_public_key: None,
        };
        let json = serde_json::to_string(&entry).expect("serialize");
        assert!(json.contains("dev.test.echo"));
        assert!(json.contains("wasm_sha256"));
    }
    #[test]
    fn registry_entry_omits_none_signature() {
        let entry = RegistryEntry {
            id: "dev.test.x".to_string(),
            name: "x".to_string(),
            version: "0.1.0".to_string(),
            description: "desc".to_string(),
            source: "org/x@v0.1.0".to_string(),
            wasm_sha256: "deadbeef".to_string(),
            signature: None,
            signer_public_key: None,
        };
        let json = serde_json::to_string(&entry).expect("serialize");
        assert!(!json.contains("\"signature\""));
    }
}
