# Qubit IO Binary User Guide

[中文](user_guide.zh_CN.md) · [README](../README.md) ·
[API reference](https://docs.rs/qubit-io-binary)

This guide is for Rust applications that read or write structured values on a
byte stream. It covers `qubit-io-binary` 0.3: a runtime-neutral bridge between
the buffer codecs in `qubit-codec-binary` and the stream abstractions in
`qubit-io`. It does not open files, choose a transport, or select an async
runtime.

## Conceptual Model

`qubit-codec-binary` owns binary representations; `qubit-io` supplies generic
synchronous `Input`/`Output` and asynchronous `AsyncInput`/`AsyncOutput`; this
crate applies the codecs to those streams.

| Wire representation | Synchronous API | Asynchronous API |
| --- | --- | --- |
| Fixed-width scalars | `BinaryReadExt`, `BinaryWriteExt` | `AsyncBinaryReadExt`, `AsyncBinaryWriteExt` |
| LEB128 | `Leb128ReadExt`, `Leb128WriteExt` | `AsyncLeb128ReadExt`, `AsyncLeb128WriteExt` |
| ZigZag integers | `ZigZagReadExt`, `ZigZagWriteExt` | `AsyncZigZagReadExt`, `AsyncZigZagWriteExt` |
| Length-prefixed UTF-8 | `StringReadExt`, `StringWriteExt` | `AsyncStringReadExt`, `AsyncStringWriteExt` |

Use extension traits when an operation only needs one stream call. Use the
typed reader and writer families when a stream has one enduring byte order or
decode policy. Their buffered variants preserve unread or unwritten bytes and
are useful for many small operations.

## Scenario: Write and Read a Small Record

Imagine a file format with a big-endian format marker, a compact unsigned
record id, and a UTF-8 label whose length is encoded as `u64` LEB128. The
successful result is the same values after a round trip.

## Installation

```toml
[dependencies]
qubit-io-binary = "0.3"
qubit-codec = "0.10"
qubit-io = "0.14"
```

The examples use `Vec<u8>` and `Cursor`; standard-library byte sources and
sinks participate through `qubit-io` adapters.

## Core Workflow

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

The non-strict extension methods are intentionally named with a
`_non_strict` suffix. Use the corresponding `_strict` methods when canonical
LEB128 encodings are required; typed readers likewise require an explicit
`Strict` or `NonStrict` policy type.

The `max_len` argument is an input-validation boundary: it limits the payload
length accepted before allocating the returned `String`. Pick it from the file
format or protocol limit, rather than from currently expected data.

For fixed byte order, use `_be` and `_le` methods. For example,
`write_i64_le(-42)` writes a little-endian integer. `uleb` methods handle
unsigned LEB128, `sleb` methods handle signed LEB128, and ZigZag combines a
signed mapping with a LEB128 payload for compact values near zero.

## Typed and Buffered Streams

When a format consistently uses one order, a typed wrapper keeps that choice
in its type:

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

`Buffered*Reader::into_parts()` returns the wrapped input and unread prefetched
bytes. `Buffered*Writer::into_parts()` returns the wrapped output and bytes not
yet written. Neither method performs I/O: flush a writer first for normal
completion, and retain it if a flush fails so that it can be retried.

## Async Workflow

Async counterparts add an `_async` suffix. Their futures are `Send`, so the
stream must implement `Send + Unpin` in addition to the relevant async trait.

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

These high-level operations are not cancellation safe. Cancelling a pending
read can leave part of a value consumed; cancelling a pending write can leave a
prefix in the output. Retry only when the surrounding transport or framing
makes that safe.

## Errors and Diagnostics

- A stream ending in the middle of a scalar, prefix, or payload reports
  `UnexpectedEof`.
- Malformed codec data, non-canonical data rejected by strict LEB128 methods,
  invalid UTF-8, and a string length above `max_len` are reported as I/O errors.
- Strict LEB128 methods reject non-canonical encodings. For persistent or
  cross-platform formats, prefer fixed-width length fields or `u64` LEB128
  lengths over target-width `usize` helpers.

## Troubleshooting and Limits

| Symptom | Check first |
| --- | --- |
| Values decode incorrectly | Ensure writer and reader use the same byte order and representation. |
| A string read fails | Verify the chosen length-prefix method and set a compatible `max_len`. |
| Buffered output is missing | Flush the buffered writer before inspecting the underlying sink. |
| Retrying async work duplicates data | Treat the operation as partially completed and use framing or a transactional transport. |

The crate is stream-oriented rather than file-oriented, and it does not provide
a runtime. Import `qubit_io_binary::prelude::*` when bringing all extension
traits into scope is convenient; import codec and byte-order types from their
own crates.

## Further Reading

- [README](../README.md) and [中文 README](../README.zh_CN.md)
- [中文用户指南](user_guide.zh_CN.md)
- [API reference](https://docs.rs/qubit-io-binary)
