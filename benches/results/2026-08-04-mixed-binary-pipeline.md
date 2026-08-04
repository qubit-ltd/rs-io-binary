# Mixed binary pipeline benchmark — 2026-08-04

Environment: Linux 6.17.0-35-generic, Intel Core i5-9600K (6 CPUs), Rust
1.94.0. The benchmark used the deterministic mixed-field fixture from
`benches/stream.rs` with 131,072 fields per iteration.

Command:

```shell
QUBIT_IO_STREAM_BENCH_GROUP=prod_mixed_binary_pipeline \
  cargo bench --bench stream -- --noplot
```

The table reports Criterion median estimates from this run. It is a local
comparison of buffering strategies, not a portable performance guarantee.

| Scenario | Time | Throughput |
| --- | ---: | ---: |
| Write: raw extension | 65.186 ms | 2.0107 M fields/s |
| Write: extension + `BufWriter` | 4.0408 ms | 32.438 M fields/s |
| Write: `BufferedBinaryWriter` | 3.7031 ms | 35.395 M fields/s |
| Read: raw extension | 46.802 ms | 2.8006 M fields/s |
| Read: extension + `BufReader` | 3.6401 ms | 36.008 M fields/s |
| Read: `BufferedBinaryReader` | 3.4581 ms | 37.903 M fields/s |
