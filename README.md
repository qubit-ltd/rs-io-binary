# Qubit IO Binary

[![Rust CI](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-io-binary/coverage-badge.json)](https://qubit-ltd.github.io/rs-io-binary/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-io-binary.svg?color=blue)](https://crates.io/crates/qubit-io-binary)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-io-binary` lets Rust applications read and write a structured binary
protocol without coupling the protocol to a file type or async runtime. It
adapts the buffer codecs in `qubit-codec-binary` to `qubit-io` streams:

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

The quick start below needs only `qubit-io-binary`. Add `qubit-codec` or
`qubit-codec-binary` when importing their byte-order or decode-policy types,
and add `qubit-io` when writing generic `Input` or `Output` bounds.

## Quick Start: a Synchronous Record

Standard `Read` and `Write` implementations, including `Cursor` and `Vec<u8>`,
implement the `qubit-io` abstractions through adapters.

```rust
use std::io::Cursor;

use qubit_io_binary::{
    BinaryReadExt,
    BinaryWriteExt,
    Leb128ReadExt,
    Leb128WriteExt,
};

let mut bytes = Vec::new();
bytes.write_u32_le(0x0102_0304)?;
bytes.write_uleb_u64(300)?;

let mut input = Cursor::new(bytes);
assert_eq!(0x0102_0304, input.read_u32_le()?);
assert_eq!(300, input.read_uleb_u64_non_strict()?);
# Ok::<(), std::io::Error>(())
```

## Runtime-Neutral Async

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
    I: AsyncInput<Item = u8> + Unpin,
    O: AsyncOutput<Item = u8> + Unpin,
{
    let version = input.read_u32_le_async().await?;
    output.write_u32_le_async(version).await
}
```

Asynchronous extension methods only require `Unpin`; they do not impose a
runtime or `Send` requirement. These operations are not cancellation safe:
dropping a pending read retains bytes already consumed, and dropping a pending
write leaves any already-written prefix in the output.

## What It Provides

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

Varint readers require an explicit `Strict` or `NonStrict` policy type. Strict
readers keep the short method names such as `read_u64`; permissive readers use
the explicit `read_u64_non_strict` (and corresponding string) methods.

Buffered readers provide `into_parts` to recover the wrapped input together
with unread prefetched bytes. Buffered writers provide `into_parts` to recover
the wrapped output together with bytes not yet written. These methods perform
no I/O: flush first for normal completion, and retain the writer to retry a
flush failure.

Strict LEB128 methods reject non-canonical encodings. String read methods accept
a maximum payload length to bound allocation. For persistent formats, prefer
fixed-width length fields or `u64` LEB128 lengths over target-width `usize`
helpers.

High-throughput string readers can call `read_utf8_payload_into` (or the async
variant) with a reusable `Vec<u8>` to avoid reallocating payload storage for
each record. The buffer is retained on invalid UTF-8 errors for diagnostics.

## Mixed Binary Stream Benchmark

The 2026-08-04 Criterion run below used a deterministic random mix of `u8`,
`i32`, `u64`, and multibyte UTF-8 strings (131,072 fields per iteration). The
full command and environment are recorded in
[`benches/results/2026-08-04-mixed-binary-pipeline.md`](benches/results/2026-08-04-mixed-binary-pipeline.md).
The reusable payload-buffer comparison is recorded in
[`benches/results/2026-08-07-string-buffer-reuse.md`](benches/results/2026-08-07-string-buffer-reuse.md).
The in-memory mixed-pipeline comparison is recorded in
[`benches/results/2026-08-07-memory-mixed-binary-pipeline.md`](benches/results/2026-08-07-memory-mixed-binary-pipeline.md).
Treat these values as a comparison of buffering strategies, not as portable
performance guarantees.

The benchmark suite is split into layers: `micro_binary_pipeline` uses
short-write in-memory streams, `memory_mixed_binary_pipeline` compares mixed
records without filesystem effects, and the `prod_*` groups measure file-backed
end-to-end pipelines. `async_binary_pipeline` separately measures the
runtime-neutral asynchronous adapters. Set `QUBIT_IO_STREAM_BENCH_GROUP` to run
one group at a time; this keeps adapter cost, buffering cost, and filesystem
cost distinguishable.

| Scenario | Time | Throughput | Relative to raw extension |
| --- | ---: | ---: | ---: |
| Write: raw extension | 65.19 ms | 2.01 M fields/s | 1.0× |
| Write: extension + `BufWriter` | 4.04 ms | 32.44 M fields/s | 16.1× |
| Write: `BufferedBinaryWriter` | 3.70 ms | 35.40 M fields/s | 17.6× |
| Read: raw extension | 46.80 ms | 2.80 M fields/s | 1.0× |
| Read: extension + `BufReader` | 3.64 ms | 36.01 M fields/s | 12.9× |
| Read: `BufferedBinaryReader` | 3.46 ms | 37.90 M fields/s | 13.5× |

The buffered wrappers were about 9% faster for writes and 5% faster for reads
than the corresponding externally buffered extension paths in this run.

## Boundaries and Further Reading

- `qubit-codec-binary` owns buffer-level binary algorithms.
- `qubit-io` owns generic synchronous and runtime-neutral asynchronous streams.
- `qubit-io-binary` composes the two without owning files or runtimes.

Use this crate for binary values on an existing stream. It does not open files
or own an async runtime. For a scenario-led tutorial, see the
[user guide](doc/user_guide.md) or [中文用户指南](doc/user_guide.zh_CN.md); for
every public item, see the [API reference](https://docs.rs/qubit-io-binary).

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
