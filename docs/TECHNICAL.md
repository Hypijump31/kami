<p align="center"><img src="../site/assets/logo.png" alt="KAMI" height="48"></p>

# KAMI - Technical Reference

> Deep technical documentation for R&D engineers.

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Data Flow](#data-flow)
3. [Crate Dependency Graph](#crate-dependency-graph)
4. [Crate API Reference](#crate-api-reference)
5. [Error Handling Strategy](#error-handling-strategy)
6. [Async Model](#async-model)
7. [WASM Execution Pipeline](#wasm-execution-pipeline)
8. [Component Cache Internals](#component-cache-internals)
9. [Wire Protocol](#wire-protocol)
10. [Performance Characteristics](#performance-characteristics)

---

## System Overview

KAMI is a WASM-based tool orchestrator designed to give AI agents (Claude, GPT, etc.) secure access to third-party tools. Each tool is a WASM Component Model module executed in a sandboxed environment with capability-based security.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         KAMI Process                                │
│                                                                     │
│  stdin ──► StdioTransport ──► McpHandler ──► KamiRuntime            │
│                                    │              │                 │
│                                    │         ┌────▼────┐            │
│                                    │         │Scheduler│            │
│                                    │         │(Semaphore)           │
│                                    │         └────┬────┘            │
│                                    │              │                 │
│                               ┌────▼────┐   ┌────▼──────┐          │
│                               │Registry │   │ToolResolver│          │
│                               │(SQLite) │   │(Cache+Load)│          │
│                               └─────────┘   └────┬──────┘          │
│                                                   │                 │
│                                              ┌────▼──────────┐     │
│                                              │WasmToolExecutor│     │
│                                              │  validate()    │     │
│                                              │  sandbox()     │     │
│                                              │  limits()      │     │
│                                              │  execute()     │     │
│                                              └───────────────┘     │
│                                                                     │
│  stdout ◄── StdioTransport ◄── JSON-RPC response                   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Data Flow

### MCP Request Lifecycle

```
1. AI Agent sends JSON-RPC over stdin
   │
2. StdioTransport.read_line()
   │  Parse line-delimited JSON
   │
3. McpServer dispatches to McpHandler
   │
4. McpHandler.dispatch(request)
   │
   ├─ "initialize" ──► Return ServerCapabilities
   │
   ├─ "tools/list" ──► ToolRepository.find_all()
   │                    Map Tool → McpToolDefinition
   │                    Build inputSchema from ToolArgument[]
   │
   └─ "tools/call" ──► KamiRuntime.execute(tool_id, input)
                        │
                        ├─ Scheduler.acquire() ──► Semaphore permit
                        │
                        ├─ ToolResolver.resolve(tool_id)
                        │   ├─ ComponentCache.get() ──► cache hit? return
                        │   ├─ ToolRepository.find_by_id()
                        │   ├─ load_component_from_file()
                        │   └─ ComponentCache.insert()
                        │
                        └─ WasmToolExecutor.execute_component()
                            ├─ validate_security_config()
                            ├─ build_wasi_ctx() ──► network, fs, env
                            ├─ HostState::with_limits() ──► memory cap
                            ├─ create_store() ──► fuel + limiter
                            ├─ set_epoch_deadline() ──► timeout
                            ├─ tokio::spawn(epoch_ticker)
                            ├─ instantiate_component()
                            ├─ call_tool_run(&store, &instance, input)
                            └─ Return ExecutionResult { content, duration, fuel }
```

### Tool Installation Lifecycle

```
1. kami install ./path/to/tool/
   │
2. Detect tool.toml (file or directory)
   │
3. parse_tool_manifest_file(path) ──► ToolManifest
   │  Validates: id format, version semver, security config
   │
4. Verify .wasm file exists at install_path/manifest.wasm
   │
5. SqliteToolRepository.insert(Tool { manifest, install_path, enabled })
   │  JSON serialize: security config + arguments
   │  Conflict detection: reject duplicate IDs
   │
6. Tool ready for execution via `kami exec <tool-id>`
```

---

## Crate Dependency Graph

```
                    kami-types (zero deps, serde only)
                   /          |           \
                  /           |            \
          kami-protocol    kami-registry    kami-guest
                |             |
                |        kami-store-sqlite
                |             |
           kami-engine    kami-sandbox
                \           /
                 \         /
                kami-runtime
                     |
            kami-transport-stdio
                     |
                  kami-cli (composition root)
                     |
                 kami-config
```

### Dependency Rules (enforced by Cargo workspace)

| From | Can depend on | Cannot depend on |
|------|--------------|------------------|
| Domain (types, protocol, registry) | Only `serde`, `thiserror` | Any application or adapter |
| Application (engine, sandbox, runtime) | Domain crates | Adapter crates |
| Adapter (store-sqlite, transport-stdio, config) | Domain + Application | Infrastructure |
| Infrastructure (cli) | Everything | — |
| SDK (guest) | Only `serde`, `serde_json` | Anything else |

---

## Crate API Reference

### kami-types

Core domain types. Zero external dependencies except serde.

| Type | Description |
|------|-------------|
| `ToolId` | Validated reverse-domain identifier (e.g. `dev.example.fetch`) |
| `ToolVersion` | Semantic version (major.minor.patch) |
| `ToolManifest` | Complete tool declaration (id, name, version, wasm, security, arguments) |
| `Tool` | Installed tool (manifest + install_path + enabled flag) |
| `ToolArgument` | MCP argument definition (name, type, description, required) |
| `SecurityConfig` | Capability declarations (net, fs, env, limits) |
| `FsAccess` | Filesystem access level enum (None, ReadOnly, Sandbox) |
| `ResourceLimits` | Fuel, memory MB, execution timeout MS |
| `KamiError` | Unified error type with ErrorKind + context |
| `DomainEvent` | Observability events (ToolInstalled, ToolExecuted, etc.) |

### kami-engine

Wasmtime Component Model integration.

| Function/Type | Description |
|---------------|-------------|
| `create_engine(config)` | Creates Wasmtime `Engine` with optional epoch interruption |
| `create_linker(engine)` | Creates `Linker<HostState>` with WASI P2 bindings |
| `create_store(engine, host_state, fuel)` | Creates `Store` with fuel + memory limiter |
| `load_component_from_file(engine, path)` | Compiles .wasm to `Component` |
| `instantiate_component(linker, store, component)` | Async instantiation |
| `call_tool_run(store, instance, input)` | Calls `run(string) -> result<string, string>` |
| `set_epoch_deadline(store, ticks)` | Configures epoch-based timeout |
| `HostState` | WasiView implementation (WasiCtx + ResourceTable + StoreLimits) |
| `InstanceConfig` | Engine configuration (fuel_enabled, epoch_interruption) |

### kami-sandbox

Capability-based security enforcement.

| Function/Type | Description |
|---------------|-------------|
| `build_wasi_ctx(security, wasi_config, jail_dir)` | Builds sandboxed WasiCtx |
| `validate_security_config(config)` | Early validation (non-zero limits, valid patterns) |
| `DefaultCapabilityChecker` | Checks capabilities against SecurityConfig |
| `check_network_access(host, allow_list)` | Wildcard pattern matching for hosts |
| `check_fs_access(path, jail_root)` | Path traversal prevention |
| `WasiConfig` | WASI context options (inherit_stdout, inherit_stderr, env_vars) |

### kami-runtime

Orchestrator and execution pipeline.

| Type | Description |
|------|-------------|
| `WasmToolExecutor` | Full isolation pipeline (validate → sandbox → limits → epoch → execute) |
| `KamiRuntime` | Top-level orchestrator (resolver + scheduler + executor) |
| `ToolResolver` | Registry → compile → cache pipeline |
| `ComponentCache` | Thread-safe compiled component cache (FIFO eviction) |
| `Scheduler` | Semaphore-based concurrency control |
| `RuntimeConfig` | Cache size, max concurrent, epoch interruption |
| `ExecutionResult` | Output (content, duration_ms, success, fuel_consumed) |

### kami-registry (port)

Abstract trait for tool persistence.

| Trait/Type | Description |
|------------|-------------|
| `ToolRepository` | `find_by_id`, `find_all`, `insert`, `delete` (async) |
| `ToolQuery` | Filtering + pagination (name_filter, enabled_only, limit, offset) |
| `RepositoryError` | NotFound, Storage, Conflict variants |

### kami-store-sqlite (adapter)

SQLite implementation of ToolRepository.

| Type | Description |
|------|-------------|
| `SqliteToolRepository` | `Mutex<Connection>` with JSON columns |
| Schema v1 | tools table, PRAGMA user_version migration |
| `open(path)` | Opens or creates database at path |
| `open_in_memory()` | In-memory database for testing |

### kami-transport-stdio (adapter)

MCP transport over stdio.

| Type | Description |
|------|-------------|
| `StdioTransport<R, W>` | Generic line-delimited JSON reader/writer |
| `McpHandler` | Dispatches JSON-RPC to MCP method handlers |
| `McpServer<R, W>` | Event loop: read → parse → dispatch → write |
| `JsonRpcOutput` | Union type for success/error responses |

### kami-guest (SDK)

Guest-side helpers for tool developers.

| Item | Description |
|------|-------------|
| `kami_tool!` | Declarative macro: generates `__kami_run` and `__kami_describe` |
| `parse_input<T>(json)` | Deserialize JSON input to typed struct |
| `to_output<T>(value)` | Serialize result to JSON string |
| `text_result(text)` | Wrap text in `{"text": ...}` |
| `error_result(message)` | Wrap error in `{"error": ...}` |
| `ToolMetadata` | Name/description/version for `describe` export |

---

## Error Handling Strategy

### Layer-Specific Error Types

```
kami-types:    KamiError (unified, with ErrorKind enum)
kami-engine:   EngineError (thiserror)
kami-sandbox:  SandboxError (thiserror)
kami-runtime:  RuntimeError (thiserror)
kami-store:    RepositoryError (thiserror)
kami-transport: TransportError (thiserror)
kami-cli:      anyhow::Error (context-rich, for display only)
```

### Error Conversion Chain

```
EngineError ──► RuntimeError ──► KamiError
SandboxError ─┘                     │
                                     ▼
RepositoryError ──► RuntimeError    Display to user
TransportError ──► KamiError
```

### ErrorKind Mapping

| Source Error | ErrorKind |
|-------------|-----------|
| `RuntimeError::ToolNotFound` | `NotFound` |
| `RuntimeError::Timeout` | `Timeout` |
| `RuntimeError::PoolExhausted` | `ResourceExhausted` |
| `RuntimeError::Engine(_)` | `Internal` |
| `SandboxError::InvalidConfig` | `InvalidInput` |
| `SandboxError::*` (other) | `PermissionDenied` |

### Rules

- **Zero `unwrap()`** — all errors propagated via `Result<T, E>`
- **Zero `panic!()`** — deterministic behavior in all paths
- `thiserror` in all library crates for typed errors
- `anyhow` only in `kami-cli` for user-facing context
- Explicit `From<T>` conversions between layers

---

## Async Model

### Tokio Runtime Usage

| Component | Async? | Reason |
|-----------|--------|--------|
| `WasmToolExecutor.execute_component()` | Yes | Wasmtime async instantiation + epoch ticker |
| `KamiRuntime.execute()` | Yes | Scheduler acquire + resolver + executor |
| `ToolResolver.resolve()` | Yes | Cache read/write locks + repository query |
| `ComponentCache.get/insert()` | Yes | `RwLock` async guards |
| `Scheduler.acquire()` | Yes | `Semaphore::acquire()` |
| `McpServer.run()` | Yes | Async stdin/stdout I/O loop |
| `ToolRepository` trait methods | Yes | Database I/O |

### `.block_on()` Isolation

Only `kami-cli` commands use `tokio::runtime::Runtime::new()?.block_on()`.
All library crates are fully async — no blocking calls.

### Timeout Mechanism (Dual-Layer)

```
Layer 1: Epoch Interruption (cooperative)
   Engine::increment_epoch() after timeout_ms
   Store traps on next epoch check

Layer 2: tokio::time::timeout (safety net)
   outer_timeout = timeout_ms + 500ms
   Catches cases where epoch check is delayed
```

---

## WASM Execution Pipeline

### Detailed Step-by-Step

```rust
// 1. VALIDATE — Reject invalid configs before any allocation
validate_security_config(security)?;
// Checks: fuel > 0, memory > 0, timeout > 0, valid net patterns

// 2. SANDBOX — Build isolated WASI context
let wasi_ctx = build_wasi_ctx(security, &wasi_config, None)?;
// Applies: network allow-list, fs access level, env var filtering
// Deny-all by default for everything

// 3. LIMITS — Create store with resource constraints
let host_state = HostState::with_limits(wasi_ctx, max_memory_bytes);
let store = create_store(&engine, host_state, fuel)?;
// store.limiter() → StoreLimits { memory_size, trap_on_grow_failure }
// store.set_fuel(fuel) → instruction budget

// 4. EPOCH — Configure timeout via cooperative interruption
set_epoch_deadline(&mut store, 1); // Trap after 1 epoch tick

// 5. TICKER — Spawn background task to trigger timeout
let tick_handle = tokio::spawn(async move {
    tokio::time::sleep(timeout_duration).await;
    engine_clone.increment_epoch(); // Triggers trap in guest
});

// 6. EXECUTE — Instantiate and call within timeout wrapper
let result = tokio::time::timeout(outer_timeout, async {
    let instance = instantiate_component(&linker, &mut store, &component).await?;
    call_tool_run(&mut store, &instance, input).await
}).await;

// 7. CLEANUP — Cancel ticker, compute fuel consumed
tick_handle.abort();
let fuel_consumed = initial_fuel - store.get_fuel().unwrap_or(0);
```

### Component Model ABI

The `call_tool_run` function uses Wasmtime's `TypedFunc` to call:

```wit
// Guest export (defined in wit/tool.wit)
run: func(input: string) -> result<string, string>;
```

This maps to Canonical ABI with indirect return pointer for the `result<string, string>` type.

---

## Component Cache Internals

```
┌─────────────────────────────────────────┐
│           ComponentCache                │
│                                         │
│  Arc<RwLock<HashMap<String, Cached>>>   │
│                                         │
│  max_size: usize (default: 32)         │
│                                         │
│  get(id) → RwLock::read()              │
│    O(1) HashMap lookup                  │
│    Returns cloned CachedComponent       │
│                                         │
│  insert(id, cached) → RwLock::write()  │
│    If at capacity: remove first key     │
│    (FIFO eviction via HashMap iter)     │
│    Insert new entry                     │
│                                         │
│  invalidate(id) → RwLock::write()      │
│    Remove specific entry                │
└─────────────────────────────────────────┘

CachedComponent {
    component: wasmtime::Component,  // Pre-compiled
    security: SecurityConfig,        // From manifest
    wasm_path: String,               // For debugging
}
```

### Why Cache Components?

| Operation | Typical Cost |
|-----------|-------------|
| WASM compilation | 10-100ms |
| Component instantiation | ~1ms |
| Function call | ~0.1ms |

Caching avoids recompilation on every `tools/call`.

---

## Wire Protocol

### MCP over Stdio

Transport: **Line-delimited JSON** (one JSON object per line, `\n` terminated).

```
Client → Server: {"jsonrpc":"2.0","id":1,"method":"initialize","params":{...}}\n
Server → Client: {"jsonrpc":"2.0","id":1,"result":{...}}\n
```

### Supported Methods

| Method | Params | Result |
|--------|--------|--------|
| `initialize` | `InitializeParams` | `InitializeResult` (capabilities, serverInfo) |
| `tools/list` | `ToolsListParams` (optional cursor) | `ToolsListResult` (tools[], next_cursor) |
| `tools/call` | `ToolsCallParams` (name, arguments) | `ToolsCallResult` (content[], isError) |

### Error Codes (JSON-RPC 2.0)

| Code | Constant | Meaning |
|------|----------|---------|
| -32700 | `PARSE_ERROR` | Invalid JSON |
| -32600 | `INVALID_REQUEST` | Not a valid JSON-RPC request |
| -32601 | `METHOD_NOT_FOUND` | Unknown method |
| -32602 | `INVALID_PARAMS` | Invalid method parameters |
| -32603 | `INTERNAL_ERROR` | Server-side error |

### tools/call Error Handling

Tool execution errors are returned as **successful JSON-RPC responses** with `isError: true` in the result (per MCP spec), NOT as JSON-RPC errors:

```json
{"jsonrpc":"2.0","id":2,"result":{
  "content":[{"type":"text","text":"tool not found: dev.example.missing"}],
  "isError":true
}}
```

---

## Performance Characteristics

### Memory

| Component | Memory Footprint |
|-----------|-----------------|
| KAMI process (idle) | ~20 MB |
| Per cached Component | ~1-10 MB (depends on WASM size) |
| Per active Store | configurable via `max_memory_mb` |
| SQLite registry | < 1 MB for 1000 tools |

### Latency

| Operation | Expected Latency |
|-----------|-----------------|
| Cold start (compile + execute) | 10-100ms |
| Warm start (cached + execute) | 1-5ms |
| tools/list (100 tools) | < 1ms |
| JSON-RPC parsing | < 0.1ms |

### Concurrency

| Config | Default | Description |
|--------|---------|-------------|
| `max_concurrent` | 4 | Semaphore permits for parallel execution |
| `cache_size` | 32 | Max cached compiled components |

Requests beyond `max_concurrent` block on the semaphore until a permit is released.

---

## SQLite Schema

### Version 1 (current)

```sql
CREATE TABLE tools (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    version     TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    wasm_path   TEXT NOT NULL,
    install_path TEXT NOT NULL,
    enabled     INTEGER NOT NULL DEFAULT 1,
    security    TEXT NOT NULL DEFAULT '{}',     -- JSON
    arguments   TEXT NOT NULL DEFAULT '[]',     -- JSON
    installed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

PRAGMA user_version = 1;  -- Schema versioning for migrations
```

### Migration Strategy

- `PRAGMA user_version` tracks schema version
- `run_migrations()` is idempotent (checks current version)
- Future versions add new `WHEN user_version < N` blocks
