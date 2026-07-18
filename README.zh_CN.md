# Qubit IO Binary

[![Rust CI](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-io-binary/coverage-badge.json)](https://qubit-ltd.github.io/rs-io-binary/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-io-binary.svg?color=blue)](https://crates.io/crates/qubit-io-binary)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

面向 Rust 的、运行时无关的同步与异步二进制流 I/O。

`qubit-io-binary` 把 `qubit-codec-binary` 的缓冲区 codec 适配到
`qubit-io` 的流抽象：

- 同步扩展基于 `Input<Item = u8>` 与 `Output<Item = u8>`；
- 异步扩展基于 `AsyncInput<Item = u8>` 与 `AsyncOutput<Item = u8>`；
- 两套 API 共享 fixed-width、LEB128、ZigZag 和 UTF-8 codec 语义；
- 同步 typed reader/writer 可携带字节序或解码策略；
- 同步 buffered wrapper 用于摊薄大量小读写的开销。

核心 API 不依赖 Tokio、`futures-io` 或某个 executor。生态异步流通过
`qubit-io` 的可选 adapter 接入。

## 安装

```toml
[dependencies]
qubit-io-binary = "0.2"
```

## 同步示例

标准库 `Read`、`Write` 实现（包括 `Cursor` 和 `Vec<u8>`）可通过 adapter
实现 `qubit-io` 抽象。

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
bytes.write_u32(0x0102_0304, ByteOrder::BigEndian)?;
bytes.write_uleb_u64(300)?;

let mut input = Cursor::new(bytes);
assert_eq!(0x0102_0304, input.read_u32(ByteOrder::BigEndian)?);
assert_eq!(300, input.read_uleb_u64()?);
# Ok::<(), std::io::Error>(())
```

## 异步示例

异步方法统一使用 `_async` 后缀，因此一个类型可以同时暴露同步和异步流
API，而不会产生名字歧义。

```rust
use qubit_io::{
    AsyncInput,
    AsyncOutput,
};
use qubit_io_binary::{
    AsyncBinaryReadExt,
    AsyncBinaryWriteExt,
};

async fn copy_header<I, O>(input: &mut I, output: &mut O) -> std::io::Result<()>
where
    I: AsyncInput<Item = u8> + Unpin,
    O: AsyncOutput<Item = u8> + Unpin,
{
    let version = input.read_u32_le_async().await?;
    output.write_u32_le_async(version).await
}
```

## API 族

| 编码 | 同步 trait | 异步 trait |
| --- | --- | --- |
| Fixed-width 标量 | `BinaryReadExt`、`BinaryWriteExt` | `AsyncBinaryReadExt`、`AsyncBinaryWriteExt` |
| LEB128 | `Leb128ReadExt`、`Leb128WriteExt` | `AsyncLeb128ReadExt`、`AsyncLeb128WriteExt` |
| ZigZag | `ZigZagReadExt`、`ZigZagWriteExt` | `AsyncZigZagReadExt`、`AsyncZigZagWriteExt` |
| 带长度前缀的 UTF-8 | `StringReadExt`、`StringWriteExt` | `AsyncStringReadExt`、`AsyncStringWriteExt` |

同步 typed wrapper 包括 `BinaryReader`、`BinaryWriter`、`Leb128Reader`、
`Leb128Writer`、`ZigZagReader`、`ZigZagWriter` 及其 buffered 变体。它们以
`Input` / `Output` 为泛型边界，并不是基于 `std::io::Read` / `Write` 定义。

Strict LEB128 方法会拒绝非 canonical 编码。字符串读取方法要求传入最大
payload 长度，以限制内存分配。持久化格式应优先使用固定宽度长度字段或
`u64` LEB128 长度，而不是依赖目标平台宽度的 `usize` helper。

## 分层

- `qubit-codec-binary` 负责缓冲区级二进制算法；
- `qubit-io` 负责通用同步流和运行时无关的异步流；
- `qubit-io-binary` 组合两者，不负责文件系统，也不绑定异步运行时。

详细说明见[中文用户指南](doc/user_guide.zh_CN.md)和
[API 文档](https://docs.rs/qubit-io-binary)。

## 开发

```bash
cargo test
./align-ci.sh
RS_CI_SKIP_TOOLCHAIN_UPDATE=1 ./ci-check.sh
```

## 许可证

本项目使用 Apache License 2.0，完整文本见 [LICENSE](LICENSE)。

Copyright (c) 2026 Haixing Hu.
