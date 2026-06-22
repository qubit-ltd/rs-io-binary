# Qubit IO Binary

[![Rust CI](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-io-binary/coverage-badge.json)](https://qubit-ltd.github.io/rs-io-binary/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-io-binary.svg?color=blue)](https://crates.io/crates/qubit-io-binary)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Chinese Document](https://img.shields.io/badge/Document-Chinese-blue.svg)](README.zh_CN.md)

Binary stream I/O adapters for Rust.

## Overview

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

## Design Goals

- **Stream Adapters Only**: keep buffer-level binary algorithms in
  `qubit-codec-binary`.
- **Ergonomic Extension Traits**: make common binary read/write calls concise on
  any `Read` or `Write`.
- **Typed Reader/Writer Wrappers**: provide stateful adapters when an API should
  carry byte order or decode policy.
- **Buffered Hot Paths**: support buffered readers and writers for repeated small
  binary operations.
- **Explicit Layering**: depend on `qubit-io` and `qubit-codec-binary` without
  pushing binary code back into `qubit-io`.

## Features

### Binary Extension Traits

- **`BinaryReadExt` / `BinaryWriteExt`**: fixed-width scalar reads and writes.
- **`Leb128ReadExt` / `Leb128WriteExt`**: unsigned and signed LEB128 reads and
  writes.
- **`ZigZagReadExt` / `ZigZagWriteExt`**: ZigZag signed integer reads and writes.
- **`StringReadExt` / `StringWriteExt`**: length-prefixed UTF-8 string helpers.

### Reader and Writer Wrappers

- **`BinaryReader` / `BinaryWriter`**: fixed-width scalar adapters.
- **`Leb128Reader` / `Leb128Writer`**: LEB128 adapters.
- **`ZigZagReader` / `ZigZagWriter`**: ZigZag adapters.
- **Buffered variants**: reduce repeated small-read or small-write overhead.

### Re-Exports

- **Binary codec types**: re-exported from `qubit-codec-binary`.
- **LEB128 policy types**: `Leb128DecodePolicy`, `Strict`, and `NonStrict` are
  available for typed reader configuration.
- **Core I/O traits**: selected generic helpers are re-exported from `qubit-io`.

## Documentation

- [User Guide](doc/user_guide.md)
- [API Reference](https://docs.rs/qubit-io-binary)
- [Chinese README](README.zh_CN.md)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
qubit-io-binary = "0.2"
```

## Quick Start

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

## API Reference

### Extension Traits

| Trait | Purpose |
|-------|---------|
| `BinaryReadExt` / `BinaryWriteExt` | Fixed-width scalar I/O |
| `Leb128ReadExt` / `Leb128WriteExt` | Unsigned and signed LEB128 I/O |
| `ZigZagReadExt` / `ZigZagWriteExt` | ZigZag signed integer I/O |
| `StringReadExt` / `StringWriteExt` | Length-prefixed UTF-8 string I/O |

### Stream Wrappers

| Type Family | Purpose |
|-------------|---------|
| `BinaryReader` / `BinaryWriter` | Carry byte-order configuration for fixed-width values |
| `Leb128Reader` / `Leb128Writer` | Read and write LEB128 values |
| `ZigZagReader` / `ZigZagWriter` | Read and write ZigZag signed integers |
| `Buffered*Reader` / `Buffered*Writer` | Batch repeated binary operations through an internal buffer |

Non-buffered wrappers expose `inner()` and `inner_mut()` because they hold no
prefetched or pending bytes. Buffered wrappers expose `inner()` for inspection
and `into_inner()` for recovery; mutate the stream through the wrapper itself so
the internal buffer stays consistent.

## Layering

- `qubit-codec-binary` contains buffer-level codecs.
- `qubit-io-binary` adapts those codecs to `Read` and `Write`.
- `qubit-io` remains the generic I/O utility layer.

## Performance Considerations

Extension traits perform direct reads and writes into stack buffers for small
values. Buffered wrappers amortize repeated small operations while preserving the
same public codec semantics.

For persistent formats, prefer fixed-width length fields such as
`write_utf8_string_uleb_u64` over target-width `usize` string length helpers.

## Testing & Code Coverage

This project keeps binary stream behavior covered by integration tests under
`tests/` and benchmark coverage under `benches/`.

### Running Tests

```bash
# Run all tests
cargo test

# Run stream benchmarks
benches/run_stream_bench_groups.sh

# Run one benchmark group
QUBIT_IO_STREAM_BENCH_GROUP=prod_varints cargo bench --bench stream

# Run with coverage report
./coverage.sh

# Generate text format report
./coverage.sh text

# Align code with CI requirements
./align-ci.sh

# Run CI checks (format, clippy, test, coverage, audit)
RS_CI_SKIP_TOOLCHAIN_UPDATE=1 ./ci-check.sh
```

## Dependencies

Runtime dependencies are intentionally small:

- `qubit-codec-binary` provides the buffer-level binary codecs.
- `qubit-io` provides generic stream helpers and extension primitives.

Development dependencies include `criterion` for benchmarks.

## License

Copyright (c) 2026. Haixing Hu.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

See [LICENSE](LICENSE) for the full license text.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines

- Keep stream adapters separate from buffer-level codecs.
- Cover error paths and partial-read or partial-write behavior.
- Keep benchmarks representative of stream adapter overhead.
- Ensure all checks pass before submitting a PR.

## Author

**Haixing Hu**

## Related Projects

- [qubit-codec-binary](https://github.com/qubit-ltd/rs-codec-binary): buffer-level
  binary codecs.
- [qubit-io](https://github.com/qubit-ltd/rs-io): generic `std::io` helpers.
- More Rust libraries from Qubit are available under the
  [qubit-ltd](https://github.com/qubit-ltd) GitHub organization.

---

Repository: [https://github.com/qubit-ltd/rs-io-binary](https://github.com/qubit-ltd/rs-io-binary)
