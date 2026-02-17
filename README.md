# KAMI

**WASM Tool Orchestrator for AI Agents**

[![CI](https://github.com/Hypijump31/kami/workflows/CI/badge.svg)](https://github.com/Hypijump31/kami/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Overview

KAMI is a high-performance orchestrator that enables AI agents to execute third-party tools **securely** and **isolated** via WebAssembly Component Model and the MCP (Model Context Protocol).

```
┌─────────────┐     MCP (JSON-RPC)     ┌─────────────┐
│  AI Agent   │ ◄───────────────────► │    KAMI     │
└─────────────┘                        └──────┬──────┘
                                              │
                                              ▼
                                    ┌─────────────────┐
                                    │  WASM Sandbox   │
                                    │  (Capability-   │
                                    │   based)        │
                                    └─────────────────┘
```

## Key Features

- 🔒 **Secure by Default** - Zero-trust sandbox with explicit capability grants
- ⚡ **High Performance** - Wasmtime runtime with async I/O via Tokio
- 🧩 **Component Model** - WASI P2 support for modern WASM components
- 🔌 **MCP Protocol** - Standard JSON-RPC 2.0 interface for AI integration
- 📦 **Tool Registry** - SQLite-based catalog with version management
- 🛡️ **Resource Limits** - CPU, memory, and time constraints per tool

## Architecture

```
kami/
├── crates/
│   ├── kami-types/           # Domain types (zero deps)
│   ├── kami-protocol/        # MCP protocol types
│   ├── kami-engine/          # Wasmtime runtime
│   ├── kami-sandbox/         # Capability isolation
│   ├── kami-runtime/         # Orchestrator
│   ├── kami-registry/        # Repository traits
│   ├── kami-store-sqlite/    # SQLite adapter
│   ├── kami-transport-stdio/ # stdio transport
│   ├── kami-config/          # Configuration
│   ├── kami-cli/             # CLI interface
│   └── kami-guest/           # Tool SDK
└── wit/                      # WIT interfaces
```

## Quick Start

### Installation

```bash
# From source
git clone https://github.com/Hypijump31/kami.git
cd kami
cargo build --release

# Run CLI
cargo run -p kami-cli -- --help
```

### Basic Usage

```bash
# List installed tools
kami list

# Install a tool
kami install ./path/to/tool.wasm

# Run a tool
kami run fetch-url --url "https://example.com"

# Inspect a tool manifest
kami inspect fetch-url
```

## Tool Manifest (tool.toml)

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
net_allow_list = ["*.example.com"]
fs_access = "none"
max_memory_mb = 64
max_execution_ms = 5000
```

## MCP Integration

KAMI implements the [Model Context Protocol](https://modelcontextprotocol.io/) for seamless AI agent integration:

```json
// Request
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "fetch-url",
    "arguments": { "url": "https://example.com" }
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [{ "type": "text", "text": "..." }]
  }
}
```

## Security Model

| Layer | Protection |
|-------|------------|
| **Network** | Explicit allow-list, deny-all by default |
| **Filesystem** | Sandboxed or no access |
| **Memory** | Configurable limits per tool |
| **CPU** | Fuel-based execution limits |
| **Time** | Execution timeout enforcement |

## Development

### Prerequisites

- Rust 1.75+
- Cargo

### Commands

```bash
cargo build                    # Build all crates
cargo test                     # Run tests
cargo clippy -- -D warnings    # Lint
cargo fmt --check              # Format check
cargo doc --no-deps --open     # Generate docs
```

### Creating a Tool

```rust
use kami_guest::prelude::*;

#[kami_tool]
fn fetch_url(url: String) -> Result<String, ToolError> {
    // Your implementation
    Ok(format!("Fetched: {}", url))
}
```

## Roadmap

- [x] Project structure
- [ ] Core WASM engine
- [ ] Sandbox isolation
- [ ] Tool registry
- [ ] MCP protocol
- [ ] CLI interface
- [ ] Tool SDK

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting PRs.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

**KAMI** - 神 (Japanese for "god/spirit") - Empowering AI with secure tool execution.
