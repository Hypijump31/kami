# Architecture Decision Records

## ADR-001: Hexagonal Architecture

**Status**: Accepted

**Context**: KAMI needs a modular, testable architecture that separates concerns cleanly and allows swapping implementations (e.g., SQLite -> PostgreSQL).

**Decision**: Adopt hexagonal (ports & adapters) architecture with 4 layers:
- **Domain** (kami-types, kami-protocol, kami-registry): Pure types and traits, no I/O
- **Application** (kami-engine, kami-sandbox, kami-runtime): Business logic
- **Adapters** (kami-store-sqlite, kami-transport-stdio, kami-config): I/O implementations
- **Infrastructure** (kami-cli): Composition root

**Consequences**: Clean dependency flow (outer -> inner), easy to test with mocks, adapter swapping possible.

---

## ADR-002: Capability-Based Security

**Status**: Accepted

**Context**: WASM tools must be sandboxed. We need fine-grained control over what each tool can access.

**Decision**: Deny-all by default. Tools declare required capabilities in `tool.toml`. The sandbox checker validates each capability against the declared security config before granting access.

**Consequences**: Zero trust by default. Every network call, filesystem access, and env var read must be explicitly allowed.

---

## ADR-003: Wasmtime as WASM Runtime

**Status**: Accepted

**Context**: Need a production-grade WASM runtime with Component Model and WASI P2 support.

**Decision**: Use Wasmtime v27 with fuel metering and memory limits.

**Consequences**: Bytecode Alliance ecosystem, strong security track record, built-in fuel metering for execution limits.

---

## ADR-004: Async-First with Tokio

**Status**: Accepted

**Context**: MCP transport and tool execution need concurrent I/O.

**Decision**: Use Tokio as the async runtime. All I/O-bound operations are async. No `.block_on()` in library crates.

**Consequences**: Consistent async model, good wasmtime integration via `wasmtime-wasi`.

---

## ADR-005: MCP over Stdio with Line-Delimited JSON

**Status**: Accepted

**Context**: KAMI needs to expose tools to AI agents via the Model Context Protocol (MCP). The transport must be simple, reliable, and compatible with standard AI tool orchestrators (Claude Desktop, Cursor, etc.).

**Decision**: Use line-delimited JSON over stdin/stdout as the primary transport. Each JSON-RPC message is a single line terminated by `\n`. The `StdioTransport` is generic over reader/writer types for testability.

**Consequences**:
- Compatible with all MCP clients out of the box
- Easy to debug (pipe stdin/stdout)
- Future transports (WebSocket, SSE) can be added as additional adapter crates without changing the handler layer
- Single-threaded request processing per connection (sufficient for stdio, multiplexing needed for network transports)

---

## ADR-006: Guest SDK with Declarative Macros

**Status**: Accepted

**Context**: Tool developers need a simple way to create WASM tools without knowing the Component Model ABI details.

**Decision**: Provide `kami-guest` crate with:
- `kami_tool!` declarative macro to wire handler functions to ABI exports
- ABI helpers (`parse_input`, `to_output`, `text_result`, `error_result`) for common patterns
- `ToolMetadata` struct for the `describe` export

**Consequences**:
- No proc-macro crate needed (simpler dependency tree)
- Tool developers write a single handler function, macro generates the rest
- Future: when `wit-bindgen` matures for guest-side, the macro can be updated to generate proper Component Model bindings transparently

---

## ADR-007: Component Cache with FIFO Eviction

**Status**: Accepted

**Context**: WASM compilation is expensive (~10-100ms), but instantiation is cheap (~1ms). Caching compiled components avoids redundant compilation.

**Decision**: `ComponentCache` stores compiled `Component` objects keyed by `ToolId`. Uses `RwLock<HashMap>` for thread-safe concurrent reads with exclusive writes. FIFO eviction when capacity is reached.

**Consequences**:
- Warm starts for frequently-used tools
- Memory-bounded cache prevents unbounded growth
- Future: LRU eviction or TTL-based expiry can be added when usage patterns are clearer
