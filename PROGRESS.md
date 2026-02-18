# KAMI - Progress Tracker

## Current State

### Phase 0: Foundations - COMPLETE

| Module | Status | Notes |
|--------|--------|-------|
| **Workspace** | Done | 11 crates, workspace deps, rust-toolchain |
| **kami-types** | Done | ToolId, ToolManifest, Capability, KamiError, DomainEvent |
| **kami-protocol** | Done | JSON-RPC 2.0, MCP tools/prompts/resources, schema validation |
| **kami-registry** | Done | ToolRepository trait, ToolQuery |
| **kami-config** | Done | Layered config (defaults + TOML + env), KamiConfig schema |
| **kami-guest** | Done | ABI helpers, kami_tool! macro, ToolMetadata |
| **WIT** | Done | world.wit, tool.wit, host.wit |

### Phase 1: Minimal Engine - COMPLETE

| Module | Status | Notes |
|--------|--------|-------|
| **kami-engine** | Done | Component Model (wasmtime v27), async instantiate, TypedFunc call, fuel metering |
| **kami-sandbox** | Done | Real WasiCtxBuilder, socket_addr_check network, fs perms, env isolation |
| **kami-runtime** | Done | WasmToolExecutor pipeline (ctx → store → instantiate → call → result) |
| **kami-cli run** | Done | Wired to engine, loads .wasm, executes with deny-all security |

### Phase 2: Isolation & Security - COMPLETE

| Module | Status | Notes |
|--------|--------|-------|
| **kami-types** | Done | `env_allow_list` on SecurityConfig |
| **kami-engine** | Done | StoreLimits in HostState, epoch interruption, store.limiter() |
| **kami-sandbox** | Done | Enhanced CapabilityChecker (env vars), validate_security_config() |
| **kami-runtime** | Done | Full pipeline: validate → sandbox → limits → epoch → timeout → execute |

### Phase 3: Registry & tool.toml - COMPLETE

| Module | Status | Notes |
|--------|--------|-------|
| **kami-types** | Done | `manifest.rs` - tool.toml parser (ToolToml → ToolManifest) |
| **kami-store-sqlite** | Done | Full CRUD, migrations (v1), in-memory support, JSON columns |
| **kami-cli install** | Done | Parse tool.toml, verify .wasm, register in SQLite |
| **kami-cli list** | Done | Query registry with optional name filter, tabular output |
| **kami-cli inspect** | Done | Detailed tool info (security, limits, arguments) |

### Phase 4: Runtime Orchestration - COMPLETE

| Module | Status | Notes |
|--------|--------|-------|
| **kami-runtime/cache** | Done | ComponentCache with RwLock<HashMap>, FIFO eviction, thread-safe |
| **kami-runtime/resolver** | Done | ToolResolver: registry → WASM load → compile → cache |
| **kami-runtime/scheduler** | Done | Semaphore-based concurrency control, RAII permits |
| **kami-runtime/orchestrator** | Done | KamiRuntime: resolve + schedule + execute pipeline |
| **kami-cli exec** | Done | Execute registered tool by ID via full runtime pipeline |

### Phase 5: MCP Protocol & Transport - COMPLETE

| Module | Status | Notes |
|--------|--------|-------|
| **kami-protocol/initialize** | Done | MCP initialize handshake types, PROTOCOL_VERSION constant |
| **kami-transport-stdio/handler** | Done | McpHandler: dispatch initialize, tools/list, tools/call |
| **kami-transport-stdio/transport** | Done | StdioTransport: generic line-delimited JSON I/O |
| **kami-transport-stdio/server** | Done | McpServer: read → dispatch → write loop |
| **kami-cli serve** | Done | `kami serve` starts MCP server on stdio |

### Phase 6: SDK & Documentation - COMPLETE

| Module | Status | Notes |
|--------|--------|-------|
| **kami-guest/abi** | Done | parse_input, to_output, text_result, error_result, ToolMetadata |
| **kami-guest/macros** | Done | `kami_tool!` declarative macro for handler wiring |
| **docs/ARCHITECTURE.md** | Done | 7 ADRs (hexagonal, security, wasmtime, async, MCP, SDK, cache) |
| **README.md** | Done | Visual GitHub layout, diagrams, crate map, CLI reference |

### Build Status
- `cargo build` - PASS
- `cargo test` - 95 tests PASS
- `cargo clippy` - CLEAN (0 warnings)

## Tasks Accomplished

### Session 1 (Phase 0)
- Created full workspace with 11 crates
- Implemented domain layer types (kami-types) with tests
- Implemented protocol types (kami-protocol) with JSON-RPC 2.0 and MCP
- Implemented registry port traits (kami-registry)
- Implemented sandbox capability checker with deny-all defaults
- Set up layered configuration (figment)
- Created CLI with clap derive
- Created WIT interface definitions

### Session 2 (Phase 1)
- Implemented kami-engine with wasmtime v27 Component Model
- Created HostState with WasiView trait implementation
- Implemented async component loading, linking, instantiation
- Solved Canonical ABI for `result<string, string>` (indirect retptr)
- Built real WasiCtxBuilder with socket_addr_check network control
- Created WasmToolExecutor async pipeline in kami-runtime
- Wired CLI `run` command to full execution pipeline
- Created WAT echo component for integration tests

### Session 3 (Phase 2 + Phase 3)
- **Phase 2 - Isolation:**
  - env_allow_list, StoreLimits, epoch interruption, validate_security_config()
  - Full executor pipeline with timeout enforcement
  - CLI --max-memory-mb and --timeout-ms flags
- **Phase 3 - Registry:**
  - tool.toml parser with TOML → ToolManifest mapping
  - SQLite CRUD repository with migrations (PRAGMA user_version)
  - JSON serialization for security config and arguments columns
  - install command: parse manifest, verify wasm, check duplicates, register
  - list command: tabular output, name filter
  - inspect command: full tool details with security and arguments
- 67 tests passing, clippy clean

### Session 4 (Phase 4)
- ComponentCache: thread-safe compiled WASM caching with FIFO eviction
- ToolResolver: registry lookup → WASM file load → compile → cache pipeline
- Scheduler: tokio::sync::Semaphore concurrency control with RAII permits
- KamiRuntime: top-level orchestrator combining resolver + scheduler + executor
- `kami exec` command: execute registered tools by ID via full runtime pipeline
- 73 tests passing, clippy clean

### Session 5 (Phase 5)
- MCP initialize handshake types (InitializeParams/Result, ServerCapabilities)
- McpHandler: JSON-RPC dispatch for initialize, tools/list, tools/call
- StdioTransport: generic async line-delimited JSON reader/writer
- McpServer: main event loop (read → parse → dispatch → respond)
- `kami serve` CLI command to start MCP server on stdio
- inputSchema generation from ToolArgument definitions
- 81 tests passing, clippy clean

### Session 6 (Phase 6)
- kami-guest SDK: ABI helpers (parse_input, to_output, text_result, error_result)
- kami_tool! declarative macro for handler wiring + ToolMetadata
- ARCHITECTURE.md: ADR-005 (MCP stdio), ADR-006 (Guest SDK), ADR-007 (Component Cache)
- README.md: full rewrite with visual diagrams, security model, crate map, CLI reference
- 89 tests passing, clippy clean

### Session 7 (Post-Phase: CLI Enhancements)
- `--input-file` / `-f` flag on `run` and `exec` for JSON file input
- Stdin support via `--input-file -` (pipe JSON from another command)
- JSON validation on file/stdin input before execution
- Shared `input.rs` helper module (resolve_input with 6 tests)
- Documentation updates (DEPLOYMENT.md, DEVELOPER.md, README.md)
- 95 tests passing, clippy clean

## Blockers
- None

## All Phases Complete

All 6 phases of the KAMI roadmap are implemented:
- 11 crates, hexagonal architecture
- Wasmtime v27 Component Model with WASI P2
- Capability-based security with deny-all defaults
- SQLite registry with tool.toml manifests
- Runtime orchestrator with cache + scheduler
- MCP server over stdio (JSON-RPC 2.0)
- Guest SDK with kami_tool! macro

## Future Enhancements
- [ ] End-to-end integration tests (install → exec → MCP round-trip)
- [ ] LRU cache eviction (replace FIFO)
- [ ] WebSocket/SSE transport adapter
- [ ] `notifications/initialized` MCP compliance
- [ ] `wit-bindgen` integration in kami-guest for full Component Model bindings
- [ ] CI/CD pipeline (GitHub Actions)
