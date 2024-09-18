FROM rust:1.80.1-alpine3.20 AS builder

WORKDIR /root/build

RUN apk add --no-cache linux-headers libudev-zero-dev musl-dev upx g++ gcc build-base \
    clang17 clang17-libclang clang17-static llvm17 llvm17-dev llvm17-libs llvm17-static ncurses-dev ncurses-static \
    zlib-dev zlib-static

COPY Cargo.toml Cargo.lock ./

RUN (< Cargo.toml sed 's/"build_dynamic"/"build_static"/g' > Cargo.toml.new) \
    && mv Cargo.toml.new Cargo.toml && mkdir src && ( echo "fn main() {}" > src/main.rs ) \
    && cargo fetch

COPY src ./src

# RUN echo "/usr/lib:/usr/local/lib:/usr/lib/llvm17/lib" > /etc/ld-musl-x86_64.path

# RUN apk add --no-cache strace

# RUN LD_LIBRARY_PATH=/usr/lib/llvm17/lib /usr/lib/gcc/x86_64-alpine-linux-musl/13.2.1/../../../../x86_64-alpine-linux-musl/bin/ld --verbose | grep SEARCH_DIR | tr -s ' ;' \\012

RUN RUSTFLAGS='' \
    RUST_BACKTRACE=1 \
    cargo b -r --target $(uname -m)-unknown-linux-musl \
    && mv target/release/passchain passchain && upx -9 passchain

FROM scratch AS release

COPY --from=builder /root/build/passchain /passchain
