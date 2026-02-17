//! WebAssembly Component Model loading, linking, and execution.

use std::path::Path;

use wasmtime::component::{Component, ComponentNamedList, Instance, Lift, Linker, Lower};
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
pub fn load_component_from_file(
    engine: &Engine,
    path: &Path,
) -> Result<Component, EngineError> {
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
    wasmtime_wasi::add_to_linker_async(&mut linker).map_err(|e| {
        EngineError::Config(format!("failed to add WASI to linker: {e}"))
    })?;
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
    let run_func = find_typed_func::<(String,), (Result<String, String>,)>(
        &mut *store,
        instance,
        "run",
    )?;

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

/// Looks up a typed export function by name.
fn find_typed_func<Params, Results>(
    store: &mut Store<HostState>,
    instance: &Instance,
    name: &str,
) -> Result<wasmtime::component::TypedFunc<Params, Results>, EngineError>
where
    Params: ComponentNamedList + Lower + Send + Sync,
    Results: ComponentNamedList + Lift + Send + Sync,
{
    instance
        .get_typed_func::<Params, Results>(store, name)
        .map_err(|_| EngineError::ExportNotFound {
            name: name.to_string(),
        })
}
