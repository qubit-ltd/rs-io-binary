# Qubit IO Binary

[![Rust CI](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-io-binary/coverage-badge.json)](https://qubit-ltd.github.io/rs-io-binary/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-io-binary.svg?color=blue)](https://crates.io/crates/qubit-io-binary)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Runtime-neutral synchronous and asynchronous binary stream I/O for Rust.

`qubit-io-binary` adapts the buffer codecs in `qubit-codec-binary` to the
stream abstractions in `qubit-io`:

- synchronous extensions use `Input<Item = u8>` and `Output<Item = u8>`;
- asynchronous extensions use `AsyncInput<Item = u8>` and
  `AsyncOutput<Item = u8>`;
- both layers share the same fixed-width, LEB128, ZigZag, and UTF-8 codec
  semantics;
- typed synchronous readers and writers can carry byte order or decode policy;
- buffered synchronous wrappers amortize repeated small operations.

The core API does not depend on Tokio, `futures-io`, or another executor.
`qubit-io` supplies opt-in adapters for ecosystem stream traits.

## Installation

```toml
[dependencies]
qubit-io-binary = "0.3"
```

## Synchronous Example

Standard `Read` and `Write` implementations, including `Cursor` and `Vec<u8>`,
implement the `qubit-io` abstractions through adapters.

```rust
use std::io::Cursor;

use qubit_codec::ByteOrder;
use qubit_io_binary::{
    BinaryReadExt,
    BinaryWriteExt,
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

## Asynchronous Example

Async method names use the `_async` suffix so a type may expose both sync and
async stream APIs without ambiguity.

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
    I: AsyncInput<Item = u8> + Send + Unpin,
    O: AsyncOutput<Item = u8> + Send + Unpin,
{
    let version = input.read_u32_le_async().await?;
    output.write_u32_le_async(version).await
}
```

All asynchronous extension methods return `Send` futures and therefore require
the input or output to implement both `Send` and `Unpin`. These operations are
not cancellation safe: dropping a pending read retains bytes already consumed,
and dropping a pending write leaves any already-written prefix in the output.

## API Families

| Encoding | Synchronous traits | Asynchronous traits |
| --- | --- | --- |
| Fixed-width scalars | `BinaryReadExt`, `BinaryWriteExt` | `AsyncBinaryReadExt`, `AsyncBinaryWriteExt` |
| LEB128 | `Leb128ReadExt`, `Leb128WriteExt` | `AsyncLeb128ReadExt`, `AsyncLeb128WriteExt` |
| ZigZag | `ZigZagReadExt`, `ZigZagWriteExt` | `AsyncZigZagReadExt`, `AsyncZigZagWriteExt` |
| Length-prefixed UTF-8 | `StringReadExt`, `StringWriteExt` | `AsyncStringReadExt`, `AsyncStringWriteExt` |

Synchronous typed wrappers include `BinaryReader`, `BinaryWriter`,
`Leb128Reader`, `Leb128Writer`, `ZigZagReader`, and `ZigZagWriter`, together
with buffered variants. They are generic over `Input` and `Output`; they are not
defined in terms of `std::io::Read` and `std::io::Write`.

Buffered readers provide `into_parts` to recover the wrapped input together
with unread prefetched bytes; `into_inner` intentionally discards those unread
bytes. Buffered writers provide a flushing `into_inner` that returns an
`IntoInnerError` retaining the complete wrapper when flushing fails.

Strict LEB128 methods reject non-canonical encodings. String read methods accept
a maximum payload length to bound allocation. For persistent formats, prefer
fixed-width length fields or `u64` LEB128 lengths over target-width `usize`
helpers.

## Layering

- `qubit-codec-binary` owns buffer-level binary algorithms.
- `qubit-io` owns generic synchronous and runtime-neutral asynchronous streams.
- `qubit-io-binary` composes the two without owning files or runtimes.

See the [user guide](doc/user_guide.md) and
[API reference](https://docs.rs/qubit-io-binary) for details.

## Testing

```bash
# Run tests with the default feature set
cargo test

# Run tests with all declared features
cargo test --all-features

# Project CI checks
./ci-check.sh

# Check code coverage
./coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public
API documentation and tests current, and run `./align-ci.sh` to format code and
`./ci-check.sh` to satisfy CI requirements before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-io-binary](https://github.com/qubit-ltd/rs-io-binary)
