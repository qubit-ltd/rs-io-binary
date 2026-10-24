# Qubit IO Binary 用户指南

[English](user_guide.md) · [README](../README.zh_CN.md) ·
[API 文档](https://docs.rs/qubit-io-binary)

本指南面向需要在字节流上读取或写入结构化二进制值的 Rust 应用，适用于
`qubit-io-binary` 0.7.1。该 crate 将 `qubit-codec-binary` 的缓冲区 codec 接入
`qubit-io` 的流抽象；它不负责打开文件、不选择传输层，也不绑定异步运行时。

## 概念模型

`qubit-codec-binary` 负责二进制表示，`qubit-io` 提供通用的同步
`ByteInput` / `ByteOutput` 与异步 `AsyncInput` / `AsyncOutput`，本 crate 则把 codec
应用到这些流上。

| 线上的表示 | 同步 API | 异步 API |
| --- | --- | --- |
| 固定宽度标量 | `BinaryReadExt`、`BinaryWriteExt` | `AsyncBinaryReadExt`、`AsyncBinaryWriteExt` |
| LEB128 | `Leb128ReadExt`、`Leb128WriteExt` | `AsyncLeb128ReadExt`、`AsyncLeb128WriteExt` |
| ZigZag 整数 | `ZigZagReadExt`、`ZigZagWriteExt` | `AsyncZigZagReadExt`、`AsyncZigZagWriteExt` |
| 带长度前缀的 UTF-8 | `StringReadExt`、`StringWriteExt` | `AsyncStringReadExt`、`AsyncStringWriteExt` |

仅执行一次流操作时使用扩展 trait；当一个流长期使用同一种字节序或解码策略时，使用
typed reader/writer。它们的 buffered 变体会保留未读或未写字节，适合大量小操作。

## 实战场景：写入并读取一条小记录

设想一种文件格式：先写一个大端 format marker，再写一个紧凑的无符号 record id，
最后写一个长度使用 `u64` LEB128 编码的 UTF-8 label。成功标准是 round trip 后得到
完全相同的值。

## 安装

```toml
[dependencies]
qubit-io-binary = "0.7"
qubit-codec = { version = "0.17", features = ["io"] }
qubit-codec-binary = "0.9"
qubit-io = "0.19"
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

连续读取很多字符串时，可以复用调用方提供的字节缓冲区，避免每个 payload 都重新分配：

```rust
use std::io::Cursor;

use qubit_io_binary::StringReadExt;

let mut input = Cursor::new(b"hello world".to_vec());
let mut payload = Vec::with_capacity(32);
input.read_utf8_payload_into(&mut payload, 11, 64)?;
assert_eq!(payload, b"hello world");
# Ok::<(), std::io::Error>(())
```

可复用接口只会在长度通过 `max_len` 校验后清空缓冲区；如果 UTF-8 无效，则保留已接收
的字节，便于诊断。

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

LEB128 reader 会把 canonicality 策略放在类型参数中。构造 typed reader 时，
请从 `qubit-codec-binary` 直接导入策略类型：

```rust
use std::io::Cursor;

use qubit_codec_binary::NonStrict;
use qubit_io_binary::Leb128Reader;

let mut reader =
    Leb128Reader::<_, NonStrict>::new(Cursor::new(vec![0xac, 0x02]));
assert_eq!(300, reader.read_u16_non_strict()?);
# Ok::<(), std::io::Error>(())
```

`Buffered*Reader::into_parts()` 返回被包装的输入和预取后尚未消费的字节；
`Buffered*Writer::into_parts()` 返回被包装的输出和 `BufferedOutputRecovery`：`Pending` 表示尚未被报告写出的字节；`Indeterminate` 表示写入 panic 时可能已产生外部前缀，不应自动重放。两者都不执行 I/O：正常完成时先 flush writer；flush 失败时应保留它，以便重试。

## 异步流程

异步 API 在方法名后加 `_async`。除实现相应异步 trait 外，流只需实现 `Unpin`，
不额外要求运行时或 `Send`。

```rust
use qubit_io::{AsyncInput, AsyncOutput};
use qubit_io_binary::{AsyncBinaryReadExt, AsyncBinaryWriteExt};

async fn relay<I, O>(input: &mut I, output: &mut O) -> std::io::Result<()>
where
    I: AsyncInput<Item = u8> + Unpin,
    O: AsyncOutput<Item = u8> + Unpin,
{
    let marker = input.read_u32_be_async().await?;
    output.write_u32_be_async(marker).await
}
```

这些高层操作不具备取消安全性。取消 pending read 可能已经消费一个值的部分字节；取消
pending write 可能已经向输出写入前缀。只有在传输或 framing 保证安全时才能整体重试。

## 错误与诊断

同步读取不提供事务回滚。标量或变长整数读取失败时，当前字段可能已有一部分字节被
消费；再次调用会从当前输入位置开始。字符串读取的校验阶段则有明确区别：

| 失败阶段 | 已消费的数据 | 恢复方式 |
| --- | --- | --- |
| 解出的长度超过 `max_len` | 长度前缀已消费，payload 尚未读取 | 停止解析当前记录；当前位置不是下一条记录的边界。 |
| 完整 payload 不是合法 UTF-8 | 长度前缀和完整 payload 均已消费 | 只有外层协议已给出完整帧边界时，才能丢弃该记录继续。 |
| 同步读取标量时发生 I/O 错误 | 可能已消费字段的一部分 | 不要盲目重试；应从可信帧边界或 seek 检查点恢复。 |
| 异步操作被取消 | 可能已消费部分前缀或 payload | 已消费字节会保留；取消不会回滚操作。 |

buffered reader 可能已从底层输入预读数据，因此底层输入位置不能代表 reader 的逻辑消费位置。
缓冲也不会让整条记录变成事务操作。

如果外层协议已经把完整帧隔离出来，可以为每帧单独创建输入并解析。某一帧格式错误时，
应用据此拒绝整帧，而不是继续读取该帧的内部字段：

```rust
use std::io::Cursor;
use qubit_io_binary::StringReadExt;

fn parse_frame(frame: &[u8]) -> std::io::Result<String> {
    Cursor::new(frame).read_utf8_string_uleb_u64(2)
}

// The application has already isolated these frames using its outer protocol.
assert!(parse_frame(&[3, b'a', b'b', b'c']).is_err());
assert_eq!("z", parse_frame(&[1, b'z'])?);
# Ok::<(), std::io::Error>(())
```

这里依赖的是应用已经掌握的帧边界。本库不负责 transport framing；长度超限后随意跳过
若干字节，也无法可靠推断下一帧从哪里开始。

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
