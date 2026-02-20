//! Tool manifest format documentation.
//!
//! Parsing of `tool.toml` lives in `kami-config::manifest_loader`
//! to keep filesystem I/O and the `toml` crate out of the domain layer.
//! Domain types (`ToolManifest`, `ToolArgument`, etc.) are in `tool.rs`.
