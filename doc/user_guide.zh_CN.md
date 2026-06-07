# Qubit IO Binary 用户指南

当 `Read` 或 `Write` stream 中的字节代表二进制值时，使用 `qubit-io-binary`。
本 crate 不负责文件处理、通用复制或通用 stream wrapper；这些能力保留在 `qubit-io`。

## 能力地图

| 领域 | API | 用途 |
| --- | --- | --- |
| Fixed-width 标量 | `BinaryReadExt`、`BinaryWriteExt` | 使用显式字节序读写整数和浮点数 |
| LEB128 | `Leb128ReadExt`、`Leb128WriteExt` | 紧凑的 unsigned / signed 整数 stream |
| ZigZag | `ZigZagReadExt`、`ZigZagWriteExt` | 通常接近 0 的紧凑 signed 整数 |
| 字符串 | `StringReadExt`、`StringWriteExt` | length-prefixed UTF-8 字符串 |
| Wrapper | `BinaryReader`、`Leb128Reader`、`ZigZagReader` 及对应 writer | 包装现有 stream 的 typed API |
| Buffered wrapper | `BufferedBinaryReader`、`BufferedLeb128Reader`、`BufferedZigZagReader` 及对应 writer | wrapper 自带缓冲，适合 file-backed stream |

## 安装

```toml
[dependencies]
qubit-io-binary = "0.1"
```

## Fixed-Width 标量

当字节序来自格式元数据时，使用运行时 byte order 方法。

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

当字节序由 API 固定时，使用 `_be` 或 `_le` 方法：

```rust
use qubit_io_binary::BinaryWriteExt;

let mut bytes = Vec::new();
bytes.write_i64_le(-42)?;
# Ok::<(), std::io::Error>(())
```

## LEB128

LEB128 方法分为 unsigned (`uleb`) 和 signed (`sleb`) 两组。

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

非 strict reader 接受能正常终止且适配目标类型的表示。strict reader，例如
`read_uleb_u64_strict`，还会拒绝非 canonical 编码。Typed reader 通过
`Leb128DecodePolicy` 选择该行为，例如 `Leb128Reader<R, NonStrict>` 或
`Leb128Reader<R, Strict>`。

持久化格式优先使用 `write_uleb_u64` 这样的固定宽度方法，不要使用
`write_uleb_usize` 这类 target-width 方法。

## ZigZag

ZigZag 将 signed 值映射为 unsigned LEB128 payload，使小的负数也保持紧凑。

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

## Length-Prefixed UTF-8 字符串

字符串 helper 使用显式长度前缀编码和解码 UTF-8 payload。

```rust
use std::io::Cursor;

use qubit_io_binary::{
    StringReadExt,
    StringWriteExt,
};

let mut bytes = Vec::new();
bytes.write_utf8_string_uleb_u64("hello")?;

let mut input = Cursor::new(bytes);
let value = input.read_utf8_string_uleb_u64(16)?;

assert_eq!("hello", value);
# Ok::<(), std::io::Error>(())
```

读取方法的 `max_len` 参数用于防止超大编码长度触发不受控分配。
`read_utf8_string_uleb` 和 `write_utf8_string_uleb` 使用 `usize` 长度前缀，
是 target-width dependent。文件格式和跨平台协议优先使用 `u64` LEB128
字符串 helper，或固定宽度的 `u16` / `u32` 长度 helper。

## Reader 和 Writer Wrapper

当一种格式有主导编码方式时，wrapper 类型提供 method-oriented API。

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

非 buffered wrapper 暴露 `inner()` 和 `inner_mut()`，因为它们没有预读或待
flush 字节。Buffered wrapper 自带内部缓冲，适合包装未缓冲的 file-backed
stream；它们只暴露 `inner()` 用于查看，并通过 `into_inner()` 取回底层
stream。混合 raw I/O 时请使用 wrapper 自身的 `Read`、`Write` 和 `Seek` 实现，
以保持内部缓冲状态一致。

## 与其他 Crate 的关系

- 只需要缓冲区级 encode/decode 时，使用 `qubit-codec-binary`。
- 需要从 stream 读取或写入这些值时，使用 `qubit-io-binary`。
- 通用 stream 复制、seek helper 和通用 wrapper 使用 `qubit-io`。
