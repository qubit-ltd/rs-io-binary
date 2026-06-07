# Qubit IO Binary

[![Rust CI](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-io-binary/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-io-binary/coverage-badge.json)](https://qubit-ltd.github.io/rs-io-binary/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-io-binary.svg?color=blue)](https://crates.io/crates/qubit-io-binary)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

面向 Rust 的二进制 stream I/O adapter。

## 概述

`qubit-io-binary` 基于 `qubit-io` 和 `qubit-codec-binary`，提供：

- fixed-width binary scalar 的 extension trait；
- unsigned / signed LEB128 整数的 extension trait；
- ZigZag 有符号整数的 extension trait；
- length-prefixed UTF-8 字符串 helper；
- `BinaryReader`、`Leb128Reader`、`ZigZagReader` 以及 buffered 变体等 typed
  reader / writer wrapper。

`Streams`、`ReadExt` 等通用 `std::io` helper，以及 `CountingReader` 等通用 wrapper
位于 `qubit-io`。

详细用法请参见[中文用户指南](doc/user_guide.zh_CN.md)。API 参考文档可在
[docs.rs](https://docs.rs/qubit-io-binary) 查看。

## 设计目标

- **只做 Stream Adapter**：缓冲区级 binary 算法保留在 `qubit-codec-binary`。
- **易用的 Extension Trait**：让任意 `Read` 或 `Write` 上的常见二进制读写调用更简洁。
- **强类型 Reader/Writer Wrapper**：当 API 需要携带字节序或解码策略时，提供有状态 adapter。
- **Buffered 热路径**：为反复小型 binary 操作提供 buffered reader 和 writer。
- **分层明确**：依赖 `qubit-io` 和 `qubit-codec-binary`，不把 binary 能力放回 `qubit-io`。

## 特性

### Binary Extension Trait

- **`BinaryReadExt` / `BinaryWriteExt`**：fixed-width scalar 读写。
- **`Leb128ReadExt` / `Leb128WriteExt`**：unsigned / signed LEB128 读写。
- **`ZigZagReadExt` / `ZigZagWriteExt`**：ZigZag signed integer 读写。
- **`StringReadExt` / `StringWriteExt`**：length-prefixed UTF-8 string helper。

### Reader 与 Writer Wrapper

- **`BinaryReader` / `BinaryWriter`**：fixed-width scalar adapter。
- **`Leb128Reader` / `Leb128Writer`**：LEB128 adapter。
- **`ZigZagReader` / `ZigZagWriter`**：ZigZag adapter。
- **Buffered 变体**：降低反复小读写的开销。

### Re-Export

- **Binary codec 类型**：从 `qubit-codec-binary` 重导出。
- **LEB128 policy 类型**：重导出 `Leb128DecodePolicy`、`Strict` 和
  `NonStrict`，用于 typed reader 配置。
- **核心 I/O trait**：从 `qubit-io` 重导出部分通用 helper。

## 文档

- [中文用户指南](doc/user_guide.zh_CN.md)
- [API 文档](https://docs.rs/qubit-io-binary)
- [英文 README](README.md)

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
qubit-io-binary = "0.1"
```

## 快速开始

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

## API 参考

### Extension Trait

| Trait | 用途 |
|-------|------|
| `BinaryReadExt` / `BinaryWriteExt` | Fixed-width scalar I/O |
| `Leb128ReadExt` / `Leb128WriteExt` | Unsigned / signed LEB128 I/O |
| `ZigZagReadExt` / `ZigZagWriteExt` | ZigZag signed integer I/O |
| `StringReadExt` / `StringWriteExt` | Length-prefixed UTF-8 string I/O |

### Stream Wrapper

| 类型族 | 用途 |
|--------|------|
| `BinaryReader` / `BinaryWriter` | 为 fixed-width value 携带字节序配置 |
| `Leb128Reader` / `Leb128Writer` | 读写 LEB128 值 |
| `ZigZagReader` / `ZigZagWriter` | 读写 ZigZag signed integer |
| `Buffered*Reader` / `Buffered*Writer` | 通过内部缓冲批量处理重复 binary 操作 |

非 buffered wrapper 暴露 `inner()` 和 `inner_mut()`，因为它们没有预读或待
flush 状态。Buffered wrapper 只暴露 `inner()` 用于查看，并通过 `into_inner()`
取回底层 stream；需要混合 raw I/O 时应通过 wrapper 自身的 `Read` / `Write` /
`Seek` 实现操作，避免破坏内部缓冲状态。

## 分层

- `qubit-codec-binary` 提供缓冲区级 codec。
- `qubit-io-binary` 将这些 codec 适配到 `Read` 和 `Write`。
- `qubit-io` 保持为通用 I/O 工具层。

## 性能考虑

Extension trait 对小型值直接读写栈上缓冲区。Buffered wrapper 会摊薄重复小操作成本，
同时保持同样的公开 codec 语义。

持久化格式优先使用 `write_utf8_string_uleb_u64` 这类固定宽度长度字段，不要使用
target-width 的 `usize` 字符串长度 helper。

## 测试与代码覆盖率

本项目通过 `tests/` 下的集成测试覆盖 binary stream 行为，并在 `benches/` 下保留 benchmark。

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行 stream benchmark
cargo bench --bench stream

# 运行覆盖率报告
./coverage.sh

# 生成文本格式报告
./coverage.sh text

# 对齐 CI 要求
./align-ci.sh

# 运行 CI 检查（格式化、clippy、测试、覆盖率、安全审计）
RS_CI_SKIP_TOOLCHAIN_UPDATE=1 ./ci-check.sh
```

## 依赖项

运行时依赖保持很少：

- `qubit-codec-binary` 提供缓冲区级 binary codec。
- `qubit-io` 提供通用 stream helper 与 extension primitive。

开发依赖包含用于 benchmark 的 `criterion`。

## 许可证

Copyright (c) 2026. Haixing Hu.

根据 Apache 许可证 2.0 版（"许可证"）授权；
除非遵守许可证，否则您不得使用此文件。
您可以在以下位置获取许可证副本：

    http://www.apache.org/licenses/LICENSE-2.0

除非适用法律要求或书面同意，否则根据许可证分发的软件
按"原样"分发，不附带任何明示或暗示的担保或条件。
有关许可证下的特定语言管理权限和限制，请参阅许可证。

完整的许可证文本请参阅 [LICENSE](LICENSE)。

## 贡献

欢迎贡献！请随时提交 Pull Request。

### 开发指南

- 保持 stream adapter 与缓冲区级 codec 分离。
- 覆盖错误路径、partial read 和 partial write 行为。
- 保持 benchmark 能代表 stream adapter 开销。
- 提交 PR 前确保所有检查通过。

## 作者

**胡海星**

## 相关项目

- [qubit-codec-binary](https://github.com/qubit-ltd/rs-codec-binary)：缓冲区级 binary codec。
- [qubit-io](https://github.com/qubit-ltd/rs-io)：通用 `std::io` helper。
- Qubit 旗下的更多 Rust 库发布在 GitHub 组织
  [qubit-ltd](https://github.com/qubit-ltd)。

---

仓库地址：[https://github.com/qubit-ltd/rs-io-binary](https://github.com/qubit-ltd/rs-io-binary)
