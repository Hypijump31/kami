//! Auto-generated Component Model bindings from WIT interfaces.
//!
//! Gated behind the `component-model` feature flag. When enabled,
//! `wit-bindgen` generates the `Guest` trait and host import functions
//! from the WIT definitions in `wit/`.
//!
//! # For Tool Authors
//!
//! To build a real WASM component (not just a test binary):
//!
//! ```toml
//! # your-tool/Cargo.toml
//! [dependencies]
//! kami-guest = { path = "../../crates/kami-guest", features = ["component-model"] }
//! ```
//!
//! ```ignore
//! // your-tool/src/lib.rs
//! use kami_guest::bindings;
//!
//! struct MyTool;
//!
//! impl bindings::Guest for MyTool {
//!     fn run(input: String) -> Result<String, String> {
//!         // your logic
//!         Ok(format!("result: {input}"))
//!     }
//!     fn describe() -> String {
//!         r#"{"name":"my-tool","version":"1.0.0"}"#.into()
//!     }
//! }
//!
//! bindings::export!(MyTool with_types_in bindings);
//! ```
//!
//! Build with: `cargo build --target wasm32-wasip2 --release`
//!
//! # Native Testing
//!
//! For native tests (no WASM), use `kami_tool!` macro instead:
//!
//! ```ignore
//! kami_guest::kami_tool! {
//!     name: "my-tool",
//!     version: "1.0.0",
//!     description: "My tool",
//!     handler: my_handler,
//! }
//! ```

#[cfg(feature = "component-model")]
wit_bindgen::generate!({
    world: "kami-tool",
    path: "../../wit",
});

#[cfg(feature = "component-model")]
pub use exports::kami::tool::tool::Guest;
