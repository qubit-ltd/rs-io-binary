# Typed buffered string-buffer reuse comparison — 2026-10-30

Measured: 2026-10-30 CST (UTC+08:00), in two serial Criterion processes. The
environment timestamp was captured at 09:25:59 CST before the first command.

Host: Linux `7.0.0-38-generic`, x86_64, Intel Core i5-9600K CPU @ 3.70GHz
(6 CPUs).

Toolchain: `rustc 1.94.0 (4a4ef493e 2026-03-02)`, Cargo 1.94.0
(`85eff7c80 2026-01-15`).

Source: HEAD `02b277a05767ef877cd946f1d5f193d261ef2ee9`; the measured, uncommitted
`benches/stream.rs` diff had SHA-256
`d93d42764e0a088091052ef978deaa5bef9f96a101ce5747777d88d456523471`
(`56 insertions, 3 deletions`). HEAD alone does not contain the typed reuse
change.

Commands, run serially in separate processes with Criterion defaults:

```shell
QUBIT_IO_STREAM_BENCH_GROUP=memory_mixed_binary_pipeline cargo bench --locked --bench stream -- read --noplot
QUBIT_IO_STREAM_BENCH_GROUP=prod_mixed_binary_pipeline cargo bench --locked --bench stream -- read --noplot
```

Criterion used the default 3-second warm-up, 100 samples, and 5-second target.
The file group's `ext_raw_read` warned that 100 samples would require an estimated
25.7 seconds instead of the 5-second target. All read cases completed; setup
performed per-field value, summary, and EOF checks before timing. Criterion
reported outliers in several cases; see its output files under
`target/criterion/{memory_mixed_binary_pipeline,prod_mixed_binary_pipeline}/<case>/new/`
(`estimates.json`, `sample.json`, and `benchmark.json`).

Both groups process the same deterministic schema of 131,072 mixed fields, using
seed `0x5EED_CAFE_1234_5678`, little-endian encoding, and a 1,024-byte string
payload limit. The buffered default and reuse cases now both use inherent typed
`u8`, `i32`, `u64`, and `u16` length-prefix reads. The reuse case keeps the
existing `read_utf8_payload_into` payload method. Extension cases and their
generic helper are unchanged. These results compare the overall payload API and
storage strategies; they do not measure allocator cost in isolation.

The Criterion output presents a lower bound, central estimate, and upper bound
for time and throughput. The relative differences below use only the central
time estimates from the same run.

## In-memory group

| Case | Time (95% CI) | Throughput (95% CI) |
| --- | ---: | ---: |
| `extension_read` | 16.519–18.353 ms (17.431 ms) | 7.1419–7.9348 M fields/s (7.5193) |
| `extension_read_reuse` | 7.5489–9.2705 ms (8.3869 ms) | 14.139–17.363 M fields/s (15.628) |
| `buffered_read` | 18.913–22.397 ms (20.602 ms) | 5.8523–6.9302 M fields/s (6.3621) |
| `buffered_read_reuse` | 21.632–27.019 ms (24.231 ms) | 4.8511–6.0593 M fields/s (5.4094) |

The typed buffered reuse central estimate was 17.6% slower than the typed
buffered default. Their time intervals overlap slightly, so this run does not
establish a clear performance separation.

## File-backed production group

| Case | Time (95% CI) | Throughput (95% CI) |
| --- | ---: | ---: |
| `ext_raw_read` | 264.07–315.12 ms (289.38 ms) | 415.95–496.35 K fields/s (452.94) |
| `ext_bufreader_read` | 20.002–24.260 ms (22.111 ms) | 5.4027–6.5530 M fields/s (5.9280) |
| `ext_bufreader_read_reuse` | 14.480–17.614 ms (16.033 ms) | 7.4413–9.0518 M fields/s (8.1752) |
| `buffered_read` | 17.574–21.176 ms (19.376 ms) | 6.1897–7.4584 M fields/s (6.7647) |
| `buffered_read_reuse` | 17.631–21.064 ms (19.333 ms) | 6.2226–7.4342 M fields/s (6.7798) |

The typed buffered reuse central estimate was 0.2% faster than the typed
buffered default, a negligible difference relative to the overlapping intervals.
This file-backed comparison includes file-open/close cost and can be affected by
OS page cache state. It does not represent durable physical-disk throughput.

The historical same-name cases used different scalar and prefix dispatch and
must not be treated as a direct version-to-version performance baseline. The
original numbers remain available in the [2026-08-07 report](2026-08-07-string-buffer-reuse.md)
and the [2026-10-24 correction report](2026-10-24-memory-mixed-binary-pipeline-correction.md).
