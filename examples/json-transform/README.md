# json-transform

JSON transformation KAMI tool — pick keys, flatten nested objects, or count elements.

## Actions

| Action    | Description                                | Requires `keys` |
|-----------|--------------------------------------------|------------------|
| `pick`    | Extract specific keys from an object       | Yes              |
| `flatten` | Flatten nested objects to dot-notation     | No               |
| `count`   | Count top-level keys                       | No               |

## Build

```bash
cargo build --target wasm32-wasip2 --release
```

## Install & Run

```bash
kami install .
kami exec dev.kami.json-transform '{"action":"pick","data":{"a":1,"b":2,"c":3},"keys":["a","c"]}'
```

Expected output:
```json
{"a":1,"c":3}
```
