<p align="center"><img src="../site/assets/logo.png" alt="KAMI" height="48"></p>

# Tool Author Guide — Build Your First KAMI Tool in 10 Minutes

> **Audience:** Developers who want to create tools that AI agents can call securely through KAMI.  
> **Prerequisites:** Rust installed, `wasm32-wasip2` target added.

---

## TL;DR

```bash
# 1. Setup (once)
rustup target add wasm32-wasip2
cargo build --release -p kami-cli

# 2. Scaffold
kami init my-tool

# 3. Write your handler (src/lib.rs)

# 4. Build → Install → Run
cd my-tool
cargo build --target wasm32-wasip2 --release
cd .. && kami install ./my-tool
kami exec dev.example.my-tool '{"name":"World"}'
```

---

## Step 1: Setup Your Environment

### Install Rust & WASM Target

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add the WASM Component Model target
rustup target add wasm32-wasip2
```

### Build KAMI CLI

```bash
git clone https://github.com/Hypijump31/kami.git
cd kami
cargo build --release -p kami-cli
# Add to PATH or use: ./target/release/kami
```

---

## Step 2: Scaffold a New Tool

```bash
kami init my-tool
cd my-tool
```

This creates:

```
my-tool/
├── Cargo.toml      # Rust project (cdylib crate type)
├── tool.toml       # KAMI manifest (ID, security, MCP metadata)
└── src/
    └── lib.rs      # Your tool handler
```

---

## Step 3: Understand the Architecture

A KAMI tool is a **WebAssembly Component** that implements two functions:

| Export | Signature | Purpose |
|--------|-----------|---------|
| `run` | `(input: string) → result<string, string>` | Execute the tool with JSON arguments |
| `describe` | `() → string` | Return tool metadata as JSON |

The `kami_tool!` macro generates both exports from a single handler function.

```
 AI Agent                    KAMI                      Your Tool (.wasm)
    │                         │                              │
    │── tools/call ──────────►│                              │
    │   {name, arguments}     │── run(json_input) ──────────►│
    │                         │                              │── your_handler()
    │                         │◄── Ok(json_output) ──────────│
    │◄── result ──────────────│                              │
```

---

## Step 4: Write Your Handler

Open `src/lib.rs`. The simplest tool looks like this:

### a) Echo Tool (pass-through)

```rust
use kami_guest::kami_tool;

kami_tool! {
    name: "dev.myorg.echo",
    version: "1.0.0",
    description: "Echoes back the input",
    handler: handle,
}

fn handle(input: &str) -> Result<String, String> {
    // Input is a JSON string — return it unchanged
    Ok(input.to_string())
}
```

### b) Typed Tool (with deserialization)

```rust
use kami_guest::{kami_tool, parse_input, text_result};
use serde::Deserialize;

kami_tool! {
    name: "dev.myorg.greet",
    version: "1.0.0",
    description: "Returns a greeting",
    handler: handle,
}

#[derive(Deserialize)]
struct Input {
    name: String,
    #[serde(default = "default_lang")]
    lang: String,
}

fn default_lang() -> String { "en".to_string() }

fn handle(input: &str) -> Result<String, String> {
    let args: Input = parse_input(input)?;
    let greeting = match args.lang.as_str() {
        "fr" => format!("Bonjour, {} !", args.name),
        "es" => format!("¡Hola, {}!", args.name),
        _    => format!("Hello, {}!", args.name),
    };
    text_result(&greeting)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greets_in_english() {
        let result = handle(r#"{"name":"Alice"}"#).unwrap();
        assert!(result.contains("Hello, Alice!"));
    }

    #[test]
    fn greets_in_french() {
        let result = handle(r#"{"name":"Bob","lang":"fr"}"#).unwrap();
        assert!(result.contains("Bonjour, Bob"));
    }

    #[test]
    fn missing_name_errors() {
        assert!(handle(r#"{}"#).is_err());
    }
}
```

### c) JSON Transform Tool (structured output)

```rust
use kami_guest::{kami_tool, parse_input, to_output};
use serde::{Deserialize, Serialize};

kami_tool! {
    name: "dev.myorg.uppercase",
    version: "1.0.0",
    description: "Uppercases a text field",
    handler: handle,
}

#[derive(Deserialize)]
struct Input { text: String }

#[derive(Serialize)]
struct Output { result: String, length: usize }

fn handle(input: &str) -> Result<String, String> {
    let args: Input = parse_input(input)?;
    let upper = args.text.to_uppercase();
    to_output(&Output {
        length: upper.len(),
        result: upper,
    })
}
```

---

## Step 5: Configure `tool.toml`

The manifest tells KAMI how to register and sandbox your tool:

```toml
[tool]
id = "dev.myorg.greet"            # Unique reverse-DNS identifier
name = "greet"                     # Human-readable name
version = "1.0.0"                  # Semantic version
wasm = "target/wasm32-wasip2/release/greet.wasm"  # Path to compiled WASM

[mcp]
description = "Returns a greeting for the given name"

# Declare input arguments (exposed to AI agents via MCP)
[[mcp.arguments]]
name = "name"
type = "string"
description = "Name to greet"
required = true

[[mcp.arguments]]
name = "lang"
type = "string"
description = "Language code (en, fr, es)"
required = false

[security]
# Filesystem access: "none" | "read-only" | "sandbox"
fs_access = "none"

# Network: list of allowed hosts (empty = no network)
# net_allow_list = ["api.example.com", "*.github.com"]

# Environment variables the tool can read
# env_allow_list = ["API_KEY"]

# Resource limits
max_memory_mb = 16          # Max RAM in MB (default: 64)
max_execution_ms = 2000     # Timeout in milliseconds (default: 5000)
# max_fuel = 1000000        # Instruction budget (default: 1000000)
```

### Security Principle: Deny-All by Default

| Permission | Default | How to Enable |
|-----------|---------|---------------|
| Network | **Blocked** | `net_allow_list = ["host.com"]` |
| Filesystem | **Blocked** | `fs_access = "read-only"` or `"sandbox"` |
| Env vars | **Blocked** | `env_allow_list = ["VAR_NAME"]` |
| Memory | **64 MB** | `max_memory_mb = 128` |
| CPU time | **5 sec** | `max_execution_ms = 10000` |

---

## Step 6: Build & Test

### Run Native Tests

```bash
# Tests run natively (no WASM) — fast iteration
cargo test
```

### Build to WASM

```bash
cargo build --target wasm32-wasip2 --release
```

The output is at `target/wasm32-wasip2/release/<name>.wasm`.

> **Tip:** Use `kami dev watch .` during development to auto-rebuild on file changes.

---

## Step 7: Install & Execute

```bash
# Go back to the KAMI workspace root (or wherever kami binary is)
cd ..

# Register the tool in KAMI's SQLite registry (local install)
kami install ./my-tool

# Or install from a remote source
# kami install https://example.com/my-tool-v1.0.0.zip
# kami install your-org/my-tool@v1.0.0

# List installed tools
kami list

# Execute the tool
kami exec dev.myorg.greet '{"name":"Alice","lang":"fr"}'
# Output: {"text":"Bonjour, Alice !"}

# Verify WASM integrity
kami verify dev.myorg.greet
```

---

## Step 8: Expose via MCP (for AI Agents)

Start KAMI as an MCP server so AI agents (Claude, Cursor, etc.) can call your tool:

### stdio mode (single client — Claude Desktop, Cursor)

```bash
kami serve
```

Configure your AI agent to launch KAMI. Example for **Claude Desktop** (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "kami": {
      "command": "/path/to/kami",
      "args": ["serve"]
    }
  }
}
```

### HTTP mode (multi-client)

```bash
kami serve --transport http --port 3000 --token my-secret
```

Then call your tool via HTTP:

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Authorization: Bearer my-secret" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "dev.myorg.greet",
      "arguments": {"name": "Alice"}
    }
  }'
```

---

## Step 9: Distribute Your Tool

Once your tool works locally, share it with other KAMI users.

### Option A: GitHub Releases (recommended)

1. Create a `.zip` archive containing `tool.toml` + your `.wasm` file:
   ```bash
   mkdir release && cp tool.toml target/wasm32-wasip2/release/greet.wasm release/
   cd release && zip -r ../greet-v1.0.0.zip . && cd ..
   ```

2. Upload to a GitHub release tagged `v1.0.0`.

3. Users install with one command:
   ```bash
   kami install your-org/greet@v1.0.0
   ```

### Option B: Direct URL

Host the `.zip` anywhere (S3, CDN, web server):

```bash
kami install https://your-cdn.com/tools/greet-v1.0.0.zip
```

### Option C: Register in the Community Index

Use `kami publish` to generate the registry entry:

```bash
kami publish --source your-org/greet@v1.0.0
```

This outputs:
```json
{
  "id": "dev.myorg.greet",
  "name": "greet",
  "version": "1.0.0",
  "description": "Returns a greeting",
  "source": "your-org/greet@v1.0.0",
  "wasm_sha256": "a1b2c3..."
}
```

Then submit a PR to [Hypijump31/kami-registry](https://github.com/Hypijump31/kami-registry) to add it to `index.json`. CI validates automatically.

### ZIP Archive Format

The archive must contain at its root (or inside one top-level directory):

```
plugin.zip
├── tool.toml          # Required
└── greet.wasm         # Referenced by tool.toml's wasm field
```

KAMI automatically flattens a single top-level directory (e.g. `greet-v1.0.0/tool.toml` → `tool.toml`).

---

## SDK Reference

### `kami_tool!` Macro

```rust
kami_tool! {
    name: "reverse.dns.id",       // Required: unique tool ID
    version: "1.0.0",             // Required: semver
    description: "What it does",  // Required: shown to AI agents
    handler: your_fn,             // Required: fn(&str) -> Result<String, String>
}
```

### ABI Helper Functions

| Function | Signature | Purpose |
|----------|-----------|---------|
| `parse_input<T>(input)` | `&str → Result<T, String>` | Deserialize JSON input to a typed struct |
| `to_output<T>(value)` | `&T → Result<String, String>` | Serialize a struct to JSON output |
| `text_result(text)` | `&str → Result<String, String>` | Wrap a string in `{"text": "..."}` |
| `error_result(msg)` | `&str → String` | Wrap an error in `{"error": "..."}` |

### Handler Contract

Your handler function must match:

```rust
fn handler(input: &str) -> Result<String, String>
```

- **`input`**: JSON string from the AI agent (contents match your `[[mcp.arguments]]`)
- **`Ok(json)`**: Success — return a JSON string
- **`Err(message)`**: Failure — return a human-readable error message

---

## Common Patterns

### Error Handling

```rust
fn handle(input: &str) -> Result<String, String> {
    let args: MyInput = parse_input(input)?;  // ? auto-converts to Err(String)

    let result = do_something(&args)
        .map_err(|e| format!("processing failed: {e}"))?;

    to_output(&result)
}
```

### Working with JSON Directly

```rust
fn handle(input: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(input)
        .map_err(|e| format!("invalid JSON: {e}"))?;

    let name = value["name"].as_str()
        .ok_or("missing 'name' field")?;

    Ok(serde_json::json!({ "greeting": format!("Hi {name}") }).to_string())
}
```

### Reading Files (requires `fs_access`)

```rust
// tool.toml: fs_access = "read-only"
fn handle(input: &str) -> Result<String, String> {
    // Paths are relative to the sandbox directory
    let data = std::fs::read_to_string("data.txt")
        .map_err(|e| format!("read failed: {e}"))?;
    Ok(data)
}
```

---

## Signing Your Tool (Ed25519)

Signing your WASM plugin provides cryptographic proof of authorship and integrity.
KAMI verifies signatures automatically at execution time when present.

### 1. Generate a Keypair (Once)

```bash
kami keygen
# Keys saved to:
#   ~/.kami/keys/kami_signing_key       (SECRET — keep safe!)
#   ~/.kami/keys/kami_signing_key.pub   (public — share freely)
```

### 2. Sign Your WASM Binary

```bash
kami sign ./my-tool/
# Output:
#   Signature: a1b2c3d4...  (128 hex chars)
#   Public key: e5f6a7b8... (64 hex chars)
#   Add to your tool.toml:
#     signature = "a1b2c3d4..."
#     signer_public_key = "e5f6a7b8..."
```

### 3. Add to tool.toml

```toml
[tool]
id = "dev.myorg.greet"
name = "greet"
version = "1.0.0"
wasm = "target/wasm32-wasip2/release/greet.wasm"
signature = "a1b2c3d4..."
signer_public_key = "e5f6a7b8..."
```

### 4. Verify

```bash
# Verify SHA-256 + Ed25519 signature
kami verify dev.myorg.greet

# Or with an explicit public key
kami verify dev.myorg.greet --public-key ~/.kami/keys/kami_signing_key.pub
```

> **Note:** Signatures are optional. Unsigned tools still work but won't have
> cryptographic authenticity verification at execution time.

---

## Updating & Versioning

```bash
# After changing your tool, rebuild and update
cargo build --target wasm32-wasip2 --release
kami update dev.myorg.greet

# Pin a specific version (prevents kami update --all from changing it)
kami pin dev.myorg.greet 1.0.0

# Unpin
kami pin --unpin dev.myorg.greet
```

---

## Troubleshooting

| Problem | Cause | Fix |
|---------|-------|-----|
| `unknown import: wasi:http` | Tool uses HTTP but security blocks it | Add `net_allow_list` in tool.toml |
| `export not found: run` | Missing `kami_tool!` macro | Add the macro with your handler |
| `network access denied for host: X` | Host not in allow-list | Add host to `net_allow_list` |
| `filesystem access denied` | `fs_access = "none"` | Change to `"read-only"` or `"sandbox"` |
| `resource limit exceeded: fuel` | Tool ran too many instructions | Increase `max_fuel` in tool.toml |
| `resource limit exceeded: memory` | Tool uses too much RAM | Increase `max_memory_mb` |
| WASM build fails | Missing target | `rustup target add wasm32-wasip2` |
| `integrity mismatch` | WASM changed since install | Run `kami update <tool-id>` |
| `signature verification failed` | WASM modified after signing | Re-sign with `kami sign` |
| `invalid public key` | Wrong key format | Use 64-char hex string or `.pub` file path |

---

## Next Steps

- Browse the [examples/](../examples/) directory for complete tools
- Read [INTEGRATION.md](INTEGRATION.md) for connecting to Claude, Cursor, and LangChain
- Read [SECURITY.md](SECURITY.md) for the full security model
- Check [ARCHITECTURE.md](ARCHITECTURE.md) for how KAMI works internally
