FROM rust:1.91.0

ENV WASI_SDK=/opt/wasi-sdk

RUN rustup component add rustfmt &&\
    rustup component add clippy &&\
    rustup target add wasm32-wasip1 &&\
    cargo install cargo-wasi &&\
    apt update &&\
    apt install -y clang &&\
    cargo install wizer --features="env_logger structopt"

RUN cd /tmp &&\
    wget https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-27/wasi-sdk-27.0-$(arch | sed s/aarch64/arm64/)-linux.tar.gz &&\
    mkdir -p $WASI_SDK  &&\
    tar xvf wasi-sdk-27.0-$(arch | sed s/aarch64/arm64/)-linux.tar.gz --strip-components=1 -C $WASI_SDK &&\
    rm wasi-sdk-27.0-$(arch | sed s/aarch64/arm64/)-linux.tar.gz
