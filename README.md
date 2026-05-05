# fledge-plugin-test-store

WASM test plugin for the [fledge](https://github.com/CorvidLabs/fledge) `store` capability.

## What it tests

Verifies that WASM plugins can persist and retrieve key-value data through the `fledge::store_set` and `fledge::store_get` host imports when granted `store = true` in `plugin.toml`. Runs the following test cases:

- **Basic set/get roundtrip** -- store a value, read it back
- **Overwrite** -- writing the same key replaces the previous value
- **Nonexistent key** -- reading a missing key returns null
- **Multiple keys** -- independent keys are stored without interference
- **Empty value** -- empty strings are stored and retrieved correctly
- **Numeric string** -- string values containing digits are preserved as-is
- **Negative tests** -- filesystem, network, and process spawn are all blocked (only `store` is granted)

## Capability exercised

```toml
[capabilities]
exec = false
store = true
metadata = false
filesystem = "none"
network = false
```

## Install and run

```bash
fledge plugins install corvid-agent/fledge-plugin-test-store
fledge plugins run test-store
```

## Build from source

```bash
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1 --release
```

The compiled WASM binary is written to `target/wasm32-wasip1/release/test-store.wasm`.
