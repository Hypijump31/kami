//! WebAssembly Component Model loading, linking, and execution.

use std::path::Path;

use wasmtime::component::{Component, Instance, Linker};
use wasmtime::{Engine, Store};

use crate::error::EngineError;
use crate::state::HostState;

/// Loads a WASM component from raw bytes.
pub fn load_component(engine: &Engine, bytes: &[u8]) -> Result<Component, EngineError> {
    Component::new(engine, bytes).map_err(|e| EngineError::Compilation {
        reason: "failed to load component from bytes".to_string(),
        source: e,
    })
}

/// Loads a WASM component from a file path.
pub fn load_component_from_file(engine: &Engine, path: &Path) -> Result<Component, EngineError> {
    Component::from_file(engine, path).map_err(|e| EngineError::Compilation {
        reason: format!("failed to load component from {}", path.display()),
        source: e,
    })
}

/// Creates a `Linker<HostState>` with WASI async bindings registered.
///
/// This linker is reusable across multiple instantiations.
pub fn create_linker(engine: &Engine) -> Result<Linker<HostState>, EngineError> {
    let mut linker = Linker::new(engine);
    wasmtime_wasi::add_to_linker_async(&mut linker)
        .map_err(|e| EngineError::Config(format!("WASI linker: {e}")))?;
    crate::bindings::KamiTool::add_to_linker(&mut linker, |s| s)
        .map_err(|e| EngineError::Config(format!("host linker: {e}")))?;
    Ok(linker)
}

/// Instantiates a component asynchronously.
pub async fn instantiate_component(
    linker: &Linker<HostState>,
    store: &mut Store<HostState>,
    component: &Component,
) -> Result<Instance, EngineError> {
    linker
        .instantiate_async(store, component)
        .await
        .map_err(|e| EngineError::Instantiation {
            reason: "failed to instantiate component".to_string(),
            source: e,
        })
}

/// Calls an exported `run(input: string) -> result<string, string>` function.
///
/// This is the standard KAMI tool interface: takes JSON input, returns JSON output.
pub async fn call_tool_run(
    store: &mut Store<HostState>,
    instance: &Instance,
    input: &str,
) -> Result<Result<String, String>, EngineError> {
    let run_func = instance
        .get_typed_func::<(String,), (Result<String, String>,)>(&mut *store, "run")
        .map_err(|_| EngineError::ExportNotFound {
            name: "run".to_string(),
        })?;

    let (result,) = run_func
        .call_async(&mut *store, (input.to_string(),))
        .await
        .map_err(|e| EngineError::Trap {
            message: e.to_string(),
        })?;

    run_func
        .post_return_async(&mut *store)
        .await
        .map_err(|e| EngineError::Trap {
            message: format!("post_return failed: {e}"),
        })?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::Config;

    fn async_engine() -> Engine {
        let mut config = Config::new();
        config.async_support(true);
        Engine::new(&config).expect("async engine")
    }

    #[test]
    fn create_linker_registers_wasi_and_host() {
        assert!(create_linker(&async_engine()).is_ok());
    }

    #[test]
    fn load_from_nonexistent_file_is_compilation_error() {
        let result = load_component_from_file(&async_engine(), Path::new("/no/such/tool.wasm"));
        assert!(matches!(result, Err(EngineError::Compilation { .. })));
    }
}
