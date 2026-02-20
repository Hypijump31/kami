//! Basic engine integration tests: component loading, echo, fuel metering.

mod common;

use kami_engine::{
    call_tool_run, create_engine, create_linker, create_store, instantiate_component,
    load_component, HostState, InstanceConfig,
};
use kami_sandbox::{build_wasi_ctx, WasiConfig};
use kami_types::SecurityConfig;

use common::ECHO_COMPONENT_WAT;

#[tokio::test]
async fn echo_component_returns_input() {
    let config = InstanceConfig::default();
    let engine = create_engine(&config).expect("engine creation");
    let linker = create_linker(&engine).expect("linker creation");
    let component =
        load_component(&engine, ECHO_COMPONENT_WAT.as_bytes()).expect("component compilation");

    let security = SecurityConfig::default();
    let wasi_config = WasiConfig::default();
    let wasi_ctx = build_wasi_ctx(&security, &wasi_config, None).expect("wasi ctx");

    let host_state = HostState::new(wasi_ctx);
    let mut store = create_store(&engine, host_state, config.max_fuel).expect("store");

    let instance = instantiate_component(&linker, &mut store, &component)
        .await
        .expect("instantiation");

    let result = call_tool_run(&mut store, &instance, "hello world")
        .await
        .expect("call_tool_run");

    assert_eq!(result, Ok("hello world".to_string()));
}

#[tokio::test]
async fn echo_component_with_json() {
    let config = InstanceConfig::default();
    let engine = create_engine(&config).expect("engine");
    let linker = create_linker(&engine).expect("linker");
    let component = load_component(&engine, ECHO_COMPONENT_WAT.as_bytes()).expect("component");

    let security = SecurityConfig::default();
    let wasi_ctx = build_wasi_ctx(&security, &WasiConfig::default(), None).expect("ctx");
    let mut store = create_store(&engine, HostState::new(wasi_ctx), 1_000_000).expect("store");

    let instance = instantiate_component(&linker, &mut store, &component)
        .await
        .expect("instantiation");

    let input = r#"{"url":"https://example.com"}"#;
    let result = call_tool_run(&mut store, &instance, input)
        .await
        .expect("call");

    assert_eq!(result, Ok(input.to_string()));
}

#[tokio::test]
async fn executor_runs_component() {
    use kami_runtime::{ToolExecutor, WasmToolExecutor};

    let config = InstanceConfig {
        epoch_interruption: true,
        ..InstanceConfig::default()
    };
    let engine = create_engine(&config).expect("engine");
    let linker = create_linker(&engine).expect("linker");
    let component = load_component(&engine, ECHO_COMPONENT_WAT.as_bytes()).expect("component");

    let executor = WasmToolExecutor::new(engine, linker);
    let security = SecurityConfig::default();

    let result = executor
        .execute(&component, r#"{"key":"value"}"#, &security)
        .await
        .expect("execution");

    assert!(result.success);
    assert_eq!(result.content, r#"{"key":"value"}"#);
    assert!(result.duration_ms < 5000);
    assert!(result.fuel_consumed > 0);
}

#[tokio::test]
async fn fuel_metering_works() {
    let config = InstanceConfig::default();
    let engine = create_engine(&config).expect("engine");
    let linker = create_linker(&engine).expect("linker");
    let component = load_component(&engine, ECHO_COMPONENT_WAT.as_bytes()).expect("component");

    let security = SecurityConfig::default();
    let wasi_ctx = build_wasi_ctx(&security, &WasiConfig::default(), None).expect("ctx");
    let mut store = create_store(&engine, HostState::new(wasi_ctx), 500_000).expect("store");

    let fuel_before = store.get_fuel().expect("get fuel");
    assert_eq!(fuel_before, 500_000);

    let instance = instantiate_component(&linker, &mut store, &component)
        .await
        .expect("instantiation");

    let result = call_tool_run(&mut store, &instance, "hello")
        .await
        .expect("call");
    assert_eq!(result, Ok("hello".to_string()));

    let fuel_after = store.get_fuel().expect("get fuel");
    assert!(
        fuel_after < fuel_before,
        "fuel should be consumed after execution"
    );
}
