//! Developer experience commands for KAMI tool authors.
//!
//! Provides `kami dev watch <tool-dir>` — rebuilds WASM on file changes.

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};
use notify::{Event, EventKind, RecursiveMode, Watcher};

/// Developer experience commands.
#[derive(Debug, Parser)]
pub struct DevArgs {
    #[command(subcommand)]
    command: DevCommand,
}

#[derive(Debug, Subcommand)]
enum DevCommand {
    /// Watch a tool directory and rebuild WASM on file changes.
    Watch(WatchArgs),
}

/// Arguments for the watch subcommand.
#[derive(Debug, Parser)]
pub struct WatchArgs {
    /// Path to the tool project directory (contains Cargo.toml).
    pub tool_dir: PathBuf,
    /// Build in release mode.
    #[arg(long)]
    pub release: bool,
}

/// Dispatch to the appropriate dev subcommand.
///
/// # Errors
/// Returns an error if the subcommand fails.
pub async fn execute(args: &DevArgs) -> anyhow::Result<()> {
    match &args.command {
        DevCommand::Watch(watch) => run_watch(watch).await,
    }
}

/// Watches a tool directory for source changes and rebuilds the WASM.
async fn run_watch(args: &WatchArgs) -> anyhow::Result<()> {
    let dir = args.tool_dir.canonicalize().map_err(|e| {
        anyhow::anyhow!("invalid tool directory '{}': {e}", args.tool_dir.display())
    })?;
    println!("[WATCH] monitoring {}", dir.display());

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = notify::recommended_watcher(move |ev| {
        let _ = tx.send(ev);
    })?;
    watcher.watch(&dir, RecursiveMode::Recursive)?;

    // Initial build.
    match build_wasm(&dir, args.release) {
        Ok(()) => println!("[OK] initial build succeeded"),
        Err(e) => eprintln!("[ERROR] initial build failed: {e}"),
    }

    let debounce = Duration::from_millis(400);
    let mut last_build = Instant::now();

    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let mut changed = false;
        while let Ok(Ok(event)) = rx.try_recv() {
            if is_source_event(&event) {
                changed = true;
            }
        }
        if changed && last_build.elapsed() >= debounce {
            println!("[WATCH] change detected, rebuilding…");
            match build_wasm(&dir, args.release) {
                Ok(()) => println!("[OK] rebuild succeeded"),
                Err(e) => eprintln!("[ERROR] rebuild failed: {e}"),
            }
            last_build = Instant::now();
        }
    }
}

/// Returns `true` if the event touched a `.rs` or `.toml` source file.
fn is_source_event(event: &Event) -> bool {
    let dominated = matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_)
    );
    dominated
        && event
            .paths
            .iter()
            .any(|p| matches!(p.extension().and_then(|e| e.to_str()), Some("rs" | "toml")))
}

/// Runs `cargo build --target wasm32-wasip2` in the given directory.
fn build_wasm(dir: &std::path::Path, release: bool) -> anyhow::Result<()> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build")
        .arg("--target")
        .arg("wasm32-wasip2")
        .current_dir(dir);
    if release {
        cmd.arg("--release");
    }
    let status = cmd
        .status()
        .map_err(|e| anyhow::anyhow!("failed to spawn cargo: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("cargo exited with {status}"))
    }
}

#[cfg(test)]
#[path = "dev_tests.rs"]
mod tests;
