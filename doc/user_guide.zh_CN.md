# Qubit IO Binary 用户指南

当字节流承载结构化二进制值时，使用 `qubit-io-binary`。本 crate 有意把三类
职责分开：

- `qubit-codec-binary` 负责内存缓冲区的编码和解码；
- `qubit-io` 定义同步流与运行时无关的异步流；
- `qubit-io-binary` 驱动 codec 在这些流上工作。

它不负责打开文件，也不选择异步运行时。

## 安装

```toml
[dependencies]
qubit-io-binary = "0.2"
```

## 选择 API

| 数据 | 同步 API | 异步 API |
| --- | --- | --- |
| Fixed-width 整数和浮点数 | `BinaryReadExt`、`BinaryWriteExt` | `AsyncBinaryReadExt`、`AsyncBinaryWriteExt` |
| Unsigned/signed LEB128 | `Leb128ReadExt`、`Leb128WriteExt` | `AsyncLeb128ReadExt`、`AsyncLeb128WriteExt` |
| ZigZag signed integer | `ZigZagReadExt`、`ZigZagWriteExt` | `AsyncZigZagReadExt`、`AsyncZigZagWriteExt` |
| 带长度前缀的 UTF-8 | `StringReadExt`、`StringWriteExt` | `AsyncStringReadExt`、`AsyncStringWriteExt` |

同步 trait 扩展 `Input<Item = u8>` 或 `Output<Item = u8>`；异步 trait 扩展
`AsyncInput<Item = u8>` 或 `AsyncOutput<Item = u8>`。导入
`qubit_io_binary::prelude::*` 可一次引入两套扩展。

## Fixed-Width 值

当字节序来自格式元数据时，使用运行时 byte order：

```rust
use std::io::Cursor;

use qubit_io_binary::{
    BinaryReadExt,
    BinaryWriteExt,
    ByteOrder,
};

let mut bytes = Vec::new();
bytes.write_u32(0x0102_0304, ByteOrder::BigEndian)?;

let mut input = Cursor::new(bytes);
assert_eq!(0x0102_0304, input.read_u32(ByteOrder::BigEndian)?);
# Ok::<(), std::io::Error>(())
```

当格式固定字节序时，使用 `_be` 或 `_le` 方法：

```rust
use qubit_io_binary::BinaryWriteExt;

let mut bytes = Vec::new();
bytes.write_i64_le(-42)?;
# Ok::<(), std::io::Error>(())
```

异步层保持相同命名，并添加 `_async` 后缀：

```rust
use qubit_io::{
    AsyncInput,
    AsyncOutput,
};
use qubit_io_binary::{
    AsyncBinaryReadExt,
    AsyncBinaryWriteExt,
};

async fn relay<I, O>(input: &mut I, output: &mut O) -> std::io::Result<()>
where
    I: AsyncInput<Item = u8> + Unpin,
    O: AsyncOutput<Item = u8> + Unpin,
{
    let value = input.read_i64_le_async().await?;
    output.write_i64_le_async(value).await
}
```

## LEB128

`uleb` 方法编码 unsigned integer，`sleb` 方法编码 signed integer。

```rust
use std::io::Cursor;

use qubit_io_binary::{
    Leb128ReadExt,
    Leb128WriteExt,
};

let mut bytes = Vec::new();
bytes.write_uleb_u64(300)?;
bytes.write_sleb_i64(-42)?;

let mut input = Cursor::new(bytes);
assert_eq!(300, input.read_uleb_u64()?);
assert_eq!(-42, input.read_sleb_i64()?);
# Ok::<(), std::io::Error>(())
```

普通 reader 接受能正常终止且适配目标类型的表示。Strict 方法（例如
`read_uleb_u64_strict`）还会拒绝非 canonical 表示。Typed reader 通过
`Leb128Reader<R, NonStrict>` 或 `Leb128Reader<R, Strict>` 选择相同策略。

持久化格式应优先使用 `u32` 或 `u64` 方法；`usize` 的宽度会随编译目标变化。

## ZigZag

ZigZag 把 signed 值映射为 unsigned LEB128 payload，使零附近的正负值都保持
紧凑。

```rust
use std::io::Cursor;

use qubit_io_binary::{
    ZigZagReadExt,
    ZigZagWriteExt,
};

let mut bytes = Vec::new();
bytes.write_zig_zag_i32(-15)?;

let mut input = Cursor::new(bytes);
assert_eq!(-15, input.read_zig_zag_i32()?);
# Ok::<(), std::io::Error>(())
```

## 带长度前缀的 UTF-8

字符串 helper 先写入长度，再写入精确的 UTF-8 payload。

```rust
use std::io::Cursor;

use qubit_io_binary::{
    StringReadExt,
    StringWriteExt,
};

let mut bytes = Vec::new();
bytes.write_utf8_string_uleb_u64("hello")?;

let mut input = Cursor::new(bytes);
assert_eq!("hello", input.read_utf8_string_uleb_u64(16)?);
# Ok::<(), std::io::Error>(())
```

每个字符串读取方法都要求 `max_len`。它是输入校验边界，不只是性能提示。
可移植文件格式应优先使用固定宽度的 `u16`/`u32` 长度或 `u64` LEB128 长度。

## 同步 Typed Wrapper

当一种格式以某个编码为主时，typed wrapper 可携带配置：

```rust
use std::io::Cursor;

use qubit_io_binary::{
    BinaryReader,
    BinaryWriter,
    LittleEndian,
};

let mut writer = BinaryWriter::<_, LittleEndian>::new(Vec::new());
writer.write_u16(0x1234)?;

let bytes = writer.into_inner();
let mut reader = BinaryReader::<_, LittleEndian>::new(Cursor::new(bytes));
assert_eq!(0x1234, reader.read_u16()?);
# Ok::<(), std::io::Error>(())
```

Wrapper 类型族包括：

- `BinaryReader` / `BinaryWriter`；
- `Leb128Reader` / `Leb128Writer`；
- `ZigZagReader` / `ZigZagWriter`；
- 它们的 `Buffered*` 变体。

这些类型以 `Input` / `Output` 为泛型边界。标准库 `Read` / `Write` 类型可以
使用，是因为 `qubit-io` 提供了 adapter，而不是因为 wrapper 绑定了标准 trait。

非 buffered wrapper 没有预读或待写字节，因此允许直接访问底层对象。Buffered
wrapper 会保留状态；raw 操作应通过 wrapper 执行，writer 在依赖底层目标前必须
显式 flush。

## Pending、错误与取消

异步方法在返回 `Poll::Pending` 时会正确保留本次 future 的局部进度。但和多数
跨多次 poll 的扩展方法一样，如果底层流已经发生进展后直接丢弃 future，可能已经
消费了一个输入值的前半部分，或写出了一个输出值的前半部分。除非底层传输具有事务
或独立分帧能力，否则不要在取消后盲目重试完整操作。

在 fixed-width 值、长度前缀或 payload 中途遇到流结尾会返回
`UnexpectedEof`。Codec 校验失败会作为明确 I/O 错误返回，不会静默产生部分值。
