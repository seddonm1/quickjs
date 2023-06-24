This repository demonstrates how to use [quickjs-wasm-rs](https://github.com/bytecodealliance/javy/tree/main/crates/quickjs-wasm-rs) with [wasmtime](https://github.com/bytecodealliance/wasmtime) to easily build a safe and isolated plugin system for Rust.

Code to accompany blog post: https://reorchestrate.com/posts/plugins-for-rust

# How to Use

The two examples `iter` and `par_iter` demonstrate how to use this library. They both support the following arguments:

- `module`: optional path to the wasm module. otherwise `quickjs.wasm` produced by the `quickjs-wasm` crate is used.
- `script`: the javascript code to evaluate by quickjs.
- `data`: an optional dataset to inject into the instance. availabie in quickjs as the global `data`.
- `iterations`: how many times to execute the javascript.
- `inherit-stdout`: allow the container to use `console.log`. requires building `quickjs-wasm` with `console` feature (default).
- `inherit-stderr`: allow the container to use `console.error`. requires building `quickjs-wasm` with `console` feature (default).
- `memory-limit-bytes`: optional runtime memory limit in bytes to restrict unconstrained memory growth. useful if running untrusted code.
- `time-limit-micros`: optional runtime time limit in microseconds. useful if running untrusted code that may be long running programs/infinite loops or to provide quality-of-service.
- `time-limit-evaluation-interval-micros`: optional interval in microseconds for evaluating if `time_limit` has been exceeded. default `100µs`.

```bash
cargo run --release --example iter -- \
--module ./quickjs.wasm \
--script ./track_points.js \
--data ./track_points.json \
--iterations 1000 \
--inherit-stdout \
--inherit-stderr \
--memory-limit-bytes 4194304 \
--time-limit-micros 1000000 \
--time-limit-evaluation-interval-micros 1000
```

## time-limit
`time-limit-micros` utilises a configurable periodic (default `100µs`) interrupt to test if the program has exceeded its `time-limit` that adds some execution overhead. Run `make bench` or either [example](examples) with `time-limit-micros` to see what the impact is on your code. Due to this cost it is only probably worth using if evaluating untrusted code or if `time-limit-evaluation-interval-micros` is tuned for your use case (i.e. a script with an expected `time-limit` of 60 seconds probably does not need to be evaulated more than every `100ms`).

```
try_execute             time:   [2.7044 ms 2.7670 ms 2.8326 ms]
```

```
try_execute_with_time_limit_100us
                        time:   [3.2581 ms 3.2964 ms 3.3367 ms]
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
