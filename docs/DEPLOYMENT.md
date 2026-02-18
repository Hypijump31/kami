# KAMI - Deployment & Operations Guide

> Build, install, configure, and integrate KAMI in production.

---

## Table of Contents

1. [Building from Source](#building-from-source)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [CLI Reference](#cli-reference)
5. [MCP Server Deployment](#mcp-server-deployment)
6. [AI Agent Integration](#ai-agent-integration)
7. [Registry Management](#registry-management)
8. [Monitoring & Observability](#monitoring--observability)
9. [Production Checklist](#production-checklist)

---

## Building from Source

### Prerequisites

| Requirement | Version | Purpose |
|------------|---------|---------|
| Rust | 1.75+ | Compilation |
| Cargo | (bundled) | Build system |
| C compiler | gcc/clang/MSVC | SQLite bundled compilation |

### Build

```bash
git clone https://github.com/Hypijump31/kami.git
cd kami

# Development build
cargo build

# Release build (optimized)
cargo build --release

# Binary location
ls target/release/kami        # Linux/macOS
ls target/release/kami.exe    # Windows
```

### Verify

```bash
# Run all tests (89+)
cargo test

# Lint (must be clean)
cargo clippy -- -D warnings

# Check binary
./target/release/kami --version
# Output: kami 0.1.0
```

### Install System-Wide

```bash
# Option 1: cargo install
cargo install --path crates/kami-cli

# Option 2: manual copy
cp target/release/kami /usr/local/bin/

# Verify
kami --version
```

---

## Installation

### Directory Structure

KAMI uses a data directory for the registry database:

```
$KAMI_DATA_DIR/          # Default: .kami/ (relative to CWD)
└── registry.db          # SQLite tool registry
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `KAMI_DATA_DIR` | `.kami` | Data directory for registry.db |

### First Run

```bash
# Create data directory
mkdir -p .kami

# Verify CLI works
kami --help
kami list
```

The SQLite database is created automatically on first use.

---

## Configuration

### Configuration File

KAMI uses `config/default.toml` for default values:

```toml
[runtime]
max_concurrent = 10          # Max parallel tool executions
pool_size = 5                # Instance pool size
default_timeout_secs = 30    # Default tool timeout

[sandbox]
default_max_memory_mb = 64   # Default memory limit per tool
default_max_fuel = 1000000   # Default instruction budget

[registry]
database_path = "kami.db"    # Registry database path

[logging]
level = "info"               # Log level: trace|debug|info|warn|error
```

### CLI Overrides

All config can be overridden via CLI flags:

```bash
# Custom config file
kami -c /path/to/config.toml serve

# Custom database path
kami serve --db /var/lib/kami/registry.db

# Custom concurrency
kami serve --concurrency 8 --cache-size 64

# Verbosity
kami -v serve     # debug
kami -vv serve    # trace
```

---

## CLI Reference

### Global Options

| Flag | Short | Description |
|------|-------|-------------|
| `--config <PATH>` | `-c` | Configuration file path |
| `--verbose` | `-v` | Increase log verbosity (-v, -vv, -vvv) |
| `--help` | `-h` | Show help |
| `--version` | `-V` | Show version |

### Commands

#### `kami install <PATH>`

Install a WASM tool from a directory or tool.toml file.

```bash
kami install ./my-tool/            # From directory
kami install ./my-tool/tool.toml   # From manifest file
kami install --db ./reg.db ./tool/ # Custom database
```

| Option | Description |
|--------|-------------|
| `--db <PATH>` | Custom database path |

#### `kami list [--filter NAME]`

List installed tools.

```bash
kami list                          # All tools
kami list --filter fetch           # Filter by name
kami list --db ./reg.db            # Custom database
```

| Option | Description |
|--------|-------------|
| `--filter <NAME>` | Filter by name substring |
| `--db <PATH>` | Custom database path |

#### `kami inspect <TOOL_ID>`

Show detailed information about an installed tool.

```bash
kami inspect dev.example.fetch-url
```

Output includes: version, description, WASM path, security config, resource limits, arguments.

#### `kami run <WASM_FILE>`

Run a WASM component directly (development/debugging).

```bash
kami run ./my_tool.wasm --input '{"key":"value"}'
kami run ./tool.wasm --fuel 2000000 --max-memory-mb 128 --timeout-ms 10000

# Input from a JSON file
kami run ./my_tool.wasm --input-file request.json

# Input from stdin
echo '{"key":"value"}' | kami run ./my_tool.wasm --input-file -
```

| Option | Default | Description |
|--------|---------|-------------|
| `-i, --input <JSON>` | `{}` | JSON input string |
| `-f, --input-file <PATH>` | — | Read JSON input from file (`-` for stdin) |
| `--fuel <N>` | 1000000 | Fuel limit |
| `-m, --max-memory-mb <N>` | 64 | Memory limit in MB |
| `-t, --timeout-ms <N>` | 5000 | Execution timeout in ms |

> **Note:** `--input-file` takes precedence over `--input`. The file content is validated as JSON before execution.

#### `kami exec <TOOL_ID>`

Execute a registered tool by ID (production path).

```bash
kami exec dev.example.fetch-url --input '{"url":"https://example.com"}'

# Input from a JSON file
kami exec dev.example.fetch-url --input-file request.json

# Input from stdin
cat request.json | kami exec dev.example.fetch-url --input-file -
```

| Option | Default | Description |
|--------|---------|-------------|
| `-i, --input <JSON>` | `{}` | JSON input string |
| `-f, --input-file <PATH>` | — | Read JSON input from file (`-` for stdin) |
| `--concurrency <N>` | 4 | Max concurrent executions |
| `--cache-size <N>` | 32 | Component cache size |
| `--db <PATH>` | `.kami/registry.db` | Database path |

#### `kami serve`

Start MCP server over stdio.

```bash
kami serve
kami serve --concurrency 8 --cache-size 64
kami serve --db /var/lib/kami/registry.db
```

| Option | Default | Description |
|--------|---------|-------------|
| `--concurrency <N>` | 4 | Max concurrent tool executions |
| `--cache-size <N>` | 32 | Component cache size |
| `--db <PATH>` | `.kami/registry.db` | Database path |

---

## MCP Server Deployment

### Standalone Mode

```bash
# Start MCP server (blocks on stdin)
kami serve --concurrency 4

# Test with a JSON-RPC request
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | kami serve
```

### Process Manager (systemd)

```ini
# /etc/systemd/system/kami-mcp.service
[Unit]
Description=KAMI MCP Server
After=network.target

[Service]
Type=simple
User=kami
WorkingDirectory=/var/lib/kami
ExecStart=/usr/local/bin/kami serve --db /var/lib/kami/registry.db --concurrency 8
StandardInput=socket
StandardOutput=socket
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

### Docker

```dockerfile
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/kami /usr/local/bin/
RUN mkdir -p /var/lib/kami
WORKDIR /var/lib/kami
ENTRYPOINT ["kami"]
CMD ["serve", "--db", "/var/lib/kami/registry.db"]
```

---

## AI Agent Integration

### Claude Desktop

Add to Claude Desktop's MCP configuration (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "kami": {
      "command": "kami",
      "args": ["serve", "--db", "/path/to/registry.db"]
    }
  }
}
```

### Cursor

Add to Cursor's MCP settings:

```json
{
  "mcpServers": {
    "kami": {
      "command": "/usr/local/bin/kami",
      "args": ["serve"]
    }
  }
}
```

### Custom Integration

Spawn KAMI as a subprocess and communicate via stdin/stdout:

```python
import subprocess
import json

proc = subprocess.Popen(
    ["kami", "serve"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    text=True
)

# Initialize
request = {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {"name": "my-agent", "version": "1.0"}
}}
proc.stdin.write(json.dumps(request) + "\n")
proc.stdin.flush()
response = json.loads(proc.stdout.readline())
print(response)

# List tools
request = {"jsonrpc": "2.0", "id": 2, "method": "tools/list"}
proc.stdin.write(json.dumps(request) + "\n")
proc.stdin.flush()
tools = json.loads(proc.stdout.readline())
print(tools)
```

---

## Registry Management

### Tool Lifecycle

```
install ──► enabled ──► exec/serve
                │
                ▼
           (future: disable/update/uninstall)
```

### Database Backup

```bash
# SQLite database is a single file
cp .kami/registry.db .kami/registry.db.backup

# Or use SQLite backup
sqlite3 .kami/registry.db ".backup backup.db"
```

### Database Inspection

```bash
# List all tools via SQL
sqlite3 .kami/registry.db "SELECT id, version, enabled FROM tools;"

# Check schema version
sqlite3 .kami/registry.db "PRAGMA user_version;"

# Full tool details
sqlite3 .kami/registry.db "SELECT id, security FROM tools;" | head
```

---

## Monitoring & Observability

### Structured Logging

KAMI uses `tracing` for structured logging:

```bash
# Info level (default)
kami serve
# Output: INFO kami: MCP server ready on stdio

# Debug level
kami -v serve
# Output: DEBUG kami: dispatching MCP request method="tools/list"

# Trace level
kami -vv serve
# Output: TRACE transport: read message len=128
```

### Key Log Events

| Event | Level | Meaning |
|-------|-------|---------|
| `executing tool` | INFO | Tool execution started |
| `execution complete` | INFO | Tool finished (with duration, fuel) |
| `execution failed` | WARN | Tool returned error |
| `cache hit` | DEBUG | Component loaded from cache |
| `compiling component` | INFO | WASM being compiled (cold start) |
| `dispatching MCP request` | DEBUG | JSON-RPC method received |
| `stdin closed, shutting down` | INFO | Server shutting down |

### Metrics (from ExecutionResult)

Each execution returns:

| Field | Type | Description |
|-------|------|-------------|
| `duration_ms` | u64 | Wall-clock execution time |
| `fuel_consumed` | u64 | Instructions executed |
| `success` | bool | Whether tool returned Ok |
| `content` | String | Tool output or error message |

---

## Production Checklist

### Before Deployment

- [ ] Build with `--release` flag
- [ ] Run `cargo test` — all tests pass
- [ ] Run `cargo clippy -- -D warnings` — zero warnings
- [ ] Review all installed tools' security configs via `kami inspect`
- [ ] Set appropriate `--concurrency` for your hardware
- [ ] Configure `KAMI_DATA_DIR` to a persistent volume
- [ ] Back up registry.db before upgrades

### Runtime Monitoring

- [ ] Monitor stderr for WARN/ERROR log entries
- [ ] Track execution durations for performance regressions
- [ ] Track fuel consumption for tools approaching limits
- [ ] Monitor disk usage of registry.db

### Security

- [ ] Run KAMI as a non-root user
- [ ] Restrict file permissions on registry.db
- [ ] Review tool.toml manifests before installing third-party tools
- [ ] Keep Wasmtime dependency updated for security patches
