//! # kami-engine
//!
//! WASM engine powered by Wasmtime with Component Model support.
//! Handles component compilation, instance creation, WASI linking,
//! and async execution.

pub mod component;
pub mod error;
pub mod instance;
pub mod linker;
pub mod memory;
pub mod state;

pub use component::{
    call_tool_run, create_linker, instantiate_component, load_component,
    load_component_from_file,
};
pub use error::EngineError;
pub use instance::{create_engine, create_store, InstanceConfig};
pub use memory::MemoryStats;
pub use state::HostState;
