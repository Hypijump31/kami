<p align="center"><img src="../site/assets/logo.png" alt="KAMI" height="48"></p>

# Getting Started with KAMI

KAMI is a secure WASM tool orchestrator that runs sandboxed tools via
the Model Context Protocol (MCP).

## Installation

### From source

```bash
git clone https://github.com/Hypijump31/kami.git
cd kami
cargo build --release
# Binary at target/release/kami
```

### Pre-built binaries

Download from the [Releases](https://github.com/Hypijump31/kami/releases)
page. Available for Linux (x86_64, aarch64), macOS (aarch64), and
Windows (x86_64).

---

## Quick Start

### 1. Create a tool project

```bash
kami init my-tool
cd my-tool
```

This generates:
- `Cargo.toml` — Rust project targeting `wasm32-wasip2`
- `tool.toml` — Tool manifest (ID, security, MCP schema)
- `src/lib.rs` — Handler function using the `kami-guest` SDK

### 2. Build the tool

```bash
cargo build --target wasm32-wasip2 --release
```

### 3. Install into the registry

```bash
# From local directory
kami install .

# From a URL
kami install https://example.com/my-tool-v1.0.0.zip

# From GitHub release
kami install owner/repo@v1.0.0
```

### 4. Search for community tools

```bash
kami search "json transform"
# Found 1 tool(s):
#   dev.kami.json-transform v1.0.0 — Pick, flatten, or count JSON keys
#     install: kami install kami-tools/json-transform@v1.0.0
```

### 5. Execute the tool

```bash
kami exec dev.example.my-tool '{"key": "value"}'
```

### 6. Serve tools to AI agents

```bash
kami serve --transport stdio
```

---

## Install Sources

KAMI supports three install sources:

| Source | Example | Description |
|--------|---------|-------------|
| **Local path** | `./my-tool/` | Directory with `tool.toml` + `.wasm` |
| **URL** | `https://host/tool.zip` | Downloads and extracts a `.zip` archive |
| **GitHub** | `owner/repo@v1.0.0` | Downloads from GitHub releases |

---

## Tool Manifest (tool.toml)

```toml
[tool]
id = "dev.example.fetch-url"
name = "fetch-url"
version = "1.0.0"
wasm = "target/wasm32-wasip2/release/fetch_url.wasm"

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

---

## CLI Commands

| Command       | Description                              |
|---------------|------------------------------------------|
| `kami init`    | Scaffold a new tool project             |
| `kami install` | Install a tool (local, URL, or GitHub)  |
| `kami search`  | Search remote registry for tools        |
| `kami publish` | Generate a registry entry to publish     |
| `kami list`    | List installed tools                    |
| `kami exec`    | Execute a tool by ID                    |
| `kami run`     | Run a WASM file directly (dev mode)     |
| `kami update`  | Update tools from their source dirs     |
| `kami pin`     | Pin a tool version (prevent updates)    |
| `kami verify`  | Verify WASM integrity (SHA-256 + Ed25519) |
| `kami keygen`  | Generate Ed25519 signing keypair         |
| `kami sign`    | Sign a WASM plugin with Ed25519          |
| `kami serve`   | Start MCP server (stdio or HTTP)        |
| `kami inspect` | Show tool manifest details              |
| `kami status`  | Runtime and registry statistics         |

---

## Examples

Four example tools are included in the `examples/` directory:

| Example          | Description                             |
|-----------------|-----------------------------------------|
| `hello-world`   | Minimal greeting tool                   |
| `echo`           | Echoes JSON input unchanged            |
| `json-transform` | Pick keys, flatten, or count           |
| `http-fetch`    | Validate and fetch from allow-listed URLs |

```bash
cd examples/hello-world
cargo build --target wasm32-wasip2 --release
kami install .
kami exec dev.kami.hello-world '{"name": "Alice"}'
```

---

## Security Model

KAMI enforces **deny-all by default**:

- No filesystem access unless explicitly granted
- No network access unless hostnames are allow-listed
- Memory and execution time limits per tool
- WASM SHA-256 integrity verification
- Ed25519 cryptographic plugin signatures
- Environment variable filtering

See [SECURITY.md](SECURITY.md) for full details.
