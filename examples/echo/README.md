# echo

Echo KAMI tool — returns the JSON input unchanged.

## Build

```bash
cargo build --target wasm32-wasip2 --release
```

## Install & Run

```bash
kami install .
kami exec dev.kami.echo '{"key":"value"}'
```

Expected output:
```json
{"key":"value"}
```
