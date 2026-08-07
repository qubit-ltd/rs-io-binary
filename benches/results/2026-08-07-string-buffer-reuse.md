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

Within this run, reusing the payload buffer reduced the median read time by
about 13% with `BufReader` and 26% with `BufferedBinaryReader`. Filesystem
cache state and CPU scheduling can materially affect these local numbers.
