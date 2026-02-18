<p align="center">
  <br>
  <strong>KAMI</strong> &mdash; 神
  <br>
  <em>Secure WASM Tool Orchestrator for AI Agents</em>
  <br><br>
  <a href="https://github.com/Hypijump31/kami/actions"><img src="https://github.com/Hypijump31/kami/workflows/CI/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.75+-orange.svg" alt="Rust"></a>
  <a href="https://modelcontextprotocol.io/"><img src="https://img.shields.io/badge/Protocol-MCP-green.svg" alt="MCP"></a>
</p>

---

KAMI enables AI agents to execute third-party tools **securely** and **isolated** via WebAssembly Component Model and the [Model Context Protocol](https://modelcontextprotocol.io/) (MCP).

```
                     ┌───────────────────────────────────────────────────┐
                     │                    KAMI                           │
 ┌──────────┐       │  ┌──────────┐  ┌──────────┐  ┌───────────────┐  │
 │ AI Agent │◄─MCP──┤  │ Protocol │─►│ Runtime  │─►│ WASM Sandbox  │  │
 │ (Claude, │  stdio │  │ Dispatch │  │ (cache,  │  │ (deny-all,    │  │
 │  GPT...) │───────►│  │ JSON-RPC │  │ schedule)│  │  fuel, epoch) │  │
 └──────────┘       │  └──────────┘  └────┬─────┘  └───────────────┘  │
                     │                     │                            │
                     │               ┌─────▼──────┐                    │
                     │               │  Registry   │                    │
                     │               │  (SQLite)   │                    │
                     │               └─────────────┘                    │
                     └───────────────────────────────────────────────────┘
```

## Why KAMI?

| Problem | KAMI's Solution |
|---------|----------------|
| AI tools can access anything on the host | **Deny-all sandbox** with explicit capability grants |
| No standard tool interface for AI agents | **MCP protocol** (JSON-RPC 2.0) — works with Claude, Cursor, etc. |
| Native plugins crash the host | **WASM isolation** — tools can't corrupt host memory |
| No resource control over third-party code | **Fuel metering + memory limits + execution timeouts** |
| Hard to distribute and manage AI tools | **Tool registry** with `tool.toml` manifest and SQLite catalog |

## Features

- **Zero-Trust Security** — Network deny-all, filesystem jail, env var allow-list
- **Wasmtime v27** — Component Model + WASI P2 for modern WASM components
- **MCP Server** — `kami serve` exposes tools via stdio JSON-RPC
- **Tool Registry** — Install, list, inspect tools from SQLite catalog
- **Resource Limits** — Per-tool memory cap, fuel metering, epoch-based timeouts
- **Component Cache** — Pre-compiled WASM caching for instant warm starts
- **Concurrency Control** — Semaphore-based scheduler limits parallel executions
- **Guest SDK** — `kami_tool!` macro for tool developers

## Quick Start

### Build

```bash
git clone https://github.com/Hypijump31/kami.git
cd kami
cargo build --release
```

### Install a Tool

```bash
# Install from a directory containing tool.toml + .wasm
kami install ./my-tool/

# List installed tools
kami list

# Inspect tool details
kami inspect dev.example.fetch-url
```

### Execute a Tool

```bash
# Direct execution (development)
kami run ./my-tool.wasm --input '{"url":"https://example.com"}'

# Execute by registry ID (production)
kami exec dev.example.fetch-url --input '{"url":"https://example.com"}'

# Input from a JSON file
kami exec dev.example.fetch-url --input-file request.json

# Input from stdin
echo '{"url":"https://example.com"}' | kami run ./my-tool.wasm --input-file -
```

### Start MCP Server

```bash
# Start stdio server (for AI agent integration)
kami serve

# With custom concurrency and database
kami serve --concurrency 8 --db ./my-registry.db
```

## Tool Manifest

Every tool is described by a `tool.toml`:

```toml
[tool]
id = "dev.example.fetch-url"
name = "fetch-url"
version = "1.0.0"
wasm = "fetch_url.wasm"

[mcp]
description = "Fetches content from a URL"

[[mcp.arguments]]
name = "url"
type = "string"
description = "The URL to fetch"
required = true

[security]
net_allow_list = ["*.example.com", "api.github.com"]
fs_access = "none"          # none | read-only | sandbox
max_memory_mb = 64
max_execution_ms = 5000
max_fuel = 1000000
```

## Building a Tool

Use the `kami-guest` SDK:

```rust
use kami_guest::kami_tool;

kami_tool! {
    name: "dev.example.echo",
    version: "1.0.0",
    description: "Echoes back the input",
    handler: handle,
}

fn handle(input: &str) -> Result<String, String> {
    let args: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "echo": args }).to_string())
}
```

## MCP Integration

KAMI implements the [Model Context Protocol](https://modelcontextprotocol.io/) for AI agent integration:

```json
// → Request: list available tools
{"jsonrpc":"2.0","id":1,"method":"tools/list"}

// ← Response
{"jsonrpc":"2.0","id":1,"result":{
  "tools":[{
    "name":"dev.example.fetch-url",
    "description":"Fetches content from a URL",
    "inputSchema":{
      "type":"object",
      "properties":{"url":{"type":"string","description":"The URL to fetch"}},
      "required":["url"]
    }
  }]
}}

// → Request: call a tool
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{
  "name":"dev.example.fetch-url",
  "arguments":{"url":"https://example.com"}
}}

// ← Response
{"jsonrpc":"2.0","id":2,"result":{
  "content":[{"type":"text","text":"...page content..."}],
  "isError":false
}}
```

## Security Model

```
┌─────────────────────────────────────────────────┐
│                  KAMI Host                       │
│                                                 │
│  ┌───────────────────────────────────────────┐  │
│  │           Capability Checker              │  │
│  │  ┌─────────┐ ┌─────────┐ ┌────────────┐  │  │
│  │  │ Network │ │   FS    │ │  Env Vars  │  │  │
│  │  │deny-all │ │ jailed  │ │ allow-list │  │  │
│  │  └─────────┘ └─────────┘ └────────────┘  │  │
│  └───────────────────────────────────────────┘  │
│                                                 │
│  ┌───────────────────────────────────────────┐  │
│  │           Resource Limits                 │  │
│  │  ┌─────────┐ ┌─────────┐ ┌────────────┐  │  │
│  │  │  Fuel   │ │ Memory  │ │  Timeout   │  │  │
│  │  │metering │ │ capped  │ │   epoch    │  │  │
│  │  └─────────┘ └─────────┘ └────────────┘  │  │
│  └───────────────────────────────────────────┘  │
│                                                 │
│  ┌───────────────────────────────────────────┐  │
│  │         WASM Sandbox (Wasmtime)           │  │
│  │  ┌─────────────────────────────────────┐  │  │
│  │  │      Guest Tool (Component Model)   │  │  │
│  │  │   No host memory access. No syscalls.│  │  │
│  │  └─────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

| Layer | Protection | Default |
|-------|------------|---------|
| **Network** | Allow-list per host/pattern | Deny all |
| **Filesystem** | None / Read-only / Sandboxed | None |
| **Env vars** | Explicit allow-list | Deny all |
| **Memory** | Per-tool MB cap via `StoreLimits` | 64 MB |
| **CPU** | Fuel metering (instruction budget) | 1M fuel |
| **Time** | Epoch-based interruption | 5000 ms |

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│ INFRASTRUCTURE                                                    │
│  kami-cli (install, run, exec, list, inspect, serve)             │
├──────────────────────────────────────────────────────────────────┤
│ ADAPTERS                                                          │
│  kami-store-sqlite    kami-transport-stdio    kami-config          │
├──────────────────────────────────────────────────────────────────┤
│ APPLICATION                                                       │
│  kami-engine          kami-sandbox            kami-runtime         │
│  (wasmtime v27)      (capability checker)    (orchestrator)       │
├──────────────────────────────────────────────────────────────────┤
│ DOMAIN                                                            │
│  kami-types           kami-protocol           kami-registry        │
│  (zero deps)         (JSON-RPC, MCP)         (ToolRepository)    │
└──────────────────────────────────────────────────────────────────┘
       ▲                                              │
       └──────── Dependency flow: outer → inner ──────┘
```

### Crate Map

| Crate | Layer | Purpose |
|-------|-------|---------|
| `kami-types` | Domain | ToolId, ToolManifest, SecurityConfig, KamiError |
| `kami-protocol` | Domain | JSON-RPC 2.0, MCP types (initialize, tools/*, prompts/*) |
| `kami-registry` | Domain | `ToolRepository` trait, `ToolQuery` |
| `kami-engine` | Application | Wasmtime Component Model, fuel, StoreLimits, epoch |
| `kami-sandbox` | Application | WasiCtxBuilder, capability checker, config validation |
| `kami-runtime` | Application | Executor, Scheduler, ComponentCache, ToolResolver, KamiRuntime |
| `kami-store-sqlite` | Adapter | SQLite CRUD, migrations, JSON columns |
| `kami-transport-stdio` | Adapter | StdioTransport, McpHandler, McpServer |
| `kami-config` | Adapter | Layered config (TOML + env + defaults) |
| `kami-cli` | Infrastructure | CLI commands (install, run, exec, list, inspect, serve) |
| `kami-guest` | SDK | `kami_tool!` macro, ABI helpers for tool developers |

## CLI Commands

| Command | Description |
|---------|-------------|
| `kami install <path>` | Install a tool from directory or .toml file |
| `kami run <file.wasm>` | Run a WASM component directly (dev mode) |
| `kami exec <tool-id>` | Execute a registered tool by ID |
| `kami list [--filter name]` | List installed tools |
| `kami inspect <tool-id>` | Show detailed tool information |
| `kami serve` | Start MCP server over stdio |

## Development

### Prerequisites

- **Rust 1.75+** with `wasm32-wasip2` target
- **Cargo** (workspace-aware)

### Commands

```bash
cargo build                    # Build all 11 crates
cargo test                     # Run all tests (81+)
cargo clippy -- -D warnings    # Lint (zero warnings)
cargo fmt --check              # Format check
cargo doc --no-deps --open     # Generate docs
cargo run -p kami-cli -- serve # Start MCP server
```

### Documentation

| Document | Description |
|----------|-------------|
| [`docs/TECHNICAL.md`](docs/TECHNICAL.md) | Data flows, crate APIs, error handling, async model, wire protocol |
| [`docs/SECURITY.md`](docs/SECURITY.md) | Threat model, defense in depth, attack surface analysis |
| [`docs/DEVELOPER.md`](docs/DEVELOPER.md) | Tool developer guide: build, test, publish WASM tools |
| [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md) | Build, install, configure, AI agent integration |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Architecture Decision Records (ADR-001 to ADR-007) |

### Project Status

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 0: Foundations | Done | 11 crates, types, protocol, config |
| Phase 1: Minimal Engine | Done | Wasmtime v27, Component Model, async |
| Phase 2: Isolation | Done | Capability checker, resource limits, epoch |
| Phase 3: Registry | Done | tool.toml, SQLite CRUD, CLI commands |
| Phase 4: Runtime | Done | Cache, Resolver, Scheduler, KamiRuntime |
| Phase 5: Protocol | Done | MCP transport, JSON-RPC dispatch, serve |
| Phase 6: SDK & Docs | Done | kami-guest, kami_tool!, architecture docs |

> **95+ tests passing, zero clippy warnings.**

## Contributing

Contributions are welcome! Please read the architecture docs in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) before submitting PRs.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  <strong>KAMI</strong> 神 &mdash; <em>Empowering AI agents with secure tool execution.</em>
</p>
