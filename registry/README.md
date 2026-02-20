# KAMI Tool Registry

Community index of KAMI tools. Like [Homebrew](https://brew.sh) formulae,
this repo contains **metadata only** — the actual tool binaries are hosted
on each author's GitHub Releases.

## Installing a tool

```bash
# Search
kami search "json transform"

# Install from this registry
kami install kami-tools/json-transform@v1.0.0
```

## Publishing your tool

### 1. Prepare your release

Create a `.zip` archive containing `tool.toml` and your `.wasm` file:

```bash
cd my-tool
cargo build --target wasm32-wasip2 --release
mkdir release
cp tool.toml target/wasm32-wasip2/release/my_tool.wasm release/
cd release && zip -r ../plugin.zip . && cd ..
```

### 2. Create a GitHub Release

Upload `plugin.zip` as a release asset on your repository, tagged (e.g. `v1.0.0`).

### 3. Generate your registry entry

```bash
kami publish --source your-org/my-tool@v1.0.0
```

This outputs a JSON entry like:

```json
{
  "id": "dev.myorg.my-tool",
  "name": "my-tool",
  "version": "1.0.0",
  "description": "What it does",
  "source": "your-org/my-tool@v1.0.0",
  "wasm_sha256": "a1b2c3..."
}
```

### 4. Submit a Pull Request

1. Fork this repository
2. Add your entry to `index.json`
3. Open a PR — CI will validate automatically

### Entry format

Each entry in `index.json` has these fields:

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Reverse-DNS tool identifier (from `tool.toml`) |
| `name` | Yes | Human-readable name |
| `version` | Yes | Semantic version |
| `description` | Yes | Short description (shown in `kami search`) |
| `source` | Yes | GitHub shorthand: `owner/repo@tag` |
| `wasm_sha256` | Yes | SHA-256 hex digest of the `.wasm` file |

### Rules

- One entry per tool ID (update the existing entry for new versions)
- The `source` must point to a valid GitHub release with `plugin.zip`
- The `plugin.zip` must contain `tool.toml` + referenced `.wasm` file
- Description should be concise (< 80 characters)

## Schema

See [schema.json](schema.json) for the JSON Schema used by CI validation.

## License

MIT
