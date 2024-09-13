# PassChain

*Working in progress*

*正在开发*

Multi-factor authentication plugin for systemd (`systemd-tty-ask-password-agent` replacement)

为 systemd 量身定制的 MFA(多因素验证) 插件，替代 `systemd-tty-ask-password-agent`

## Roadmap / 路线图

### Core / 核心部分

- [x] Encrypt process / 加密过程
- [x] Decrypt process / 解密过程

- [ ] Rewritting code / 重构代码

### Supported Factors / 支持的因素

- [x] [Passphrase](https://en.wikipedia.org/wiki/Passphrase) / [密码](https://zh.wikipedia.org/wiki/密碼片語)
- [x] [CTAP2](https://en.wikipedia.org/wiki/Client_to_Authenticator_Protocol) (HMAC Ext.) / CTAP2 (HMAC 扩展)
- [ ] [TPM](https://en.wikipedia.org/wiki/Trusted_Platform_Module) 2.0 / [可信平台模块](https://zh.wikipedia.org/wiki/%E4%BF%A1%E8%B3%B4%E5%B9%B3%E5%8F%B0%E6%A8%A1%E7%B5%84) 2.0

### Frontend / 前端

- [ ] [Password Agent](https://systemd.io/PASSWORD_AGENTS/) `/dev/console` (coming soon)
- [ ] `/dev/tty`

## Build / 编译

```sh
git clone https://github.com/sb-child/passchain
```

### Release (`musl` target) / 发布版

Install [docker](https://www.docker.com/) or [podman](https://podman.io/) first.

```sh
docker build -o build .
# or
podman build -o build .
```

Then check the `build` directory.

### Development (depend on your OS) / 测试版

Install [rust](https://www.rust-lang.org/) first.

And install missing libraries if any.

```sh
# build
cargo b
# build and run
cargo r
```

## License / 许可证

MPL-2.0, see [LICENSE](./LICENSE)
