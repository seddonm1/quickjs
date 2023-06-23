This repository demonstrates how to use [quickjs-wasm-rs](https://github.com/bytecodealliance/javy/tree/main/crates/quickjs-wasm-rs) with [wasmtime](https://github.com/bytecodealliance/wasmtime) to easily build a safe and isolated plugin system for Rust.

Code to accompany blog post: https://reorchestrate.com/posts/plugins-for-rust

# Examples

Run a sequential executor:

```bash
make iter_example
```

Run a parallel executor:

```bash
make par_iter_example
```

Both accept additional arguments like:

```bash
make build_wasm &&\
cargo run --release --example iter -- \
--module ./quickjs.wasm \
--script ./track_points.js \
--data ./track_points.json \
--iterations 1000 \
--inherit-stdout \
--inherit-stderr \
--memory-limit-bytes 4194304 \
--time-limit-nanos 20000000 \
--time-limit-evaluation-interval-nanos 1000000
```

# Build

To build the `.wasm` module:

```bash
make build_wasm
```

To build the project:

```bash
make build
```

# Test

```bash
make test
```

# Bench

```bash
make bench
```

# Credits

- Peter Malmgren https://github.com/pmalmgren/wasi-data-sharing
- Shopify https://github.com/Shopify/javy now https://github.com/bytecodealliance/javy
- Bytecode Alliance https://github.com/bytecodealliance/wasmtime
- Bytecode Alliance https://github.com/bytecodealliance/wizer
