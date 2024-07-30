FROM rust:1.80.0

ENV QUICKJS_WASM_SYS_WASI_SDK_PATH=/opt/wasi-sdk

RUN rustup component add rustfmt &&\
    rustup component add clippy &&\
    rustup target add wasm32-wasi &&\
    cargo install cargo-wasi &&\
    apt update &&\
    apt install -y clang &&\
    cargo install wizer --features="env_logger structopt"

RUN cd /tmp &&\
    wget https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-23/wasi-sdk-23.0-$(arch | sed s/aarch64/arm64/)-linux.tar.gz &&\
    mkdir -p $QUICKJS_WASM_SYS_WASI_SDK_PATH  &&\
    tar xvf wasi-sdk-23.0-$(arch | sed s/aarch64/arm64/)-linux.tar.gz --strip-components=1 -C $QUICKJS_WASM_SYS_WASI_SDK_PATH &&\
    rm wasi-sdk-23.0-$(arch | sed s/aarch64/arm64/)-linux.tar.gz
