# In-memory mixed binary pipeline correction — 2026-10-24

Measured: 2026-10-24 14:19 CST

Host: Linux 7.0.0-38-generic, x86_64, Intel Core i5-9600K (6 CPUs)

Toolchain: `rustc 1.94.0 (4a4ef493e 2026-03-02)`, Cargo 1.94.0
Command: `QUBIT_IO_STREAM_BENCH_GROUP=memory_mixed_binary_pipeline cargo bench --locked --bench stream -- --noplot`

Each iteration processes 131,072 mixed fields from the same in-memory fixture.
Criterion reports the 95% confidence interval for time and throughput. The four
read cases use identical input bytes, schema, length limits, and summary checks;
the extension cases read through `Cursor<&[u8]>`, while the buffered cases use
`BufferedBinaryReader<Cursor<&[u8]>, LittleEndian>`. The reuse variants reuse
the payload `Vec<u8>`. Criterion used its default 3-second warm-up, 100 samples,
and 5-second measurement target; the first write case estimated 8.85 seconds and
reported that 100 samples could not fit in its original 5-second target.

| Case | Time (95% CI) | Throughput (95% CI) |
| --- | ---: | ---: |
| `extension_read` | 2.8734–2.9785 ms | 44.006–45.616 M fields/s |
| `extension_read_reuse` | 2.3790–2.3843 ms | 54.973–55.094 M fields/s |
| `buffered_read` | 3.3693–3.3759 ms | 38.826–38.902 M fields/s |
| `buffered_read_reuse` | 2.9907–3.0469 ms | 43.018–43.827 M fields/s |

The write cases are included by the same command and are recorded here for
completeness:

| Case | Time (95% CI) | Throughput (95% CI) |
| --- | ---: | ---: |
| `extension_write` | 1.2048–1.3065 ms | 100.32–108.79 M fields/s |
| `buffered_write` | 2.0072–2.0139 ms | 65.083–65.302 M fields/s |

These measurements correct the historical report's input-construction error.
They are a single-host result and do not establish a general performance ranking.
