# Qubit IO Binary 用户指南

[English](user_guide.md) · [README](../README.zh_CN.md) ·
[API 文档](https://docs.rs/qubit-io-binary)

本指南面向需要在字节流上读取或写入结构化二进制值的 Rust 应用，适用于
`qubit-io-binary` 0.3。该 crate 将 `qubit-codec-binary` 的缓冲区 codec 接入
`qubit-io` 的流抽象；它不负责打开文件、不选择传输层，也不绑定异步运行时。

## 概念模型

`qubit-codec-binary` 负责二进制表示，`qubit-io` 提供通用的同步
`Input` / `Output` 与异步 `AsyncInput` / `AsyncOutput`，本 crate 则把 codec
应用到这些流上。

| 线上的表示 | 同步 API | 异步 API |
| --- | --- | --- |
| 固定宽度标量 | `BinaryReadExt`、`BinaryWriteExt` | `AsyncBinaryReadExt`、`AsyncBinaryWriteExt` |
| LEB128 | `Leb128ReadExt`、`Leb128WriteExt` | `AsyncLeb128ReadExt`、`AsyncLeb128WriteExt` |
| ZigZag 整数 | `ZigZagReadExt`、`ZigZagWriteExt` | `AsyncZigZagReadExt`、`AsyncZigZagWriteExt` |
| 带长度前缀的 UTF-8 | `StringReadExt`、`StringWriteExt` | `AsyncStringReadExt`、`AsyncStringWriteExt` |

仅执行一次流操作时使用扩展 trait；当一个流长期使用同一种字节序或解码策略时，使用
typed reader/writer。它们的 buffered 变体会保留未读或未写字节，适合大量小操作。

## 贯穿场景：写入并读取一条小记录

设想一种文件格式：先写一个大端 format marker，再写一个紧凑的无符号 record id，
最后写一个长度使用 `u64` LEB128 编码的 UTF-8 label。成功标准是 round trip 后得到
完全相同的值。

## 安装

```toml
[dependencies]
qubit-io-binary = "0.3"
qubit-codec = "0.11"
qubit-io = "0.14"
```

示例使用 `Vec<u8>` 和 `Cursor`；标准库字节源与字节宿可通过 `qubit-io` adapter
参与该抽象。

## 核心流程

```rust
use std::io::Cursor;

use qubit_codec::ByteOrder;
use qubit_io_binary::{
    BinaryReadExt,
    BinaryWriteExt,
    Leb128ReadExt,
    Leb128WriteExt,
    StringReadExt,
    StringWriteExt,
};

let mut bytes = Vec::new();
bytes.write_u32(0x5142_4954, ByteOrder::BigEndian)?;
bytes.write_uleb_u64(300)?;
bytes.write_utf8_string_uleb_u64("inventory")?;

let mut input = Cursor::new(bytes);
assert_eq!(0x5142_4954, input.read_u32(ByteOrder::BigEndian)?);
assert_eq!(300, input.read_uleb_u64_non_strict()?);
assert_eq!("inventory", input.read_utf8_string_uleb_u64(64)?);
# Ok::<(), std::io::Error>(())
```

宽松 extension 方法刻意使用 `_non_strict` 后缀。需要规范 LEB128 编码时应使用对应的
`_strict` 方法；typed reader 同样必须显式指定 `Strict` 或 `NonStrict` 策略类型。

`max_len` 是输入校验边界：它限制返回 `String` 前可接受的 payload 长度。应根据文件
格式或协议限制设置，而不要根据当前预期数据设置。

字节序固定时，使用 `_be` 和 `_le` 方法，例如 `write_i64_le(-42)`。`uleb` 方法处理
无符号 LEB128，`sleb` 方法处理有符号 LEB128；ZigZag 将有符号值映射为无符号 LEB128
payload，使零附近的正负值都保持紧凑。

## Typed 与 Buffered 流

格式长期使用同一种字节序时，typed wrapper 会把该选择保存到类型中：

```rust
use std::io::Cursor;

use qubit_codec::LittleEndian;
use qubit_io_binary::{BinaryReader, BinaryWriter};

let mut writer = BinaryWriter::<_, LittleEndian>::new(Vec::new());
writer.write_u16(0x1234)?;
let bytes = writer.into_inner();

let mut reader = BinaryReader::<_, LittleEndian>::new(Cursor::new(bytes));
assert_eq!(0x1234, reader.read_u16()?);
# Ok::<(), std::io::Error>(())
```

`Buffered*Reader::into_parts()` 返回被包装的输入和预取后尚未消费的字节；
`Buffered*Writer::into_parts()` 返回被包装的输出和尚未写出的字节。两者都不执行 I/O：
正常完成时先 flush writer；flush 失败时应保留它，以便重试。

## 异步流程

异步 API 在方法名后加 `_async`。其 future 是 `Send`，因此流除实现相应异步 trait 外，
还必须实现 `Send + Unpin`。

```rust
use qubit_io::{AsyncInput, AsyncOutput};
use qubit_io_binary::{AsyncBinaryReadExt, AsyncBinaryWriteExt};

async fn relay<I, O>(input: &mut I, output: &mut O) -> std::io::Result<()>
where
    I: AsyncInput<Item = u8> + Send + Unpin,
    O: AsyncOutput<Item = u8> + Send + Unpin,
{
    let marker = input.read_u32_be_async().await?;
    output.write_u32_be_async(marker).await
}
```

这些高层操作不具备取消安全性。取消 pending read 可能已经消费一个值的部分字节；取消
pending write 可能已经向输出写入前缀。只有在传输或 framing 保证安全时才能整体重试。

## 错误与诊断

- 在标量、长度前缀或 payload 中途结束的流会返回 `UnexpectedEof`。
- 畸形 codec 数据、strict LEB128 拒绝的非 canonical 编码、无效 UTF-8 和超过
  `max_len` 的字符串长度都会作为 I/O error 返回。
- strict LEB128 方法拒绝非 canonical 编码。持久化或跨平台格式应优先采用固定宽度
  长度字段或 `u64` LEB128 长度，而非依赖目标平台宽度的 `usize` helper。

## 排障与限制

| 症状 | 首先检查 |
| --- | --- |
| 解出的值不正确 | writer 与 reader 是否使用相同的字节序和表示。 |
| 读取字符串失败 | 长度前缀方法是否匹配，`max_len` 是否兼容。 |
| 底层输出缺少数据 | 检查底层 sink 前是否 flush 了 buffered writer。 |
| 重试异步操作出现重复数据 | 把操作视为部分完成，并采用 framing 或事务性传输。 |

本 crate 面向流，不负责文件和运行时。需要一次导入全部扩展 trait 时可使用
`qubit_io_binary::prelude::*`；codec 与 byte-order 类型应从各自所属 crate 导入。

## 延伸阅读

- [README](../README.zh_CN.md) 与 [English README](../README.md)
- [English user guide](user_guide.md)
- [API 文档](https://docs.rs/qubit-io-binary)
