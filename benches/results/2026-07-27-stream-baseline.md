# Stream baseline observation — 2026-07-27

Environment: Linux 6.17, Intel Core i5-9600K (6 CPUs), Rust 1.94.0.
Each production group ran in its own process with 12 samples, a 2-second
warm-up, and an 8-second requested measurement window. Criterion lengthened
several windows because one fixed-width sample processes about 1.3 GiB.

The host was under substantial concurrent load. Immediately after the run,
the load average was `4.14, 9.67, 9.14` with eight logged-in users. Confidence
intervals were correspondingly wide, especially for fixed-width I/O. Treat
these numbers as a contention-affected observation, not as a replacement for
the stable 2026-07-18 baseline and not as evidence of a code regression.

## Fixed-width binary pipeline

| Implementation | Write median | Read median |
| --- | ---: | ---: |
| extension trait | 9.5766 s | 5.0026 s |
| standard library native | 7.0316 s | 707.70 ms |
| wrapper | 6.8122 s | 3.0029 s |
| internally buffered wrapper | 7.9465 s | 1.7388 s |

The within-run ordering differs sharply from the 2026-07-18 baseline: the
standard-library read path remained much less affected than the extension and
wrapper paths. That divergence, together with the host load, makes a
cross-run implementation comparison invalid.

## Unsigned LEB128

| Implementation | Write median | Read median |
| --- | ---: | ---: |
| extension trait | 837.44 ms | 759.64 ms |
| standard library manual | 547.56 ms | 418.72 ms |
| wrapper | 555.68 ms | 617.12 ms |
| internally buffered wrapper | 999.02 ms | 462.64 ms |

## Signed ZigZag

| Implementation | Write median | Read median |
| --- | ---: | ---: |
| extension trait | 607.67 ms | 605.74 ms |
| standard library manual | 474.66 ms | 391.28 ms |
| wrapper | 669.32 ms | 552.11 ms |
| internally buffered wrapper | 621.01 ms | 366.60 ms |

The internally buffered readers ranked ahead of the extension paths in this
run, the opposite of the earlier stable result. A later rerun on an otherwise
idle host is required before drawing a performance conclusion.

Command:

```shell
benches/run_stream_bench_groups.sh
```
