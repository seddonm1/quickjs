This repository demonstrates how to use [quickjs-wasm-rs](https://github.com/Shopify/javy/tree/main/crates/quickjs-wasm-rs) with [wasmtime](https://github.com/bytecodealliance/wasmtime) to easily build a safe and isolated plugin system for Rust.

Code to accompany blog post: https://reorchestrate.com/posts/plugins-for-rust

First `build-wasm.sh` script which will download and build the `quickjs.wasm` module.

# Examples

Run a sequential executor:

```bash
cargo run --example iter --release
```

Run a parallel executor:

```bash
cargo run --example par_iter --release
```

Both accept additional arguments like:

```bash
cargo run --release --example iter -- \
--module ./quickjs.wasm \
--script ./track_points.js \
--data ./track_points.json \
--iterations 1000 \
--inherit-stdout \
--inherit-stderr
```

# Build

```bash
cargo build --package quickjs --release
```

# Test

```bash
cargo test --package quickjs --release
```

# Bench

```bash
cargo bench --package quickjs
```

# Credits

- Peter Malmgren https://github.com/pmalmgren/wasi-data-sharing
- Shopify https://github.com/Shopify/javy
- Bytecode Alliance https://github.com/bytecodealliance/wasmtime
- Bytecode Alliance https://github.com/bytecodealliance/wizer
