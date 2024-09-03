FROM rust:1.80.1-alpine3.20 AS builder

WORKDIR /root/build
RUN apk add --no-cache linux-headers libudev-zero-dev musl-dev upx

COPY Cargo.toml Cargo.lock ./
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
RUN cargo fetch

COPY src ./src

RUN cargo b -r && mv target/release/passchain passchain && upx -9 passchain

FROM scratch AS release

COPY --from=builder /root/build/passchain /passchain
