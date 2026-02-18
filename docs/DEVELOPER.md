# KAMI - Tool Developer Guide

> How to build, test, and publish WASM tools for KAMI.

---

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Tool Anatomy](#tool-anatomy)
4. [Quick Start: Echo Tool](#quick-start-echo-tool)
5. [The tool.toml Manifest](#the-tooltoml-manifest)
6. [Using kami-guest SDK](#using-kami-guest-sdk)
7. [Input/Output Contract](#inputoutput-contract)
8. [Security Declarations](#security-declarations)
9. [Testing Your Tool](#testing-your-tool)
10. [Installing and Running](#installing-and-running)
11. [MCP Integration](#mcp-integration)
12. [Best Practices](#best-practices)
13. [Troubleshooting](#troubleshooting)

---

## Overview

A KAMI tool is a **WASM Component** that:
1. Exports a `run(input: string) -> result<string, string>` function
2. Is described by a `tool.toml` manifest
3. Runs inside a sandboxed environment with declared capabilities

```
Your Rust Code ──► cargo build ──► .wasm ──► kami install ──► kami exec
                   (wasm32-wasip2)           (+ tool.toml)
```

---

## Prerequisites

```bash
# Rust toolchain with WASM target
rustup target add wasm32-wasip2

# KAMI CLI (build from source)
git clone https://github.com/Hypijump31/kami.git
cd kami && cargo build --release
export PATH="$PWD/target/release:$PATH"
```

---

## Tool Anatomy

A minimal tool project:

```
my-tool/
├── Cargo.toml          # Rust crate config
├── tool.toml           # KAMI manifest
├── src/
│   └── lib.rs          # Tool implementation
└── target/
    └── wasm32-wasip2/
        └── release/
            └── my_tool.wasm  # Compiled component
```

---

## Quick Start: Echo Tool

### 1. Create the project

```bash
cargo new --lib my-echo-tool
cd my-echo-tool
```

### 2. Configure Cargo.toml

```toml
[package]
name = "my-echo-tool"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
kami-guest = { path = "../kami/crates/kami-guest" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### 3. Write the tool (src/lib.rs)

```rust
use kami_guest::kami_tool;

kami_tool! {
    name: "dev.example.echo",
    version: "1.0.0",
    description: "Echoes back the input with a timestamp",
    handler: handle,
}

fn handle(input: &str) -> Result<String, String> {
    // Parse input as JSON
    let args: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| format!("invalid JSON: {e}"))?;

    // Build response
    let response = serde_json::json!({
        "echo": args,
        "tool": "dev.example.echo"
    });

    Ok(response.to_string())
}
```

### 4. Write the manifest (tool.toml)

```toml
[tool]
id = "dev.example.echo"
name = "echo"
version = "1.0.0"
wasm = "my_echo_tool.wasm"

[mcp]
description = "Echoes back the input with metadata"

[[mcp.arguments]]
name = "message"
type = "string"
description = "The message to echo"
required = true

[security]
fs_access = "none"
max_memory_mb = 16
max_execution_ms = 1000
max_fuel = 500000
```

### 5. Build

```bash
cargo build --target wasm32-wasip2 --release
cp target/wasm32-wasip2/release/my_echo_tool.wasm .
```

### 6. Install and run

```bash
kami install .
kami list
kami exec dev.example.echo --input '{"message":"hello world"}'
```

---

## The tool.toml Manifest

### Complete Reference

```toml
[tool]
id = "dev.example.my-tool"      # Unique ID (reverse-domain notation, REQUIRED)
name = "my-tool"                 # Human-readable name (REQUIRED)
version = "1.0.0"               # Semver version (REQUIRED)
wasm = "my_tool.wasm"           # WASM filename relative to install dir (REQUIRED)

[mcp]
description = "What this tool does"  # Shown in tools/list (REQUIRED)

[[mcp.arguments]]                # Repeatable section for each argument
name = "url"                     # Argument name (REQUIRED)
type = "string"                  # JSON Schema type: string|number|boolean|object|array (REQUIRED)
description = "The URL to fetch" # Shown in inputSchema (REQUIRED)
required = true                  # Whether the argument is required (default: false)

[[mcp.arguments]]
name = "timeout"
type = "number"
description = "Timeout in milliseconds"
required = false

[security]
net_allow_list = ["*.example.com"]  # Network hosts (default: [] = deny all)
fs_access = "none"                  # none | read-only | sandbox (default: none)
env_allow_list = ["API_KEY"]        # Env vars to expose (default: [] = deny all)
max_memory_mb = 64                  # Memory limit (default: 64)
max_execution_ms = 5000             # Timeout (default: 5000)
max_fuel = 1000000                  # Instruction budget (default: 1000000)
```

### ID Format Rules

- Must contain at least one dot (`.`)
- Use reverse-domain notation: `org.company.tool-name`
- Lowercase, hyphens allowed
- Examples: `dev.example.fetch-url`, `com.acme.summarize`, `io.github.user.my-tool`

---

## Using kami-guest SDK

### The kami_tool! Macro

```rust
use kami_guest::kami_tool;

kami_tool! {
    name: "dev.example.my-tool",
    version: "1.0.0",
    description: "Description for MCP",
    handler: my_handler_function,
}
```

This generates:
- `__kami_run(input: &str) -> Result<String, String>` — delegates to your handler
- `__kami_describe() -> String` — returns tool metadata JSON

### ABI Helpers

```rust
use kami_guest::{parse_input, to_output, text_result, error_result};

// Parse typed input from JSON
#[derive(serde::Deserialize)]
struct MyArgs { url: String, count: u32 }

fn handler(input: &str) -> Result<String, String> {
    // Parse JSON input into typed struct
    let args: MyArgs = parse_input(input)?;

    // ... your logic ...

    // Option 1: Return JSON object
    to_output(&serde_json::json!({ "result": "data" }))

    // Option 2: Return simple text
    text_result("Operation completed successfully")

    // On error: return Err(String)
    // error_result("message") is for building error JSON payloads
}
```

---

## Input/Output Contract

### Input

Your handler receives a **JSON string** representing the arguments passed by the AI agent:

```json
{"url": "https://example.com", "timeout": 5000}
```

### Output (Success)

Return `Ok(String)` with a JSON string. This becomes the `content[0].text` in the MCP response:

```rust
Ok(r#"{"data": "fetched content"}"#.to_string())
```

### Output (Error)

Return `Err(String)` with an error message. This becomes `isError: true` in the MCP response:

```rust
Err("connection refused: https://example.com".to_string())
```

### WIT Interface

```wit
interface tool {
    run: func(input: string) -> result<string, string>;
    describe: func() -> string;
}
```

---

## Security Declarations

### Principle of Least Privilege

Declare **only** what your tool needs:

```toml
# ❌ BAD: Too permissive
[security]
net_allow_list = ["*"]
fs_access = "sandbox"
env_allow_list = ["PATH", "HOME", "USER", "API_KEY", "SECRET"]
max_memory_mb = 1024

# ✅ GOOD: Minimal permissions
[security]
net_allow_list = ["api.github.com"]
fs_access = "none"
env_allow_list = ["GITHUB_TOKEN"]
max_memory_mb = 32
```

### Network Access

```toml
# No network (default)
net_allow_list = []

# Specific hosts only
net_allow_list = ["api.github.com", "api.openai.com"]

# Wildcard subdomain
net_allow_list = ["*.example.com"]  # matches foo.example.com
```

### Filesystem Access

```toml
fs_access = "none"       # No file I/O (safest, default)
fs_access = "read-only"  # Read files in sandboxed directory
fs_access = "sandbox"    # Read + write in sandboxed directory
```

### Resource Limits

| Parameter | Recommended Range | What Happens at Limit |
|-----------|-------------------|----------------------|
| `max_fuel` | 100K - 10M | WASM trap: OutOfFuel |
| `max_memory_mb` | 8 - 256 | WASM trap: memory grow failure |
| `max_execution_ms` | 500 - 30000 | WASM trap: epoch interrupt |

---

## Testing Your Tool

### Native Tests

Test your handler function natively (no WASM needed):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_input() {
        let result = handle(r#"{"message":"hello"}"#);
        assert!(result.is_ok());
        let json: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(json["echo"]["message"], "hello");
    }

    #[test]
    fn invalid_input() {
        let result = handle("not json");
        assert!(result.is_err());
    }

    #[test]
    fn empty_input() {
        let result = handle("{}");
        assert!(result.is_ok());
    }
}
```

### WASM Integration Test

```bash
# Build WASM
cargo build --target wasm32-wasip2 --release

# Test with kami run (direct execution)
kami run target/wasm32-wasip2/release/my_tool.wasm \
  --input '{"message":"test"}'

# Install and test via registry
kami install .
kami exec dev.example.my-tool --input '{"message":"test"}'
```

---

## Installing and Running

### Install

```bash
# From a directory containing tool.toml + .wasm
kami install ./my-tool/

# From a tool.toml file directly
kami install ./my-tool/tool.toml
```

### Verify

```bash
# List all tools
kami list

# Inspect details
kami inspect dev.example.my-tool
```

### Execute

```bash
# Via registry (production mode — uses cache, scheduler, full pipeline)
kami exec dev.example.my-tool --input '{"message":"hello"}'

# Direct WASM execution (dev mode — no registry, no cache)
kami run ./my_tool.wasm --input '{"message":"hello"}'

# Input from a JSON file (works with both run and exec)
kami exec dev.example.my-tool --input-file request.json
kami run ./my_tool.wasm --input-file request.json

# Input from stdin (pipe)
echo '{"message":"hello"}' | kami exec dev.example.my-tool --input-file -
```

---

## MCP Integration

Once installed, your tool is automatically available via `kami serve`:

```bash
# Start MCP server
kami serve
```

An AI agent can then:

1. **Discover** your tool via `tools/list`:
```json
{"jsonrpc":"2.0","id":1,"method":"tools/list"}
```

2. **Call** your tool via `tools/call`:
```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{
  "name":"dev.example.my-tool",
  "arguments":{"message":"hello from AI"}
}}
```

Your tool's `mcp.arguments` section generates the `inputSchema` that AI agents use to understand what parameters are expected.

---

## Best Practices

### Do

- Return structured JSON from your handler (easier for AI agents to parse)
- Handle all error cases with descriptive `Err(String)` messages
- Set `max_memory_mb` as low as practical for your workload
- Use `parse_input<T>()` for typed deserialization
- Test with both valid and malformed input

### Don't

- Don't declare network access unless your tool actually needs it
- Don't use `fs_access = "sandbox"` for read-only workloads
- Don't set `max_fuel` to very high values — 1M is sufficient for most tools
- Don't expose sensitive env vars unless strictly required
- Don't return raw error stacktraces (keep messages user-friendly)

---

## Troubleshooting

### Build Errors

| Error | Solution |
|-------|---------|
| `target wasm32-wasip2 not found` | `rustup target add wasm32-wasip2` |
| `unresolved import wasi:*` | Ensure `crate-type = ["cdylib"]` in Cargo.toml |
| `kami_guest not found` | Check path dependency in Cargo.toml |

### Runtime Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `execution timed out` | Handler too slow or infinite loop | Increase `max_execution_ms` or optimize |
| `fuel exhausted` | Too many instructions | Increase `max_fuel` |
| `memory grow failure` | Allocating too much | Increase `max_memory_mb` |
| `tool not found` | ID mismatch | Check `kami list` vs your tool.toml ID |
| `WASM file missing` | .wasm not at expected path | Check `wasm` field in tool.toml |

### Debugging

```bash
# Verbose logging
kami -v exec dev.example.my-tool --input '{}'

# Trace logging (all details)
kami -vv exec dev.example.my-tool --input '{}'
```
