# stream 基准说明（生产场景）

本文档用于约束 `benches/stream.rs` 的基准口径，避免不同版本间口径漂移导致误判。

- 基准只覆盖二进制整数路径：
  - `micro_binary_pipeline`：受控短读短写内存流上的固定宽度读写，用于隔离
    adapter、临时缓冲和内部 buffer 的成本。
  - `memory_mixed_binary_pipeline`：不经过文件系统的混合固定宽度整数和 UTF-8
    字符串记录流，用于隔离 adapter、缓冲和 payload 分配成本。
  - `prod_binary_pipeline`：固定字段的二进制读写。
  - `prod_mixed_binary_pipeline`：混合固定宽度整数和 UTF-8 字符串的记录流。
    该组同时测量每条记录分配 `String` 的默认读取路径，以及复用
    `Vec<u8>` 的 `read_utf8_payload_into` 路径。
  - `prod_varints`：随机类型字段流的无符号 LEB128 编解码。
  - `prod_signed_varints`：随机类型字段流的 ZigZag 编解码。
  - `async_binary_pipeline`：运行时无关的异步 `u64` 读写，以及与
    `qubit-io` 异步 buffer 组合后的对比。
- 七个 benchmark group 的输入/输出介质与采样设置如下。Criterion 默认值为
  预热 3s、测量目标 5s、100 samples；命令行参数可覆盖这些值。测量目标是
  Criterion 的采样测量时间，不代表整个 benchmark 进程的总运行时间。

| Group | 输入/输出介质 | 采样设置 |
| --- | --- | --- |
| `micro_binary_pipeline` | 合成短读短写内存流 | Criterion 默认 |
| `memory_mixed_binary_pipeline` | `Cursor` 内存流 | Criterion 默认 |
| `prod_binary_pipeline` | 临时文件；比较原始文件、标准库缓冲及 crate buffered 路径 | 预热 2s / 测量目标 8s / 12 samples |
| `prod_mixed_binary_pipeline` | 临时文件；比较原始文件、标准库缓冲及 crate buffered 路径 | Criterion 默认 |
| `prod_varints` | 临时文件；比较 extension、手写标准库、wrapper 与 buffered 路径 | 预热 2s / 测量目标 8s / 12 samples |
| `prod_signed_varints` | 临时文件；比较 extension、手写标准库、wrapper 与 buffered 路径 | 预热 2s / 测量目标 8s / 12 samples |
| `async_binary_pipeline` | 确定性 ready 异步内存流 | Criterion 默认 |
- 已移除 UTF-8 文本读写基准。
- `micro_binary_pipeline` 使用 32,768 个 `u64` 值、每次最多读写 7 字节的
  合成内存流；它不使用 `Cursor`，并以 16 字节内部缓冲强制覆盖 buffer 边界。
- `memory_mixed_binary_pipeline` 使用与 `prod_mixed_binary_pipeline` 相同的
  131,072 条混合记录，但使用 `Cursor` 内存流，不把文件打开、关闭和 page cache
  成本混入 adapter 对比。读取包含四种对照：extension trait 与
  `BufferedBinaryReader` 各自分别使用新建 `String` 或复用 payload `Vec<u8>`；
  `extension_read_reuse` 和 `buffered_read_reuse` 使用相同字段 schema 与输入字节，
  计时前检查四种读取摘要一致。
- 生产型输入规模采用大批量重复：
  - 单批记录数：`BINARY_BATCH = 1_048_576`
  - 单批 varint 字段数：`VARINT_COUNT = 262_144`
  - 每次 benchmark iteration 内重复次数：`BINARY_REPEAT = 32`、`VARINT_REPEAT = 64`
  - `BINARY_REPEAT` 与 `VARINT_REPEAT` 低于纯内存场景，因为相应生产型固定字段与 varint 基准使用真实文件 I/O；二进制固定字段单次样本仍会处理约 1.3 GiB 数据。
- 数据分布采用近似正态分布采样（基于固定 seed 的 Box-Muller），以贴近真实业务里“高峰聚集、少量极端值”的场景。
- `prod_varints` 与 `prod_signed_varints` 的字段类型也是随机分布：
  - 无符号 LEB128 在 `u8/u16/u32/u64/usize/u128` 中随机选择字段类型，并按类型生成随机值。
  - ZigZag 在 `i8/i16/i32/i64/isize/i128` 中随机选择字段类型，并按类型生成随机值。
  - 读 benchmark 使用同一字段 schema 流 dispatch 对应读取方法，避免连续读取单一类型造成过度理想化。
- `prod_binary_pipeline` 使用真实文件系统文件作为输入输出：
  - 写入基准使用 `File::create` + `BufWriter<File>`，每轮覆盖写同一个临时文件。
  - 读取基准先由 writer 生成临时源文件，再使用 `File::open` + `BufReader<File>` 读取。
  - `std_native_*` 使用标准库原生 fixed-width 路径：`write_all()` + `to_le_bytes()`，以及 `read_exact()` + `from_le_bytes()`。该组只适用于固定宽度二进制字段。
  - 基准不调用 `sync_all()`，因此结果仍可能受 OS page cache 影响；目标是排除 `Cursor<&[u8]>` / `Cursor<Vec<u8>>` 的内存特化，而不是测物理磁盘落盘延迟。
  - 临时文件目录创建在系统临时目录下，benchmark 结束时尽力清理。
- `prod_varints` 与 `prod_signed_varints` 同样使用真实文件系统文件：
  - `ext_*` 使用 `File::create/open` + `BufWriter<File>` / `BufReader<File>`，再调用对应 extension trait。
  - `std_manual_*` 使用 `BufWriter<File>` / `BufReader<File>`，并在 benchmark 内部手写安全的 LEB128 / ZigZag 编解码，不调用本 crate 的 codec，也不使用 unchecked slice 优化。
  - `wrapper_*` 使用 `BinaryReader/Writer`、`Leb128Reader/Writer`、`ZigZagReader/Writer` 包装 `BufReader<File>` / `BufWriter<File>`。
- `buffered_*` 使用 `BufferedBinaryReader/Writer`、`BufferedLeb128Reader/Writer`、
  `BufferedZigZagReader/Writer` 包装文件；这些
  wrapper 通过 `TranscodeDecodeInput` / `TranscodeEncodeOutput` 适配器委托
  codec 生命周期，并由适配器管理缓冲区。benchmark 调用 wrapper 的公开读写
  方法，不直接调用 codec unsafe 方法。
- `prod_mixed_binary_pipeline` 也使用临时文件：extension 路径分别比较原始
  `File` 与标准库 `BufReader` / `BufWriter`，crate buffered 路径使用
  `BufferedBinaryReader/Writer` 包装文件。其余内存型与异步组不使用文件 I/O。
- `async_binary_pipeline` 使用确定性的 ready `AsyncInput`/`AsyncOutput`，每次
  最多传输 3 个字节，比较异步 extension trait 与 `qubit-io` 异步 buffer 的组合。
- `memory_mixed_binary_pipeline` 的 2026-10-24 修正结果及其输入构造说明见
  [修正报告](results/2026-10-24-memory-mixed-binary-pipeline-correction.md)。该报告
  保留历史测量数值；本说明修订不重新测量或替换这些结果。
- 每个大组必须在独立的 `cargo bench` 进程中执行，避免前一个大组的长时间
  文件 IO、分支预测和 CPU 调度状态污染后续大组结果。`benches/stream.rs`
  会读取 `QUBIT_IO_STREAM_BENCH_GROUP`，每次只注册并执行一个大组。
  直接运行 `cargo bench --bench stream` 不再作为完整套件入口。

## 基线约定

当前基线口径是同一次 benchmark run 内的 `Read` / `Write` extension trait 实现：

- `ext_*`：使用 `BinaryReadExt` / `BinaryWriteExt`、`Leb128ReadExt` / `Leb128WriteExt`、`ZigZagReadExt` / `ZigZagWriteExt`。
- `std_native_*`：仅在 `prod_binary_pipeline` 中出现，使用标准库 `BufReader<File>` / `BufWriter<File>` 加 `read_exact()` / `write_all()` 和基础类型字节序转换。
- `std_manual_*`：仅在 `prod_varints` 与 `prod_signed_varints` 中出现，使用标准库 `BufReader<File>` / `BufWriter<File>` 加手写安全 LEB128 / ZigZag 协议实现。
- `wrapper_*`：使用 `BinaryReader` / `BinaryWriter`、`Leb128Reader` / `Leb128Writer`、`ZigZagReader` / `ZigZagWriter`；reader 显式选择 `Strict` 或 `NonStrict`，宽松路径调用带 `_non_strict` 后缀的方法。
- `buffered_*`：使用 `Buffered*Reader/Writer` wrapper，并通过
  `TranscodeDecodeInput` / `TranscodeEncodeOutput` 委托 codec 生命周期和缓冲管理；
  benchmark 源码调用 wrapper 的公开方法。
- `*_read_reuse`：读取混合记录时复用调用方的 UTF-8 payload `Vec<u8>`；这些结果与默认
  `*_read` 路径分开比较，避免把分配策略变化误认为缓冲区变化。
  在内存组中，`extension_read_reuse` 使用 `Cursor<&[u8]>`，
  `buffered_read_reuse` 使用 `BufferedBinaryReader<Cursor<&[u8]>, LittleEndian>`；
  两种输入构造不会混淆。

结果解读时应比较同一 group 下相同方向的 `ext_*` 与 `wrapper_*`，例如：

- `prod_varints/ext_leb128_read_mixed_batch`
- `prod_varints/std_manual_leb128_read_mixed_batch`
- `prod_varints/buffered_leb128_read_mixed_batch`

不再把“上一次提交版本”作为主要性能基线。提交间 baseline 只适合判断同一实现随代码演进是否漂移，不适合评估 wrapper 相对 extension trait 的收益。

示例流程（在本仓库根目录执行）：

```bash
benches/run_stream_bench_groups.sh
```

如只运行单个大组：

```bash
QUBIT_IO_STREAM_BENCH_GROUP=prod_varints cargo bench --bench stream
```
