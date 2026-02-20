# http-fetch

KAMI tool example demonstrating network access capabilities.

## Description

Fetches content from a URL via HTTP GET. Validates the URL scheme (http/https),
enforces a configurable maximum response size, and requires explicit network
permissions via `tool.toml`.

> **Note:** This example validates URLs and formats responses. Actual HTTP
> requests are performed by the WASI HTTP outgoing handler at runtime.

## Security

This tool requires network access. The `tool.toml` declares:

```toml
[security]
net_allow_list = ["*"]    # Allow all hosts (restrict in production!)
fs_access = "none"        # No filesystem access
max_memory_mb = 32
max_execution_ms = 10000
```

In production, replace `"*"` with specific hostnames:

```toml
net_allow_list = ["api.github.com", "*.example.com"]
```

## Usage

```bash
# Build
cargo build --target wasm32-wasip2 --release

# Install
kami install .

# Execute
kami exec dev.kami.http-fetch '{"url":"https://example.com"}'

# With max_bytes
kami exec dev.kami.http-fetch '{"url":"https://api.github.com/zen","max_bytes":1024}'
```

## Arguments

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `url` | string | yes | URL to fetch (http:// or https://) |
| `max_bytes` | number | no | Max response size (default: 65536) |
