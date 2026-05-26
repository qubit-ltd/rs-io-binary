# Qubit IO Binary

[![Rust CI](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-io-binary/coverage-badge.json)](https://qubit-ltd.github.io/rs-io-binary/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-io-binary.svg?color=blue)](https://crates.io/crates/qubit-io-binary)
[![License](https://img.shields.io/crates/l/qubit-io-binary.svg)](LICENSE)

Binary stream I/O adapters for Rust.

`qubit-io-binary` builds on `qubit-io` and `qubit-codec-binary` to provide:

- extension traits for fixed-width binary scalars;
- extension traits for unsigned and signed LEB128 integers;
- extension traits for ZigZag encoded signed integers;
- length-prefixed UTF-8 string helpers;
- typed reader and writer wrappers such as `BinaryReader`, `Leb128Reader`,
  `ZigZagReader`, and buffered variants.

Generic `std::io` helpers such as `Streams`, `ReadExt`, and wrapper types such
as `CountingReader` live in `qubit-io`.

Detailed usage is documented in the [user guide](doc/user_guide.md). API
reference documentation is available on [docs.rs](https://docs.rs/qubit-io-binary).

## Installation

```toml
[dependencies]
qubit-io-binary = "0.1"
```

## Quick Example

```rust
use std::io::Cursor;

use qubit_io_binary::{
    BinaryReadExt,
    BinaryWriteExt,
    ByteOrder,
    Leb128ReadExt,
    Leb128WriteExt,
};

let mut bytes = Vec::new();
bytes.write_u16(0x1234, ByteOrder::BigEndian)?;
bytes.write_uleb_u32(300)?;

let mut input = Cursor::new(bytes);
assert_eq!(0x1234, input.read_u16(ByteOrder::BigEndian)?);
assert_eq!(300, input.read_uleb_u32()?);
# Ok::<(), std::io::Error>(())
```

## Layering

- `qubit-codec-binary` contains buffer-level codecs.
- `qubit-io-binary` adapts those codecs to `Read` and `Write`.
- `qubit-io` remains the generic I/O utility layer.

Repository: [https://github.com/qubit-ltd/rs-io-binary](https://github.com/qubit-ltd/rs-io-binary)
