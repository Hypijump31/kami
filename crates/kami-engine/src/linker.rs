//! Host function linking for WASM Component instances.
//!
//! Custom host functions beyond WASI will be registered here
//! in future phases (logging, capability checks, metrics, etc.).

pub use crate::component::create_linker;
