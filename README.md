# Passchain

Multi-factor authentication for LUKS

## Build

### Release (musl version)

```sh
docker build -o build .
# or
podman build -o build .
```

Then check the `build` directory.

### Debug (depend on your environment)

```sh
# build
cargo b
# build and run
cargo r
```


## License

MPL-2.0
