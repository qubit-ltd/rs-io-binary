# Mixed binary pipeline string-buffer reuse — 2026-08-07

Environment: Linux 6.17.0-35-generic, Intel Core i5-9600K (6 CPUs), Rust
1.94.0. The deterministic fixture in `benches/stream.rs` contains 131,072
mixed fields per iteration.

Command:

```shell
QUBIT_IO_STREAM_BENCH_GROUP=prod_mixed_binary_pipeline \
  cargo +1.94.0 bench --bench stream -- --noplot
```

The table reports Criterion median estimates from this run. It is a local
comparison of buffering and payload-allocation strategies, not a portable
performance guarantee.

Methodology note: the buffered default case used inherent typed scalar and length-prefix reads, while the reusable-payload case used extension-trait reads through a generic helper. The observed 26% reduction compares both changes together and does not isolate payload-buffer reuse. The original measurements are retained as historical data. A subsequent typed-dispatch comparison is recorded in the [2026-10-30 report](2026-10-30-typed-buffered-string-buffer-reuse.md).

| Scenario | Time | Throughput |
| --- | ---: | ---: |
| Write: raw extension | 78.455 ms | 1.6707 M fields/s |
| Write: extension + `BufWriter` | 4.6676 ms | 28.082 M fields/s |
| Write: `BufferedBinaryWriter` | 6.3764 ms | 20.556 M fields/s |
| Read: raw extension | 45.964 ms | 2.8516 M fields/s |
| Read: extension + `BufReader` | 4.1831 ms | 31.334 M fields/s |
| Read: extension + `BufReader` + reusable payload | 3.6234 ms | 36.173 M fields/s |
| Read: `BufferedBinaryReader` | 3.9652 ms | 33.055 M fields/s |
| Read: `BufferedBinaryReader` + reusable payload | 2.9448 ms | 44.510 M fields/s |

Within this run, the combined change from the default path to the reusable-payload
path had an observed median read-time difference of about 13% with `BufReader`
and 26% with `BufferedBinaryReader`. These differences compare payload handling
and scalar/length-prefix dispatch together, so they cannot be attributed to
payload-buffer reuse alone. Filesystem cache state and CPU scheduling can
materially affect these local numbers.
