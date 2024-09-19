FROM rust:1.81-bookworm AS builder

# glibc version 2.36

WORKDIR /root/build

RUN apt-get update && \
    apt-get install -y linux-headers-amd64 libudev-dev clang llvm-dev libclang-dev && \
    apt-get clean

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && ( echo "fn main() {}" > src/main.rs ) \
    && cargo fetch

COPY src ./src

RUN RUSTFLAGS='' \
    RUST_BACKTRACE=1 \
    cargo b -r \
    && mv target/release/passchain passchain

FROM scratch AS release

COPY --from=builder /root/build/passchain /passchain
