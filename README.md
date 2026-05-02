# fledge-plugin-test-store

WASM plugin that tests the `store` capability for [fledge](https://github.com/CorvidLabs/fledge).

Verifies that WASM plugins can persist and retrieve key-value data via `fledge::store_set` and `fledge::store_get` when granted `store = true`, and that other capabilities remain blocked.

## Install & Run

```bash
fledge plugins install CorvidLabs/fledge-plugin-test-store
fledge plugins run test-store
```

## Requirements

- [fledge](https://github.com/CorvidLabs/fledge) with WASM runtime support
- `wasm32-wasip1` Rust target: `rustup target add wasm32-wasip1`
