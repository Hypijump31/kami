# hello-world

Minimal KAMI tool example — returns a greeting for a given name.

## Build

```bash
cargo build --target wasm32-wasip2 --release
```

## Install & Run

```bash
kami install .
kami exec dev.kami.hello-world '{"name":"Alice"}'
```

Expected output:
```json
{"text":"Hello, Alice! Welcome to KAMI."}
```
