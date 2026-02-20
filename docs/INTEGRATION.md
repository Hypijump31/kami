# AI Agent Integration Guide

KAMI exposes tools via the **Model Context Protocol (MCP)** over both
stdio and HTTP transports. Any AI agent supporting MCP can discover and
call KAMI tools.

---

## Claude Desktop

Add KAMI as an MCP server in your Claude Desktop config file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "kami": {
      "command": "kami",
      "args": ["serve", "--transport", "stdio"]
    }
  }
}
```

After restarting Claude Desktop, KAMI tools become available automatically.
Claude will list them when you ask about available tools.

### With a custom registry

```json
{
  "mcpServers": {
    "kami": {
      "command": "kami",
      "args": ["serve", "--transport", "stdio", "--db", "/path/to/registry.db"]
    }
  }
}
```

---

## Cursor

Cursor supports MCP servers via its settings panel.

1. Open **Settings** → **Features** → **MCP Servers**
2. Click **Add Server**
3. Fill in:
   - **Name**: `kami`
   - **Command**: `kami serve --transport stdio`
4. Save and restart Cursor

Cursor will discover all installed KAMI tools and make them available
during code generation.

---

## Continue.dev

[Continue.dev](https://continue.dev) is an open-source AI coding assistant for VS Code and JetBrains that supports MCP servers.

Edit `~/.continue/config.json` (macOS/Linux) or `%USERPROFILE%\.continue\config.json` (Windows):

```json
{
  "mcpServers": [
    {
      "name": "kami",
      "command": "kami",
      "args": ["serve", "--transport", "stdio"]
    }
  ]
}
```

### With a custom registry

```json
{
  "mcpServers": [
    {
      "name": "kami",
      "command": "kami",
      "args": ["serve", "--transport", "stdio", "--db", "/path/to/registry.db"]
    }
  ]
}
```

After saving, reload the Continue extension. KAMI tools will appear in the
`@kami` context menu during chat.

> **Note:** MCP support in Continue.dev requires v0.8.0 or later.
> Check the [Continue.dev docs](https://docs.continue.dev) for the latest config format.

---

## HTTP Transport

For agents that prefer HTTP/SSE, start KAMI with the HTTP transport:

```bash
kami serve --transport http --port 3000
```

### Endpoints

| Method | Path              | Description                |
|--------|-------------------|----------------------------|
| POST   | `/mcp`            | JSON-RPC 2.0 endpoint      |
| GET    | `/health`         | Health check (`200 OK`)    |
| GET    | `/health/ready`   | Readiness probe            |

### Example MCP request

```bash
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }'
```

---

## LangChain / Custom Clients

Use any MCP-compatible client library. Example flow:

```python
import subprocess, json

proc = subprocess.Popen(
    ["kami", "serve", "--transport", "stdio"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    text=True,
)

# Initialize
proc.stdin.write(json.dumps({
    "jsonrpc": "2.0", "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "my-agent", "version": "0.1"}
    }
}) + "\n")
proc.stdin.flush()
response = json.loads(proc.stdout.readline())

# List tools
proc.stdin.write(json.dumps({
    "jsonrpc": "2.0", "id": 2,
    "method": "tools/list"
}) + "\n")
proc.stdin.flush()
tools = json.loads(proc.stdout.readline())
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Tools not showing up | Run `kami list` to verify tools are installed |
| No tools installed yet | `kami search fetch` to find tools, then `kami install owner/repo@tag` |
| Connection refused (HTTP) | Check the port with `--port` flag |
| Permission errors | Ensure `kami` binary is in your `PATH` |
| Slow first execution | First call compiles WASM; subsequent calls use the cache |

### Installing tools for agents

Before starting the MCP server, install the tools you need:

```bash
# Search the registry
kami search "json transform"

# Install from GitHub
kami install kami-tools/json-transform@v1.0.0

# Or from a URL
kami install https://example.com/my-tool.zip

# Verify installation
kami list
```

### Debug logging

```bash
kami serve --transport stdio -vv --log-format json
```
