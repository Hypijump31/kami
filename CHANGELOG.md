# Changelog

All notable changes to KAMI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added (Session 24 — Documentation Completion)
- **`GET /health/ready`** readiness probe endpoint in `kami-transport-http` (completes Annexe B v1.0)
- **Continue.dev integration guide** in `docs/INTEGRATION.md` (stdio + custom registry, v0.8.0+)
- **4th example** `http-fetch` added to examples table in `docs/GETTING_STARTED.md`
- **1 new test** `ready_endpoint_returns_ok` in `kami-transport-http/tests/router_integration.rs`
- **403 tests** total, clippy clean, fmt clean

### Fixed (Session 24)
- `docs/GETTING_STARTED.md`: `your-org` placeholders replaced with `Hypijump31`, step numbering fixed (duplicate "5." → "6.")
- `docs/DEPLOYMENT.md`: test count updated from `89+` to `402+`
- `docs/INTEGRATION.md` endpoints table: `GET /health/ready` now points to an implemented endpoint

### Added (Session 23 — Cryptographic Plugin Signatures)
- **Ed25519 plugin signing**: Full cryptographic signature workflow for WASM tools
- **`kami keygen`**: Generate Ed25519 keypair to `~/.kami/keys/` (with `--force` overwrite, Unix 0o600 perms)
- **`kami sign`**: Sign a WASM plugin, prints signature + public key + instructions for `tool.toml`
- **`kami verify` enhanced**: Now checks both SHA-256 integrity and Ed25519 signature (with `--public-key` flag)
- **Signature verification at execution**: Resolver verifies Ed25519 signature before running tool (if present)
- **Domain types**: `signature` and `signer_public_key` optional fields on `ToolManifest`
- **`kami-runtime/signature.rs`** (144 lines): `generate_keypair()`, `sign_file()`, `verify_file_signature()`, `public_key_from_secret()`
- **SQLite migration v4**: `signature TEXT` and `signer_public_key TEXT` columns
- **Registry schema**: Optional `signature` and `signer_public_key` fields (backward compatible)
- **New dependencies**: `ed25519-dalek` v2 (std, rand_core), `rand` v0.8
- **11 new unit tests** across signature.rs (6), keygen.rs (3), sign.rs (2)
- **402 tests** total, clippy clean, fmt clean

### Changed (Session 23)
- `kami verify` refactored into `verify_sha256()` and `verify_signature()` functions
- `kami verify --public-key` accepts hex string or file path (auto-detected)
- `resolver.rs` adds step 5: Ed25519 signature check after SHA-256 integrity
- `repository_impl.rs` COLS/INSERT/UPDATE extended for signature columns
- `row_mapping.rs` reads signature columns from SQLite rows
- `publish.rs` RegistryEntry includes optional signature/signer_public_key

### Added (Session 21 — Remote Plugin Install & Search)
- **`kami install` from URL**: `kami install https://example.com/tool.zip` downloads, extracts, and registers tools
- **`kami install` from GitHub**: `kami install owner/repo@v1.0.0` fetches release archives from GitHub
- **`kami search` command**: queries a remote JSON registry index, fuzzy matches on name/description/id
- **`kami publish` command**: generates a registry entry from local tool, prints PR submission instructions (Homebrew model)
- **Registry template** (`registry/`): `index.json` + `schema.json` + CI workflow for validating PRs
- **`download.rs` module** (149 lines): HTTP download via `reqwest`, ZIP extraction with zip-slip prevention, `strip_top_dir` flattening
- **Plugin storage**: `~/.kami/plugins/<tool-id>/` with configurable `KAMI_DATA_DIR`
- **New dependencies**: `reqwest` (rustls-tls), `zip`, `serde` added to `kami-cli`
- **9 new unit tests** across download.rs (4), search.rs (3), publish.rs (2)
- **391 tests** total, clippy clean, fmt clean

### Changed (Session 21)
- `InstallArgs.path` renamed to `InstallArgs.source` — accepts local paths, URLs, or GitHub shorthand
- `install.rs` refactored: `resolve_source()` detects source type and delegates accordingly
- `shared.rs`: added `data_dir()`, `plugins_dir()`, `dirs_or_fallback()` helpers
- New `Publish` CLI subcommand wired into main.rs

### Added (Session 20 — Dead Code Cleanup)
- Removed dead code: `context.rs`, `linker.rs`, duplicate `create_tool_linker`, `KamiTool` re-export
- Coverage improved: 71.51% (1132/1583 lines, 380 tests)

### Added (Session 19 — Coverage Campaign: 49% → 71%)
- 161 new tests (221→382), coverage 49.09%→71.19%
- `docs/TOOL_AUTHOR_GUIDE.md`: 8-step guide for tool developers

### Added (Session 18 — E2E Demo: Full Pipeline Working)
- **Host-side WIT bindings** (`kami-engine/src/bindings.rs`): `wasmtime::component::bindgen!` with typed `KamiTool`, `Host` trait for `kami:tool/host@0.1.0`, `create_tool_linker()`, `instantiate_tool()`, `call_run()`
- **Echo tool guest** (`tests/fixtures/echo-tool/`): real Rust project compiled to `wasm32-wasip2` (66KB), implements `kami:tool/tool` interface via `wit_bindgen::generate!`
- **E2E integration tests** (`kami-runtime/tests/e2e_echo.rs`): 2 tests exercising full pipeline (parse manifest → register in SQLite → resolve → compile → sandbox → instantiate → call → result)
- **221 tests** total, clippy clean, fmt clean

### Changed (Session 18)
- `create_linker()` now registers both WASI and `kami:tool/host` interface imports
- `WasmToolExecutor` tries typed instantiation (WIT components) first, falls back to flat exports (test WAT)
- Workspace `Cargo.toml`: added `exclude = ["tests/fixtures/echo-tool"]`
- WIT files: removed duplicate `package` doc comments (blocked `bindgen!` parsing)

### Added (Session 17 — Annexe B v1.0 Progress)
- **SECURITY.md**: Root-level responsible disclosure policy (scope, process, 48h ack, 7-day fix timeline)
- **CI benchmarks**: `benchmarks` job in `.github/workflows/ci.yml` — criterion via `github-action-benchmark`, 150% alert threshold
- **wit-bindgen 0.51**: `component-model` feature flag on `kami-guest`, `bindings.rs` module with `wit_bindgen::generate!`, `Guest` trait re-export
- 25 new unit tests across 8 modules: error mapping, diagnostics, builders, serde roundtrips
- Coverage measurement: `cargo-tarpaulin` configured, 49.09% (759/1546 lines)
- **219 tests** total (+ 3 ignored doc-tests), clippy clean, fmt clean

### Changed (Session 17)
- `wit-bindgen` workspace dep: 0.36 → 0.51 (aligns with wasmtime 27, avoids ABI mismatch)
- `kami-guest/Cargo.toml`: `component-model` feature, `wit-bindgen` optional dep
- `kami-types/event.rs`: expanded from 1 to 4 tests (all constructors + serde roundtrip)

### Added (Session 16 — Sprint 3.3 + 3.4: Fuzzing & Benchmarks)
- **cargo audit**: `.cargo/audit.toml` with documented ignore list (2 low-severity wasmtime advisories + 2 unmaintained transitive deps)
- **proptest fuzzing** (13 property-based tests):
  - `kami-sandbox/tests/fuzz_filesystem.rs` — 4 tests: arbitrary paths, absolute rejection, `..` rejection, simple relative acceptance
  - `kami-config/tests/fuzz_manifest.rs` — 4 tests: arbitrary input no-panic, valid TOML parses, missing sections fail
  - `kami-protocol/tests/fuzz_jsonrpc.rs` — 5 tests: arbitrary JSON no-panic, round-trip serde, missing fields fail
- **criterion benchmarks**: `kami-runtime/benches/cache_bench.rs` — 4 benchmarks (insert_100, get_hit, get_miss, evict_lru_10)
- `examples/http-fetch/` — 4th example tool: URL scheme validation, max_bytes limit, 6 tests
- Workspace dependencies: `proptest = "1"`, `criterion = { version = "0.5", features = ["html_reports"] }`
- **194 tests** total (+ 3 ignored doc-tests), clippy clean, fmt clean, audit clean

### Changed (Session 16)
- `orchestrator.rs` compacted from 154 to 147 lines (merged imports, trimmed accessor docs)
- `tool.rs` compacted from 154 to 149 lines (trimmed `wasm_sha256` doc comment)

### Added (Session 15 — Sprint 7.2: Versioning & Update)
- `kami update <tool-id>` command — re-reads tool.toml, recomputes WASM SHA-256, updates registry; supports `--all` flag (skips pinned tools)
- `kami pin <tool-id> [version]` command — locks tool version to prevent updates; `--unpin` flag to remove pin
- `ToolRepository::update()` trait method in `kami-registry` — enables tool record updates through the port abstraction
- `kami-store-sqlite`: DB migration v3 — adds `pinned_version TEXT` and `updated_at TEXT` columns to `tools` table
- `kami-store-sqlite`: `SqliteToolRepository::update()` implementation with full 12-column UPDATE SQL
- `kami-store-sqlite/query_builder.rs` — extracted `build_find_all_query()` to keep `repository_impl.rs` under 150 lines
- `kami-types/Tool`: `pinned_version: Option<String>` and `updated_at: Option<String>` fields (serde skip_serializing_if = None)
- `examples/json-transform/` — 3rd example tool: pick keys, flatten nested objects (dot-notation), count keys (5 tests)
- `crates/kami-transport-stdio/tests/mcp_roundtrip.rs` — 4 end-to-end MCP integration tests (initialize, tools/list, unknown method error, full CRUD lifecycle with pinning)
- `docs/INTEGRATION.md` — AI agent integration guide (Claude Desktop, Cursor, HTTP transport, LangChain/Python, troubleshooting)
- `docs/GETTING_STARTED.md` — quick start guide with installation, tool.toml reference, CLI commands table, examples
- `.github/ISSUE_TEMPLATE/` — bug_report.md, feature_request.md, security.md templates
- `CODE_OF_CONDUCT.md` — Contributor Covenant 2.1
- `output::print_info()` helper in `kami-cli`
- **176 tests** total (+ 3 ignored doc-tests), clippy clean, fmt clean

### Changed (Session 15)
- `kami-store-sqlite`: `COLS` constant now includes `pinned_version, updated_at`; made `pub(crate)` for query_builder module access
- `kami-store-sqlite/row_mapping.rs`: `row_to_tool()` reads columns 10 (pinned_version) and 11 (updated_at)
- `kami-cli/main.rs`: Commands enum extended with `Update` and `Pin` variants (14 commands total)
- All `Tool {}` constructions updated across 8+ files with `pinned_version: None, updated_at: None`
- All mock `ToolRepository` implementations updated with `update()` method (EmptyRepository, MockRepository)

### Changed (Session 14 — v0.2 Compliance Audit)
- **LRU cache eviction**: `ComponentCache` now uses proper LRU tracking (order Vec + touch-on-access) instead of FIFO; switched from `RwLock` to `Mutex` since reads now update access order
- **150-line rule**: 7 files split — `dev.rs`, `metrics.rs`, `rate_limiter.rs`, `capability.rs`, `repository.rs`, `integration.rs`, `tests/repository.rs`
- **Repository split**: `SqliteToolRepository` struct/constructors in `repository.rs` (45 lines), `ToolRepository` trait impl in `repository_impl.rs` (125 lines)
- **Engine test refactor**: 343-line `integration.rs` → `engine_basic.rs` + `engine_isolation.rs` + `common/mod.rs` (shared WAT fixture)
- Removed `#[allow(dead_code)]` from `main.rs`

### Added (Session 14)
- `CONTRIBUTING.md` — code standards, commit format, PR checklist, architecture overview
- `lru_evicts_least_recently_used` test in `tests/cache.rs`
- **158 tests** total (+ 3 ignored doc-tests), clippy clean

### Added (Session 13)
- `kami dev watch <tool-dir>` command — filesystem watcher using `notify 6.1`, debounced rebuilds on `.rs`/`.toml` changes, optional `--release` flag (3 unit tests)
- `examples/hello-world/` — minimal KAMI tool: takes `{"name":"..."}`, returns greeting (Cargo.toml, tool.toml, src/lib.rs, README.md, 2 tests)
- `examples/echo/` — echo tool: validates JSON and returns input unchanged (Cargo.toml, tool.toml, src/lib.rs, README.md, 3 tests)
- `DiagnosticError` trait in `kami-types` — `hint()` + `fix()` for user-actionable error messages; implemented on `EngineError`, `SandboxError`, `RuntimeError`
- `kami exec` now prints diagnostic hints and fix suggestions on execution errors
- `kami-runtime/rate_limiter.rs` — token-bucket rate limiter with per-tool and global limits (4 tests)
- `kami-runtime/pipeline.rs` — multi-tool pipeline execution: sequential step chaining with `input_from: "previous"` (4 tests)
- `RuntimeError::RateLimited` variant for rate limit rejections
- Workspace dependencies: `notify = "6.1"`, `serde` added to `kami-runtime`

### Added (Session 12)
- `kami-transport-http` crate (new): axum 0.7 HTTP server for MCP JSON-RPC
  - `POST /mcp` — dispatches JSON-RPC requests to the existing `McpHandler`
  - `GET /health` — liveness probe returning `{"status":"ok","service":"kami"}`
  - Bearer token authentication via `Authorization: Bearer <token>` header
  - 7 unit tests (4 auth, 3 server constructor)
- `kami serve --transport http --port 3000 --token <secret>` — HTTP transport mode
- `kami status` command — shows tool registry stats (total/enabled/disabled) + runtime config (concurrency, cache size, version)

### Changed (Session 12)
- `kami serve` now supports `--transport stdio|http`, `--port`, and `--token` flags (backward-compatible: default is stdio)
- Workspace `Cargo.toml`: added `axum = "0.7"` and `kami-transport-http` internal crate

### Added (Session 11)
- `kami-cli/init.rs`: `execute_at(args, base_dir)` helper — enables testable `kami init` without CWD manipulation
- `kami-cli/init.rs`: 2 unit tests — `init_creates_expected_files` (verifies all 4 files exist + content correctness) and `init_fails_if_dir_exists` (conflict detection)
- `templates.rs`: corrected `kami-guest` path from `../../crates/kami-guest` to `../crates/kami-guest` (fix for tool projects scaffolded alongside the KAMI workspace)

### Added (Session 10)
- `kami-runtime/metrics.rs`: `ExecutionMetrics` with `AtomicU64` counters (total/success/failed executions, fuel consumed, cache hits/misses) — zero-overhead, thread-safe recording
- `kami-runtime/metrics.rs`: `MetricsSnapshot` copyable struct for reading counters atomically
- `KamiRuntime::metrics()` — returns `Arc<ExecutionMetrics>` for live metric access
- `KamiRuntime::shutdown()` — drains in-flight executions via `Scheduler::drain()` before returning
- `Scheduler::drain()` — acquires all permits to guarantee all executing tasks have completed
- `kami-cli`: `--log-format plain|json` global flag — JSON output enables structured logging for ELK/Datadog aggregation
- `.github/workflows/release.yml` — binary release workflow for 4 targets (Linux x86_64, macOS Intel/ARM, Windows)
- `.github/workflows/ci.yml` — enhanced with documentation build check (`RUSTDOCFLAGS=-D warnings`) and coverage job (cargo-tarpaulin + Codecov)
- Graceful shutdown in `kami serve` — `tokio::select!` races server loop against `ctrl_c` signal

### Changed (Session 10)
- `KamiRuntime::execute()`: records metrics at all exit points using `inspect_err` (attempt, hit/miss, success/failure, fuel) — failures during resolve/schedule are also counted
- `kami-runtime/lib.rs`: exports `ExecutionMetrics` and `MetricsSnapshot`
- `tracing-subscriber` feature list in workspace `Cargo.toml`: added `json` for JSON log format
- `kami-cli/serve.rs`: handler now clones `Arc<KamiRuntime>` for graceful shutdown ownership

### Security (Session 9)
- **WASM Integrity**: SHA-256 hash computed at install time (`kami install`) and verified before execution — prevents tampering between install and run
- `kami-runtime`: `RuntimeError::IntegrityViolation` maps to `KamiError::PermissionDenied`

### Added (Session 9)
- `kami-runtime/integrity.rs`: `compute_file_hash(path)` and `verify_hash(path, expected)` using `sha2` + `hex` crates
- `kami-types/ToolManifest`: `wasm_sha256: Option<String>` field (backward-compatible, skipped in serialization when `None`)
- `kami-store-sqlite`: Schema migration v2 — adds `wasm_sha256 TEXT` column to `tools` table
- `kami-cli`: `kami verify <tool-id>` command — compares stored vs actual SHA-256, reports OK/WARN/FAIL
- `kami-cli/output.rs`: `print_warning()` helper
- `kami-runtime`: `#[tracing::instrument]` on `WasmToolExecutor::execute`, `ToolResolver::resolve`, `KamiRuntime::execute`, `McpHandler::dispatch`
- `kami-config/loader.rs`: 4 unit tests for `load_config` (defaults, values, missing file, timeout duration)
- `kami-runtime/tests/resolver.rs`: 3 integration tests (tool not found, missing WASM file, integrity violation)
- `kami-runtime/tests/orchestrator.rs`: 4 integration tests (not found, cache invalidate, scheduler permits, default config)

### Changed (Session 9)
- `kami-runtime/types.rs`: Extracted `ExecutionResult` + `ToolExecutor` trait from executor; trait signature aligned with `execute_component` (accepts `&Component`, `&str`, `&SecurityConfig`)
- `kami-runtime/runtime_config.rs`: Extracted `RuntimeConfig` from orchestrator; added `Default` impl (cache=32, concurrency=4, epoch=true)
- `kami-runtime/executor.rs`: `WasmToolExecutor` now implements `ToolExecutor` trait; `execute_component` method removed (use trait method `execute`)
- `kami-runtime/resolver.rs`: Step 4 now verifies WASM hash before compilation
- `kami-cli/install.rs`: Computes and stores SHA-256 hash in manifest after verifying WASM file exists
- `kami-types/version.rs`: Extracted `ToolVersion` Display/FromStr into its own module
- Tests split to integration files: `kami-runtime/tests/cache.rs`, `kami-sandbox/tests/network.rs`, `kami-config/tests/manifest_loader.rs`, `kami-protocol/tests/jsonrpc.rs`

### Security
- **HIGH**: Fixed path traversal in `FsJail::validate_path` — now rejects absolute/rooted paths (including Unix-style on Windows via `has_root()`), rejects `..` components, and verifies containment via `canonicalize()` (anti-symlink)
- **HIGH**: Fixed IP address bypass in network allow-list — `is_addr_allowed(SocketAddr)` requires explicit IP literals in the allow-list; hostname wildcard patterns no longer match IP addresses
- **MEDIUM**: Fixed `env_allow_list` not being enforced — `WasiCtxBuilder` now filters environment variables against the configured allow-list
- **LOW**: Fixed SQL injection risk in `find_all` LIMIT/OFFSET — now uses bound rusqlite parameters instead of `format!()`
- Fixed `unwrap_or_default()` on deserialisation of corrupt SQLite columns — errors now propagate as `RepositoryError::DataCorruption`

### Added
- `kami-transport-stdio`: `McpHandler::handle_notification()` for silent handling of JSON-RPC notifications
- `kami-transport-stdio`: `dispatch/` sub-module with `initialize`, `tools_list`, `tools_call` handlers as free functions
- `kami-transport-stdio`: MCP `notifications/initialized` compliance — server falls back to `JsonRpcNotification` parse for messages without an `id` field (no response sent)
- `kami-store-sqlite`: `row_mapping.rs` module with `row_to_tool` and `OptionalExt` helpers
- `kami-store-sqlite`: `tests/repository.rs` integration test file (8 tests)
- `kami-cli`: `shared.rs` module with `open_repository()` and `create_runtime()` helpers
- `kami-cli`: `commands/templates.rs` with `__PLACEHOLDER__` substitution for `kami init` scaffolding
- `kami-registry`: `RepositoryError::DataCorruption` variant for corrupt storage

### Changed
- `kami-cli`: All async commands converted from `Runtime::new().block_on()` to `pub async fn execute()` with `#[tokio::main]` on `main()`
- `kami-cli`: All commands using the registry now use `shared::open_repository()` instead of duplicating the open pattern
- `kami-config`: `parse_tool_manifest_file()` and `parse_tool_manifest()` moved here from `kami-types` (Clean Architecture: I/O belongs in adapters)
- `kami-types`: `manifest.rs` is now documentation-only; `toml` dependency removed
- `kami-sandbox/filesystem`: `FsJail::validate_path` now rejects paths via `has_root() || is_absolute()` for cross-platform correctness

### Added (prior sessions)
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
