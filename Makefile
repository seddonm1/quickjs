iter_example: build_wasm
	cargo run --release --example iter

par_iter_example: build_wasm
	cargo run --release --example par_iter

build: build_wasm
	cargo build --release --package quickjs

test: build_wasm
	cargo test --release --package quickjs

bench: build_wasm
	cargo bench --package quickjs

build_wasm:
	cargo build --release --package quickjs-wasm --target wasm32-wasi
	wizer --allow-wasi ${CARGO_TARGET_DIR}/wasm32-wasi/release/quickjs-wasm.wasm --wasm-bulk-memory true -o quickjs.wasm

lint:
	cargo clippy --all-targets --all-features -- -D warnings &&\
	cargo fmt --all -- --check