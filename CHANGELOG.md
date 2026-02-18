# Changelog

All notable changes to KAMI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Workspace setup with 11 crates following hexagonal architecture
- `kami-types`: Domain types (ToolId, ToolManifest, Capability, KamiError, DomainEvent)
- `kami-protocol`: JSON-RPC 2.0 types and MCP method definitions
- `kami-registry`: Abstract `ToolRepository` trait (port)
- `kami-engine`: Wasmtime Component Model runtime (async, fuel metering, TypedFunc)
- `kami-sandbox`: Real WasiCtxBuilder with capability-based security (fs, network, env)
- `kami-runtime`: WasmToolExecutor async pipeline (load → sandbox → execute → result)
- `kami-store-sqlite`: SQLite repository adapter (stub)
- `kami-transport-stdio`: Transport error types (stub)
- `kami-config`: Layered configuration with figment (defaults + TOML + env)
- `kami-cli`: CLI with install, run, list, inspect subcommands (run wired to engine)
- `kami-guest`: Guest SDK module (stub)
- WIT interface definitions (world.wit, tool.wit, host.wit)
- Default configuration file (config/default.toml)
- Integration tests with WAT echo component (Canonical ABI)
- `kami-types`: `env_allow_list` field on SecurityConfig for explicit env var control
- `kami-engine`: `StoreLimits` integration via `HostState::with_limits()` and `store.limiter()`
- `kami-engine`: Epoch-based interruption support (`set_epoch_deadline`)
- `kami-sandbox`: `validate_security_config()` for early misconfiguration detection
- `kami-sandbox`: `InvalidConfig` error variant
- `kami-runtime`: Full isolation pipeline (validate → sandbox → limits → epoch → timeout)
- `kami-cli`: `--max-memory-mb` and `--timeout-ms` flags for run command
- Phase 2 integration tests (config validation, fuel tracking, memory limits, capability denial)
- `kami-types`: `manifest.rs` tool.toml parser (TOML sections → ToolManifest)
- `kami-store-sqlite`: Full CRUD implementation with SQLite migrations (v1)
- `kami-store-sqlite`: JSON columns for security config and arguments
- `kami-store-sqlite`: In-memory database support for testing
- `kami-cli install`: Parse tool.toml, verify WASM, register in SQLite
- `kami-cli list`: Tabular output with name filter
- `kami-cli inspect`: Detailed tool info (security, limits, arguments)
- `kami-cli`: `--db` flag for custom database path on all registry commands
- `kami-runtime`: `ComponentCache` for pre-compiled WASM component caching (FIFO eviction)
- `kami-runtime`: `ToolResolver` for registry → compile → cache pipeline
- `kami-runtime`: `Scheduler` with semaphore-based concurrency control
- `kami-runtime`: `KamiRuntime` top-level orchestrator (resolve + schedule + execute)
- `kami-runtime`: `RuntimeConfig` for cache size, concurrency, and epoch settings
- `kami-cli exec`: Execute registered tools by ID via full runtime pipeline
- `kami-protocol`: MCP `initialize` method types (handshake, capabilities, server/client info)
- `kami-protocol`: `PROTOCOL_VERSION` constant (`2024-11-05`)
- `kami-transport-stdio`: `McpHandler` for JSON-RPC method dispatch (initialize, tools/list, tools/call)
- `kami-transport-stdio`: `StdioTransport` generic line-delimited JSON I/O
- `kami-transport-stdio`: `McpServer` event loop (read → parse → dispatch → respond)
- `kami-transport-stdio`: `inputSchema` generation from `ToolArgument` definitions
- `kami-cli serve`: Start MCP server over stdio with registry and runtime
- `kami-guest`: ABI helpers (`parse_input`, `to_output`, `text_result`, `error_result`)
- `kami-guest`: `kami_tool!` declarative macro for handler wiring
- `kami-guest`: `ToolMetadata` struct for `describe` export
- `docs/ARCHITECTURE.md`: ADR-005 (MCP stdio), ADR-006 (Guest SDK), ADR-007 (Component Cache)
- `README.md`: Complete rewrite with visual diagrams, security model, crate map, CLI reference
- `docs/TECHNICAL.md`: Full technical reference (data flows, crate APIs, error handling, wire protocol, performance)
- `docs/SECURITY.md`: Security deep dive (threat model, defense in depth, attack surface analysis)
- `docs/DEVELOPER.md`: Tool developer guide (build, test, publish WASM tools with kami-guest SDK)
- `docs/DEPLOYMENT.md`: Operations guide (build, install, configure, AI agent integration, monitoring)
- `kami-cli`: Binary renamed from `kami-cli` to `kami` via `[[bin]]` section
- `kami-cli`: `--input-file` / `-f` flag on `run` and `exec` commands for JSON file input
- `kami-cli`: Support for stdin input via `--input-file -` (pipe JSON from another command)
- `kami-cli`: JSON validation on file/stdin input before execution
