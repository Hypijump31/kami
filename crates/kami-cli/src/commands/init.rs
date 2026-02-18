//! `kami init` command.
//!
//! Scaffolds a new KAMI tool project with Cargo.toml, tool.toml,
//! and a minimal src/lib.rs handler.

use std::path::Path;

use clap::Args;

use crate::output;

/// Scaffold a new KAMI tool project.
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Tool name (e.g. "my-tool"). Used for directory and crate name.
    pub name: String,
    /// Reverse-domain prefix for the tool ID (e.g. "dev.example").
    #[arg(short, long, default_value = "dev.example")]
    pub prefix: String,
}

/// Executes the init command.
pub fn execute(args: &InitArgs) -> anyhow::Result<()> {
    let dir = Path::new(&args.name);
    if dir.exists() {
        anyhow::bail!(
            "directory already exists: {}",
            dir.display()
        );
    }

    let tool_id = format!("{}.{}", args.prefix, args.name);
    let crate_name = args.name.replace('-', "_");

    // Create directory structure
    std::fs::create_dir_all(dir.join("src"))?;

    // 1. Generate Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
kami-guest = {{ git = "https://github.com/Hypijump31/kami.git", path = "crates/kami-guest" }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#,
        name = args.name
    );

    std::fs::write(dir.join("Cargo.toml"), cargo_toml)?;

    // 2. Generate tool.toml
    let tool_toml = format!(
        r#"[tool]
id = "{tool_id}"
name = "{name}"
version = "1.0.0"
wasm = "{crate_name}.wasm"

[mcp]
description = "TODO: Describe what this tool does"

[[mcp.arguments]]
name = "input"
type = "string"
description = "TODO: Describe this argument"
required = true

[security]
net_allow_list = []
fs_access = "none"
max_memory_mb = 32
max_execution_ms = 5000
max_fuel = 1000000
"#,
        tool_id = tool_id,
        name = args.name,
        crate_name = crate_name
    );

    std::fs::write(dir.join("tool.toml"), tool_toml)?;

    // 3. Generate src/lib.rs
    let lib_rs = format!(
        r#"use kami_guest::kami_tool;

kami_tool! {{
    name: "{tool_id}",
    version: "1.0.0",
    description: "TODO: Describe what this tool does",
    handler: handle,
}}

fn handle(input: &str) -> Result<String, String> {{
    let args: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| format!("invalid JSON: {{e}}"))?;

    let response = serde_json::json!({{
        "result": args,
        "tool": "{tool_id}"
    }});

    Ok(response.to_string())
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn valid_input() {{
        let result = handle(r#"{{"input":"hello"}}"#);
        assert!(result.is_ok());
    }}

    #[test]
    fn invalid_input() {{
        let result = handle("not json");
        assert!(result.is_err());
    }}

    #[test]
    fn empty_input() {{
        let result = handle("{{}}");
        assert!(result.is_ok());
    }}
}}
"#,
        tool_id = tool_id
    );

    std::fs::write(dir.join("src").join("lib.rs"), lib_rs)?;

    // 4. Generate .gitignore
    let gitignore = "target/\n*.wasm\n";
    std::fs::write(dir.join(".gitignore"), gitignore)?;

    output::print_success(&format!(
        "Created tool project: {name}/",
        name = args.name
    ));
    println!();
    println!("  {}/", args.name);
    println!("  ├── Cargo.toml");
    println!("  ├── tool.toml");
    println!("  ├── .gitignore");
    println!("  └── src/");
    println!("      └── lib.rs");
    println!();
    println!("Next steps:");
    println!(
        "  cd {}",
        args.name
    );
    println!("  cargo build --target wasm32-wasip2 --release");
    println!(
        "  cp target/wasm32-wasip2/release/{crate_name}.wasm ."
    );
    println!("  kami validate .");
    println!("  kami install .");

    Ok(())
}
