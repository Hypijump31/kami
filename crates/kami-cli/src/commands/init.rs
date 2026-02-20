//! `kami init` command.
//!
//! Scaffolds a new KAMI tool project with Cargo.toml, tool.toml,
//! and a minimal src/lib.rs handler.

use std::path::Path;

use clap::Args;

use crate::commands::templates;
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

/// Executes the init command in the current working directory.
pub fn execute(args: &InitArgs) -> anyhow::Result<()> {
    execute_at(args, Path::new("."))
}

/// Executes the init command with an explicit base directory (testable).
pub(crate) fn execute_at(args: &InitArgs, base_dir: &Path) -> anyhow::Result<()> {
    let dir = base_dir.join(&args.name);
    if dir.exists() {
        anyhow::bail!("directory already exists: {}", dir.display());
    }

    let tool_id = format!("{}.{}", args.prefix, args.name);
    let crate_name = args.name.replace('-', "_");

    std::fs::create_dir_all(dir.join("src"))?;

    write_cargo_toml(&dir, &args.name)?;
    write_tool_toml(&dir, &tool_id, &args.name, &crate_name)?;
    write_lib_rs(&dir, &tool_id)?;
    std::fs::write(dir.join(".gitignore"), templates::GITIGNORE)?;

    print_success(&args.name, &crate_name);
    Ok(())
}

fn write_cargo_toml(dir: &Path, name: &str) -> anyhow::Result<()> {
    let content = templates::apply(templates::CARGO_TOML, &[("__TOOL_NAME__", name)]);
    std::fs::write(dir.join("Cargo.toml"), content)?;
    Ok(())
}

fn write_tool_toml(dir: &Path, tool_id: &str, name: &str, crate_name: &str) -> anyhow::Result<()> {
    let content = templates::apply(
        templates::TOOL_TOML,
        &[
            ("__TOOL_ID__", tool_id),
            ("__TOOL_NAME__", name),
            ("__CRATE_NAME__", crate_name),
        ],
    );
    std::fs::write(dir.join("tool.toml"), content)?;
    Ok(())
}

fn write_lib_rs(dir: &Path, tool_id: &str) -> anyhow::Result<()> {
    let content = templates::apply(templates::LIB_RS, &[("__TOOL_ID__", tool_id)]);
    std::fs::write(dir.join("src").join("lib.rs"), content)?;
    Ok(())
}

fn print_success(name: &str, crate_name: &str) {
    output::print_success(&format!("Created tool project: {name}/"));
    println!();
    println!("  {name}/");
    println!("  ├── Cargo.toml");
    println!("  ├── tool.toml");
    println!("  ├── .gitignore");
    println!("  └── src/");
    println!("      └── lib.rs");
    println!();
    println!("Next steps:");
    println!("  cd {name}");
    println!("  cargo build --target wasm32-wasip2 --release");
    println!("  cp target/wasm32-wasip2/release/{crate_name}.wasm .");
    println!("  kami validate .");
    println!("  kami install .");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_creates_expected_files() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let args = InitArgs {
            name: "test-tool".to_string(),
            prefix: "dev.test".to_string(),
        };
        execute_at(&args, tmp.path()).expect("init");

        let base = tmp.path().join("test-tool");
        assert!(base.join("Cargo.toml").exists());
        assert!(base.join("tool.toml").exists());
        assert!(base.join("src/lib.rs").exists());
        assert!(base.join(".gitignore").exists());

        let cargo = std::fs::read_to_string(base.join("Cargo.toml")).expect("read Cargo.toml");
        assert!(cargo.contains("name = \"test-tool\""));
        assert!(cargo.contains("path = \"../crates/kami-guest\""));

        let tool = std::fs::read_to_string(base.join("tool.toml")).expect("read tool.toml");
        assert!(tool.contains("id = \"dev.test.test-tool\""));

        let lib = std::fs::read_to_string(base.join("src/lib.rs")).expect("read lib.rs");
        assert!(lib.contains("name: \"dev.test.test-tool\""));
    }

    #[test]
    fn init_fails_if_dir_exists() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join("existing")).expect("mkdir");
        let args = InitArgs {
            name: "existing".to_string(),
            prefix: "dev.test".to_string(),
        };
        assert!(execute_at(&args, tmp.path()).is_err());
    }
}
