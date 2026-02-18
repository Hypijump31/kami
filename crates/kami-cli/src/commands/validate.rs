//! `kami validate` command.
//!
//! Validates a tool.toml manifest and checks that the referenced
//! WASM file exists. Useful before `kami install`.

use std::path::Path;

use clap::Args;

use kami_sandbox::validate_security_config;
use kami_types::parse_tool_manifest_file;

use crate::output;

/// Validate a tool.toml manifest without installing.
#[derive(Debug, Args)]
pub struct ValidateArgs {
    /// Path to tool directory or tool.toml file.
    pub path: String,
}

/// Executes the validate command.
pub fn execute(args: &ValidateArgs) -> anyhow::Result<()> {
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

    // 1. Parse manifest
    println!("Checking manifest...");
    let manifest = parse_tool_manifest_file(&manifest_path)
        .map_err(|e| anyhow::anyhow!("manifest error: {e}"))?;

    println!(
        "  ID:          {}",
        manifest.id
    );
    println!(
        "  Name:        {}",
        manifest.name
    );
    println!(
        "  Version:     {}",
        manifest.version
    );
    println!(
        "  Description: {}",
        manifest.description
    );

    // 2. Validate security config
    println!("Checking security config...");
    validate_security_config(&manifest.security)
        .map_err(|e| anyhow::anyhow!("security config error: {e}"))?;

    println!(
        "  Network:     {} host(s) allowed",
        manifest.security.net_allow_list.len()
    );
    println!(
        "  Filesystem:  {:?}",
        manifest.security.fs_access
    );
    println!(
        "  Fuel:        {}",
        manifest.security.limits.max_fuel
    );
    println!(
        "  Memory:      {} MB",
        manifest.security.limits.max_memory_mb
    );
    println!(
        "  Timeout:     {} ms",
        manifest.security.limits.max_execution_ms
    );

    // 3. Check WASM file
    println!("Checking WASM file...");
    let tool_dir =
        manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let wasm_path = tool_dir.join(&manifest.wasm);

    if wasm_path.exists() {
        let metadata = std::fs::metadata(&wasm_path)?;
        println!(
            "  WASM:        {} ({} bytes)",
            manifest.wasm,
            metadata.len()
        );
    } else {
        anyhow::bail!(
            "WASM file not found: {} (referenced in tool.toml)",
            wasm_path.display()
        );
    }

    // 4. Check arguments
    println!(
        "Checking arguments... ({} defined)",
        manifest.arguments.len()
    );
    for arg in &manifest.arguments {
        let req = if arg.required { "required" } else { "optional" };
        println!(
            "  {}: {} ({})",
            arg.name, arg.arg_type, req
        );
    }

    output::print_success(&format!(
        "Tool {} v{} is valid",
        manifest.id, manifest.version
    ));

    Ok(())
}
