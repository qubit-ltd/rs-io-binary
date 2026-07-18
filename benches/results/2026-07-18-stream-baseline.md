# Stream baseline — 2026-07-18

Environment: Linux 6.17, Intel Core i5-9600K (6 CPUs), Rust 1.94.0.
Each production group ran in its own process with 12 samples, a 2-second
warm-up, and an 8-second requested measurement window. Criterion lengthened
several windows because one fixed-width sample processes about 1.3 GiB.

## Fixed-width binary pipeline

| Implementation | Write median | Read median |
| --- | ---: | ---: |
| extension trait | 1.856 s | 292.027 ms |
| standard library native | 1.912 s | 349.505 ms |
| wrapper | 2.145 s | 318.126 ms |
| internally buffered wrapper | 2.179 s | 1.689 s |

The extension-trait path remains the fastest fixed-width baseline. The wrapper
adds about 8.9% on reads and 15.6% on writes. The internally buffered fixed-width
reader is 5.8× slower than the extension-trait reader and remains the clearest
performance concern in this suite.

## Unsigned LEB128

| Implementation | Write median | Read median |
| --- | ---: | ---: |
| extension trait | 293.562 ms | 234.425 ms |
| standard library manual | 325.866 ms | 327.539 ms |
| wrapper | 282.789 ms | 229.817 ms |
| internally buffered wrapper | 353.856 ms | 396.825 ms |

## Signed ZigZag

| Implementation | Write median | Read median |
| --- | ---: | ---: |
| extension trait | 291.644 ms | 243.496 ms |
| standard library manual | 317.499 ms | 326.288 ms |
| wrapper | 292.475 ms | 240.756 ms |
| internally buffered wrapper | 365.928 ms | 410.822 ms |

For both varint families, extension-trait and wrapper results are effectively
the leading pair. The internally buffered variants are consistently slower and
do not justify choosing them for throughput alone.

Command:

```shell
benches/run_stream_bench_groups.sh -- --noplot
```
