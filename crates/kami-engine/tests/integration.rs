//! Integration tests for the KAMI engine pipeline.
//!
//! Tests the full flow: Component loading -> Sandbox -> Execution.

use kami_engine::{
    call_tool_run, create_engine, create_linker, create_store, instantiate_component,
    load_component, HostState, InstanceConfig,
};
use kami_sandbox::{build_wasi_ctx, WasiConfig};
use kami_types::SecurityConfig;

/// Minimal WASM component that echoes its input as Ok(input).
///
/// Canonical ABI for `result<string, string>`:
/// - Core params: (ptr: i32, len: i32) for input string
/// - Core result: (i32) pointer to return-area struct
///   - [retptr+0]: i32 discriminant (0=Ok, 1=Err)
///   - [retptr+4]: i32 string ptr
///   - [retptr+8]: i32 string len
const ECHO_COMPONENT_WAT: &str = r#"
(component
  (core module $m
    (memory (export "memory") 1)

    ;; Return area at a fixed location (offset 0x1000)
    (global $retarea (mut i32) (i32.const 4096))

    (func (export "cabi_realloc") (param i32 i32 i32 i32) (result i32)
      ;; Bump allocator starting at offset 256
      ;; new_size is param 3, we just return a fixed offset above static data
      i32.const 256
    )

    ;; run(ptr: i32, len: i32) -> i32 (retptr)
    (func (export "run") (param $ptr i32) (param $len i32) (result i32)
      ;; Write discriminant 0 (Ok) at retarea+0
      global.get $retarea
      i32.const 0
      i32.store

      ;; Write string ptr at retarea+4
      global.get $retarea
      i32.const 4
      i32.add
      local.get $ptr
      i32.store

      ;; Write string len at retarea+8
      global.get $retarea
      i32.const 8
      i32.add
      local.get $len
      i32.store

      ;; Return pointer to the result struct
      global.get $retarea
    )

    ;; post-return cleanup (takes the retptr)
    (func (export "cabi_post_run") (param i32))
  )

  (core instance $i (instantiate $m))

  (func (export "run")
    (param "input" string)
    (result (result string (error string)))
    (canon lift
      (core func $i "run")
      (memory $i "memory")
      (realloc (func $i "cabi_realloc"))
      (post-return (func $i "cabi_post_run"))
    )
  )
)
"#;

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
    let component =
        load_component(&engine, ECHO_COMPONENT_WAT.as_bytes()).expect("component");

    let security = SecurityConfig::default();
    let wasi_ctx = build_wasi_ctx(&security, &WasiConfig::default(), None).expect("ctx");
    let mut store = create_store(&engine, HostState::new(wasi_ctx), 1_000_000).expect("store");

    let instance = instantiate_component(&linker, &mut store, &component)
        .await
        .expect("instantiation");

    let result = call_tool_run(&mut store, &instance, r#"{"url":"https://example.com"}"#)
        .await
        .expect("call");

    assert_eq!(result, Ok(r#"{"url":"https://example.com"}"#.to_string()));
}

#[tokio::test]
async fn executor_runs_component() {
    use kami_runtime::WasmToolExecutor;

    let config = InstanceConfig::default();
    let engine = create_engine(&config).expect("engine");
    let linker = create_linker(&engine).expect("linker");
    let component =
        load_component(&engine, ECHO_COMPONENT_WAT.as_bytes()).expect("component");

    let executor = WasmToolExecutor::new(engine, linker);
    let security = SecurityConfig::default();

    let result = executor
        .execute_component(&component, r#"{"key":"value"}"#, &security, 1_000_000)
        .await
        .expect("execution");

    assert!(result.success);
    assert_eq!(result.content, r#"{"key":"value"}"#);
    assert!(result.duration_ms < 5000);
}

#[tokio::test]
async fn fuel_metering_works() {
    let config = InstanceConfig::default();
    let engine = create_engine(&config).expect("engine");
    let linker = create_linker(&engine).expect("linker");
    let component =
        load_component(&engine, ECHO_COMPONENT_WAT.as_bytes()).expect("component");

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
    // Fuel should have been consumed
    assert!(fuel_after < fuel_before, "fuel should be consumed after execution");
}
