# Qubit IO Binary

[![Rust CI](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-io-binary/coverage-badge.json)](https://qubit-ltd.github.io/rs-io-binary/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-io-binary.svg?color=blue)](https://crates.io/crates/qubit-io-binary)
[![License](https://img.shields.io/crates/l/qubit-io-binary.svg)](LICENSE)

面向 Rust 的二进制 stream I/O adapter。

`qubit-io-binary` 基于 `qubit-io` 和 `qubit-codec-binary`，提供：

- fixed-width binary scalar 的 extension trait；
- unsigned / signed LEB128 整数的 extension trait；
- ZigZag 有符号整数的 extension trait；
- length-prefixed UTF-8 字符串 helper；
- `BinaryReader`、`Leb128Reader`、`ZigZagReader` 以及 buffered 变体等 typed
  reader / writer wrapper。

`Streams`、`ReadExt` 等通用 `std::io` helper，以及 `CountingReader` 等通用 wrapper
位于 `qubit-io`。

详细用法请参见[中文用户指南](doc/user_guide.zh_CN.md)。API 参考文档可在
[docs.rs](https://docs.rs/qubit-io-binary) 查看。

## 安装

```toml
[dependencies]
qubit-io-binary = "0.1"
```

## 快速示例

```rust
use std::io::Cursor;

use qubit_io_binary::{
    BinaryReadExt,
    BinaryWriteExt,
    ByteOrder,
    Leb128ReadExt,
    Leb128WriteExt,
};

let mut bytes = Vec::new();
bytes.write_u16(0x1234, ByteOrder::BigEndian)?;
bytes.write_uleb_u32(300)?;

let mut input = Cursor::new(bytes);
assert_eq!(0x1234, input.read_u16(ByteOrder::BigEndian)?);
assert_eq!(300, input.read_uleb_u32()?);
# Ok::<(), std::io::Error>(())
```

## 分层

- `qubit-codec-binary` 提供缓冲区级 codec。
- `qubit-io-binary` 将这些 codec 适配到 `Read` 和 `Write`。
- `qubit-io` 保持为通用 I/O 工具层。

仓库地址：[https://github.com/qubit-ltd/rs-io-binary](https://github.com/qubit-ltd/rs-io-binary)
