FROM rust:1.80-alpine3.20 as builder

WORKDIR /root/build
RUN apk add --no-cache linux-headers libudev-zero-dev musl-dev upx
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo b -r && mv target/release/passchain passchain && upx -9 passchain

FROM scratch as release

COPY --from=builder /root/build/passchain /passchain