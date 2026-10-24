# Qubit IO Binary User Guide

[中文](user_guide.zh_CN.md) · [README](../README.md) ·
[API reference](https://docs.rs/qubit-io-binary)

This guide is for Rust applications that read or write structured values on a
byte stream. It covers `qubit-io-binary` 0.7.1: a runtime-neutral bridge between
the buffer codecs in `qubit-codec-binary` and the stream abstractions in
`qubit-io`. It does not open files, choose a transport, or select an async
runtime.

## Conceptual Model

`qubit-codec-binary` owns binary representations; `qubit-io` supplies generic
synchronous `ByteInput`/`ByteOutput` and asynchronous `AsyncInput`/`AsyncOutput`; this
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
qubit-io-binary = "0.7"
qubit-codec = { version = "0.17", features = ["io"] }
qubit-codec-binary = "0.9"
qubit-io = "0.19"
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

Integer LEB128 extension methods name their decoding policy explicitly:
`_non_strict` accepts non-canonical prefixes, while `_strict` requires a
canonical prefix. String methods that read a `u64` or `usize` ULEB128 length
without a policy suffix, such as `read_utf8_string_uleb_u64`, are non-strict;
their async counterparts are also non-strict. Use the `_strict` and
`_strict_async` string methods to require a canonical length prefix. This
strictness choice applies only to the prefix encoding: every string-reading
variant still checks `max_len` and validates the payload as UTF-8. Typed
LEB128 readers require an explicit `Strict` or `NonStrict` policy type.

The `max_len` argument is an input-validation boundary: it limits the payload
length accepted before allocating the returned `String`. Pick it from the file
format or protocol limit, rather than from currently expected data.

When decoding many strings, reuse a caller-owned byte buffer to avoid a fresh
allocation for every payload:

```rust
use std::io::Cursor;

use qubit_io_binary::StringReadExt;

let mut input = Cursor::new(b"hello world".to_vec());
let mut payload = Vec::with_capacity(32);
input.read_utf8_payload_into(&mut payload, 11, 64)?;
assert_eq!(payload, b"hello world");
# Ok::<(), std::io::Error>(())
```

The reusable API clears the buffer only after the length passes `max_len`; on
an invalid UTF-8 error it retains the received bytes for diagnostics.

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

LEB128 readers select their canonicality policy in the type parameter. Import
the policy from `qubit-codec-binary` when constructing a typed reader:

```rust
use std::io::Cursor;

use qubit_codec_binary::NonStrict;
use qubit_io_binary::Leb128Reader;

let mut reader =
    Leb128Reader::<_, NonStrict>::new(Cursor::new(vec![0xac, 0x02]));
assert_eq!(300, reader.read_u16_non_strict()?);
# Ok::<(), std::io::Error>(())
```

`Buffered*Reader::into_parts()` returns the wrapped input and unread prefetched
bytes. `Buffered*Writer::into_parts()` returns the wrapped output and a
`BufferedOutputRecovery`: `Pending` bytes were not reported as written, while
`Indeterminate` means a panicking write may have produced an external prefix
and must not be replayed automatically. Neither method performs I/O: flush a
writer first for normal completion, and retain it if a flush fails so that it
can be retried.

## Async Workflow

Async counterparts add an `_async` suffix. They only require `Unpin` in addition
to the relevant async trait and do not impose a runtime or `Send` requirement.

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

These high-level operations are not cancellation safe. Cancelling a pending
read can leave part of a value consumed; cancelling a pending write can leave a
prefix in the output. Retry only when the surrounding transport or framing
makes that safe.

## Errors and Diagnostics

Synchronous reads are not transactional. A failed scalar or varint read may
have consumed part of its field; calling it again starts at the current input
position. String reads have two distinct validation stages:

| Failure stage | Bytes consumed | Recovery |
| --- | --- | --- |
| Decoded length exceeds `max_len` | Length prefix; payload remains unread | Stop parsing this record. The current position is not the next record boundary. |
| Complete payload is invalid UTF-8 | Length prefix and complete payload | Discard the record only when an outer protocol already provides its complete frame boundary. |
| Synchronous I/O error during a scalar | Possibly part of the field | Do not blindly retry. Recover from trusted framing or a seek checkpoint. |
| Async operation is cancelled | Possibly part of the prefix or payload | Treat consumed bytes as retained; cancellation does not roll the operation back. |

A buffered reader may prefetch bytes from its underlying input. The underlying
input position therefore does not describe the reader's logical consumption,
and buffering does not make a whole record transactional.

Synchronous writes are not transactional either. A string write may fail
after its length prefix and part of its payload have been accepted. Do not
retry the whole string method without an application-defined recovery
boundary: doing so can append another length prefix.

For buffered writers, retrying flush only delivers bytes already retained
by the buffer. Pending from into_parts describes those retained bytes,
not the complete remainder of a failed string call. Pending can be empty,
and a later flush can succeed, while the original field is still incomplete.
Use the enclosing frame, seek/rollback policy, or transactional transport
to decide how to recover. These methods do not provide automatic rollback.

When an outer protocol has already isolated complete frames, parse each frame
through its own bounded input. A malformed frame can then be rejected without
continuing through its remaining fields:

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

This approach relies on a frame boundary the application already knows. The
library does not provide transport framing, and skipping an arbitrary number
of bytes after an excessive length cannot safely guess where the next frame
begins.

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
