# PassChain

*Working in progress*

*正在开发*

Multi-factor authentication plugin for ~~[systemd](https://syste.md/)~~ [systemd](https://systemd.io/) ([`systemd-tty-ask-password-agent`](https://www.freedesktop.org/software/systemd/man/latest/systemd-tty-ask-password-agent.html) replacement)

为 ~~[systemd](https://syste.md/)~~ [systemd](https://systemd.io/) 量身定制的 MFA(多因素验证) 插件，[`systemd-tty-ask-password-agent`](https://www.freedesktop.org/software/systemd/man/latest/systemd-tty-ask-password-agent.html) 替代品

## Why do I need it / 为什么要用这个

- Multiple (depending on how many factors you inputted) [Argon2](https://en.wikipedia.org/wiki/Argon2) computes makes your password stronger, and harder to bruteforce.
- 进行多次(取决于输入多少个因素) Argon2 计算，让密码更难暴力破解
- The password comes from the checksum of multiple factors. Although your Yubikey has been cracked - a passphrase behind is protecting you.
- 使用多个因素的校验和作为密码。即使 Yubikey 被破解 - 还有密码短语作为最后一道防线
- The decryption process doesn't know any information about your factors. You can input infinite factors, then wait forever and let cryptsetup try to decrypt your disk.
- 你设置的因素信息，解密过程都不会知道。你可以输入无限个因素，等到世界末日然后交给 cryptsetup 尝试解锁硬盘

## Features / 功能

- [x] Supports unlimited factors / 支持设置无限多的因素
- [x] Order sensitive / 对顺序敏感
- [x] Only two salt values are stored / 只需保存两个随机盐值

## Roadmap / 路线图

### Core / 核心部分

- [x] Encrypt process / 加密过程
- [ ] Decrypt process / 解密过程 (coming soon)
- [ ] Rewritting code / 重构代码

### Supported Factors / 支持的因素

- [x] [Passphrase](https://en.wikipedia.org/wiki/Passphrase) / [密码](https://zh.wikipedia.org/wiki/密碼片語)
- [x] [CTAP2](https://en.wikipedia.org/wiki/Client_to_Authenticator_Protocol) (HMAC Ext.) / CTAP2 (HMAC 扩展)
- [ ] [TPM](https://en.wikipedia.org/wiki/Trusted_Platform_Module) 2.0 / [可信平台模块](https://zh.wikipedia.org/wiki/%E4%BF%A1%E8%B3%B4%E5%B9%B3%E5%8F%B0%E6%A8%A1%E7%B5%84) 2.0

### Frontend / 前端

- [ ] [Password Agent](https://systemd.io/PASSWORD_AGENTS/) `/dev/console` (coming soon)
- [ ] CLI `stdin` `stdout` `stderr`

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
