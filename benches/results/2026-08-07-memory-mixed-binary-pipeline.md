# In-memory mixed binary pipeline — 2026-08-07

Environment: Linux 6.17.0-35-generic, Intel Core i5-9600K (6 CPUs), Rust
1.94.0. The deterministic fixture in `benches/stream.rs` contains 131,072
mixed fields per iteration.

Command:

```shell
QUBIT_IO_STREAM_BENCH_GROUP=memory_mixed_binary_pipeline \
  cargo +1.94.0 bench --bench stream -- --noplot
```

The table reports Criterion median estimates from this run. It measures the
in-memory adapter pipeline and excludes filesystem open, close, and cache
effects. These values are local comparisons, not portable performance
guarantees.

| Scenario | Time | Throughput |
| --- | ---: | ---: |
| Write: extension | 1.0160 ms | 129.00 M fields/s |
| Write: buffered wrapper | 1.6214 ms | 80.837 M fields/s |
| Read: extension | 2.8479 ms | 46.024 M fields/s |
| Read: buffered wrapper | 3.4039 ms | 38.506 M fields/s |
| Read: buffered wrapper + reusable payload | 2.2380 ms | 58.567 M fields/s |

The benchmark isolates adapter and allocation costs from the file-backed
`prod_*` groups. The reusable payload buffer improved the buffered read in
this run, while the buffered wrapper added overhead for this in-memory
workload.
