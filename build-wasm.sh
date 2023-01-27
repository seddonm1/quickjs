export QUICKJS_WASM_SYS_WASI_SDK_PATH=/opt/wasi-sdk
# Check that something is present where the user says the wasi-sdk is located
if [ ! -d "$QUICKJS_WASM_SYS_WASI_SDK_PATH" ]; then
  wget https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-19/wasi-sdk-19.0-linux.tar.gz
  mkdir -p $QUICKJS_WASM_SYS_WASI_SDK_PATH
  tar xvf wasi-sdk-19.0-linux.tar.gz --strip-components=1 -C $QUICKJS_WASM_SYS_WASI_SDK_PATH
  rm wasi-sdk-19.0-linux.tar.gz
fi
# Build the base package
cargo build --release --package quickjs-wasm --target wasm32-wasi
# If wizer is not installed then install it
if [ -z $(which wizer) ]
then
  cargo install wizer --all-features
fi
# apply wizer optimisation
wizer --allow-wasi target/wasm32-wasi/release/quickjs-wasm.wasm --wasm-bulk-memory true -o quickjs.wasm
