# Qubit IO Binary User Guide

Use `qubit-io-binary` when bytes in a `Read` or `Write` stream represent binary
values. The crate does not own file handling, copying, or generic stream
wrappers; those remain in `qubit-io`.

## Capability Map

| Area | API | Purpose |
| --- | --- | --- |
| Fixed-width scalars | `BinaryReadExt`, `BinaryWriteExt` | read and write integers and floats with explicit byte order |
| LEB128 | `Leb128ReadExt`, `Leb128WriteExt` | compact unsigned and signed integer streams |
| ZigZag | `ZigZagReadExt`, `ZigZagWriteExt` | compact signed integers that are usually near zero |
| Strings | `StringReadExt`, `StringWriteExt` | length-prefixed UTF-8 strings |
| Wrappers | `BinaryReader`, `Leb128Reader`, `ZigZagReader` and writer counterparts | typed wrapper APIs around existing streams |
| Buffered wrappers | `BufferedBinaryReader`, `BufferedLeb128Reader`, `BufferedZigZagReader` and writer counterparts | wrapper-owned buffering for file-backed streams |

## Installation

```toml
[dependencies]
qubit-io-binary = "0.1"
```

## Fixed-Width Scalars

Use runtime byte order methods when the byte order comes from format metadata.

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

Use `_be` or `_le` methods when the byte order is fixed by the API:

```rust
use qubit_io_binary::BinaryWriteExt;

let mut bytes = Vec::new();
bytes.write_i64_le(-42)?;
# Ok::<(), std::io::Error>(())
```

## LEB128

LEB128 methods are split into unsigned (`uleb`) and signed (`sleb`) families.

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

Non-strict readers accept any well-terminated representation that fits the target
type. Strict readers, such as `read_uleb_u64_strict`, also reject non-canonical
encodings. Typed readers select this behavior with `Leb128DecodePolicy`, using
`Leb128Reader<R, NonStrict>` or `Leb128Reader<R, Strict>`.

For persistent formats, prefer fixed-width integer methods such as
`write_uleb_u64` over target-width methods such as `write_uleb_usize`.

## ZigZag

ZigZag maps signed values to unsigned LEB128 payloads so small negative values
remain compact.

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

## Length-Prefixed UTF-8 Strings

The string helpers encode and decode UTF-8 payloads with explicit length
prefixes.

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

The `max_len` argument on read methods protects callers from oversized encoded
lengths. `read_utf8_string_uleb` and `write_utf8_string_uleb` use `usize` length
prefixes and are target-width dependent. Prefer the `u64` LEB128 string helpers
or fixed-width `u16` / `u32` length helpers for files and cross-platform
protocols.

## Reader and Writer Wrappers

Wrapper types provide a method-oriented API when a format has one dominant
encoding.

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

Non-buffered wrappers expose `inner()` and `inner_mut()` because they do not
hold prefetched or pending bytes. Buffered wrappers own an internal buffer and
are useful when wrapping unbuffered file-backed streams. They expose `inner()`
for inspection and `into_inner()` for recovery; use the wrapper's own `Read`,
`Write`, and `Seek` implementations for raw operations so buffered state remains
consistent.

## Relationship to Other Crates

- Use `qubit-codec-binary` when you only need buffer-level encode/decode.
- Use `qubit-io-binary` when the values are read from or written to streams.
- Use `qubit-io` for generic stream copying, seeking helpers, and wrappers.
