FROM rust:1.80.1-alpine3.20 AS builder

WORKDIR /root/build
RUN apk add --no-cache linux-headers libudev-zero-dev musl-dev upx clang17 clang17-libclang clang17-static llvm

COPY Cargo.toml Cargo.lock ./

RUN ls /lib/*

RUN mkdir src && ( echo "fn main() {}" > src/main.rs ) && cargo fetch

COPY src ./src

RUN RUSTFLAGS='' \
    cargo b -r && mv target/release/passchain passchain && upx -9 passchain

FROM scratch AS release

COPY --from=builder /root/build/passchain /passchain
