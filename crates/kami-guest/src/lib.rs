//! # kami-guest
//!
//! SDK for building WASM tools that run inside KAMI.
//!
//! This crate provides:
//! - **ABI helpers**: Parse input, serialize output, build results
//! - **`kami_tool!` macro**: Generate Component Model exports from a handler function
//!
//! # Quick Start
//!
//! ```ignore
//! use kami_guest::kami_tool;
//!
//! kami_tool! {
//!     name: "dev.example.echo",
//!     version: "1.0.0",
//!     description: "Echoes back the input",
//!     handler: handle,
//! }
//!
//! fn handle(input: &str) -> Result<String, String> {
//!     Ok(format!("echo: {input}"))
//! }
//! ```

pub mod abi;
pub mod macros;

pub use abi::{error_result, parse_input, text_result, to_output, ToolMetadata};
