# Qubit IO Binary User Guide

Use `qubit-io-binary` when a byte stream carries structured binary values. The
crate deliberately separates three concerns:

- `qubit-codec-binary` encodes and decodes in-memory buffers;
- `qubit-io` defines synchronous and runtime-neutral asynchronous streams;
- `qubit-io-binary` drives the codecs over those streams.

It does not open files and does not select an async runtime.

## Installation

```toml
[dependencies]
qubit-io-binary = "0.2"
```

## Choosing an API

| Data | Synchronous | Asynchronous |
| --- | --- | --- |
| Fixed-width integers and floats | `BinaryReadExt`, `BinaryWriteExt` | `AsyncBinaryReadExt`, `AsyncBinaryWriteExt` |
| Unsigned/signed LEB128 | `Leb128ReadExt`, `Leb128WriteExt` | `AsyncLeb128ReadExt`, `AsyncLeb128WriteExt` |
| ZigZag signed integers | `ZigZagReadExt`, `ZigZagWriteExt` | `AsyncZigZagReadExt`, `AsyncZigZagWriteExt` |
| Length-prefixed UTF-8 | `StringReadExt`, `StringWriteExt` | `AsyncStringReadExt`, `AsyncStringWriteExt` |

Synchronous traits extend `Input<Item = u8>` or `Output<Item = u8>`.
Asynchronous traits extend `AsyncInput<Item = u8>` or
`AsyncOutput<Item = u8>`. Importing `qubit_io_binary::prelude::*` brings both
sets into scope.

## Fixed-Width Values

Use runtime byte order when it comes from format metadata:

```rust
use std::io::Cursor;

use qubit_codec::ByteOrder;
use qubit_io_binary::{BinaryReadExt, BinaryWriteExt};

let mut bytes = Vec::new();
bytes.write_u32(0x0102_0304, ByteOrder::BigEndian)?;

let mut input = Cursor::new(bytes);
assert_eq!(0x0102_0304, input.read_u32(ByteOrder::BigEndian)?);
# Ok::<(), std::io::Error>(())
```

Use `_be` and `_le` methods when the format fixes the byte order:

```rust
use qubit_io_binary::BinaryWriteExt;

let mut bytes = Vec::new();
bytes.write_i64_le(-42)?;
# Ok::<(), std::io::Error>(())
```

The async layer preserves the same naming and adds `_async`:

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

`uleb` methods encode unsigned integers. `sleb` methods encode signed integers.

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

Normal readers accept any terminating representation that fits the target type.
Strict methods, such as `read_uleb_u64_strict`, additionally reject
non-canonical representations. Typed readers select the same behavior with
`Leb128Reader<R, NonStrict>` or `Leb128Reader<R, Strict>`.

For persistent formats, prefer `u32` or `u64` methods over `usize`; the latter
changes width with the compilation target.

## ZigZag

ZigZag maps signed values to unsigned LEB128 payloads so values near zero remain
compact on both sides of zero.

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

## Length-Prefixed UTF-8

String helpers write a length followed by an exact UTF-8 payload.

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

Every string read accepts `max_len`. Treat it as an input validation boundary,
not merely a performance hint. Prefer fixed-width `u16`/`u32` lengths or `u64`
LEB128 lengths for portable file formats.

## Typed Synchronous Wrappers

Typed wrappers carry configuration when a format uses one dominant encoding:

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

The wrapper families are:

- `BinaryReader` / `BinaryWriter`;
- `Leb128Reader` / `Leb128Writer`;
- `ZigZagReader` / `ZigZagWriter`;
- their `Buffered*` variants.

They are generic over `Input` and `Output`. A standard `Read` or `Write` type is
usable because `qubit-io` adapts it, not because the wrapper is tied to the
standard traits.

Non-buffered wrappers expose direct inner access because they retain no
prefetched or pending bytes. Buffered wrappers retain state; perform raw
operations through the wrapper and explicitly flush writers before relying on
the underlying destination.

## Pending, Errors, and Cancellation

Async methods correctly retain local progress while returning `Poll::Pending`.
As with most multi-poll extension methods, dropping an operation after the
underlying stream has made progress may consume part of an input value or write
part of an output value. Do not cancel and blindly retry a whole binary
operation unless the underlying transport is transactional or independently
framed.

End-of-stream in the middle of a fixed-width value, length prefix, or payload is
an `UnexpectedEof`. Codec validation failures remain explicit I/O errors rather
than silently producing a partial value.
