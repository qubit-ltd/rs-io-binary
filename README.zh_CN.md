# Qubit IO Binary

[![Rust CI](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-io-binary/coverage-badge.json)](https://qubit-ltd.github.io/rs-io-binary/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-io-binary.svg?color=blue)](https://crates.io/crates/qubit-io-binary)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-io-binary` 让 Rust 应用在既有流上读写结构化二进制协议，而不把协议绑定到
某种文件类型或异步运行时。它把 `qubit-codec-binary` 的缓冲区 codec 适配到
`qubit-io` 流抽象：

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
qubit-io-binary = "0.3"
```

下面的快速开始示例只需要 `qubit-io-binary`。如果要导入字节序或解码策略
类型，请另外添加 `qubit-codec` 或 `qubit-codec-binary`；如果要编写泛型
`Input` 或 `Output` 约束，请另外添加 `qubit-io`。

## 快速开始：同步记录

标准库 `Read`、`Write` 实现（包括 `Cursor` 和 `Vec<u8>`）可通过 adapter
实现 `qubit-io` 抽象。

```rust
use std::io::Cursor;

use qubit_io_binary::{
    BinaryReadExt,
    BinaryWriteExt,
    Leb128ReadExt,
    Leb128WriteExt,
};

let mut bytes = Vec::new();
bytes.write_u32_le(0x0102_0304)?;
bytes.write_uleb_u64(300)?;

let mut input = Cursor::new(bytes);
assert_eq!(0x0102_0304, input.read_u32_le()?);
assert_eq!(300, input.read_uleb_u64_non_strict()?);
# Ok::<(), std::io::Error>(())
```

## 运行时无关的异步 API

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

异步扩展方法只要求 `Unpin`，不额外要求运行时或 `Send`。这些操作不具备取消
安全性：丢弃尚未完成的读取 future 会保留已经消费的字节，丢弃尚未完成的写入
future 会在输出中留下已经写入的前缀。

## 核心能力

| 编码 | 同步 trait | 异步 trait |
| --- | --- | --- |
| Fixed-width 标量 | `BinaryReadExt`、`BinaryWriteExt` | `AsyncBinaryReadExt`、`AsyncBinaryWriteExt` |
| LEB128 | `Leb128ReadExt`、`Leb128WriteExt` | `AsyncLeb128ReadExt`、`AsyncLeb128WriteExt` |
| ZigZag | `ZigZagReadExt`、`ZigZagWriteExt` | `AsyncZigZagReadExt`、`AsyncZigZagWriteExt` |
| 带长度前缀的 UTF-8 | `StringReadExt`、`StringWriteExt` | `AsyncStringReadExt`、`AsyncStringWriteExt` |

同步 typed wrapper 包括 `BinaryReader`、`BinaryWriter`、`Leb128Reader`、
`Leb128Writer`、`ZigZagReader`、`ZigZagWriter` 及其 buffered 变体。它们以
`Input` / `Output` 为泛型边界，并不是基于 `std::io::Read` / `Write` 定义。

变长整数 reader 必须显式指定 `Strict` 或 `NonStrict` 策略类型。严格 reader
使用 `read_u64` 等简短方法名；宽松 reader 使用明确的
`read_u64_non_strict`（字符串方法同样如此）。

缓冲 reader 可通过 `into_parts` 同时取回底层输入与尚未消费的预取字节；缓冲 writer
可通过 `into_parts` 同时取回底层输出与尚未写出的字节。这些方法不执行 I/O：正常
完成时先刷新，刷新失败时保留 writer 以便检查或重试。

Strict LEB128 方法会拒绝非 canonical 编码。字符串读取方法要求传入最大
payload 长度，以限制内存分配。持久化格式应优先使用固定宽度长度字段或
`u64` LEB128 长度，而不是依赖目标平台宽度的 `usize` helper。

## 混合二进制流基准

下表是 2026-08-04 的 Criterion 运行结果。工作负载每轮包含 131,072 个按确定性随机
序列选择的 `u8`、`i32`、`u64` 和多字节 UTF-8 字符串字段。完整命令和环境记录在
[`benches/results/2026-08-04-mixed-binary-pipeline.md`](benches/results/2026-08-04-mixed-binary-pipeline.md)。
它用于比较缓冲策略，不构成可移植的性能承诺。

| 场景 | 时间 | 吞吐量 | 相对裸 extension |
| --- | ---: | ---: | ---: |
| 写入：裸 extension | 65.19 ms | 2.01 M fields/s | 1.0× |
| 写入：extension + `BufWriter` | 4.04 ms | 32.44 M fields/s | 16.1× |
| 写入：`BufferedBinaryWriter` | 3.70 ms | 35.40 M fields/s | 17.6× |
| 读取：裸 extension | 46.80 ms | 2.80 M fields/s | 1.0× |
| 读取：extension + `BufReader` | 3.64 ms | 36.01 M fields/s | 12.9× |
| 读取：`BufferedBinaryReader` | 3.46 ms | 37.90 M fields/s | 13.5× |

本次运行中，buffered wrapper 相比对应的外部缓冲 extension 路径，写入约快 9%，
读取约快 5%。

## 边界与延伸阅读

- `qubit-codec-binary` 负责缓冲区级二进制算法；
- `qubit-io` 负责通用同步流和运行时无关的异步流；
- `qubit-io-binary` 组合两者，不负责文件系统，也不绑定异步运行时。

本 crate 适用于既有流上的二进制值，不负责打开文件，也不拥有异步运行时。需要贯穿
场景教程时，请参阅[中文用户指南](doc/user_guide.zh_CN.md)或
[English user guide](doc/user_guide.md)；全部公开项目请参阅
[API 文档](https://docs.rs/qubit-io-binary)。

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./ci-check.sh

# 检查代码覆盖率
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./align-ci.sh`格式化代码，运行`./ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-io-binary](https://github.com/qubit-ltd/rs-io-binary)
