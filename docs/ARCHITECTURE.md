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
