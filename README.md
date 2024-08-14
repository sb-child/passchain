# PassChain

*Working in progress*

Multi-factor authentication for LUKS

## Build

```sh
git clone https://github.com/sb-child/passchain
```

### Release (`musl` target)

Install [docker](https://www.docker.com/) or [podman](https://podman.io/) first.

```sh
docker build -o build .
# or
podman build -o build .
```

Then check the `build` directory.

### Development (depend on your OS)

Install [rust](https://www.rust-lang.org/) first.

And install missing libraries if any.

```sh
# build
cargo b
# build and run
cargo r
```

## License

MPL-2.0
