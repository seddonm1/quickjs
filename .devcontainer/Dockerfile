FROM rust:1.68.2

RUN rustup component add rustfmt &&\
    rustup component add clippy &&\
    rustup target add wasm32-wasi &&\
    cargo install cargo-wasi &&\
    apt update &&\
    apt install -y clang