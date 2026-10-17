# Benchmark

## Rust parse

### typescript.js

```text
Benchmark 1: swc-next@e06ead61165c3a6de44205764ef0da2973c34189; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time    193.8 ms  ±     2.0 ms     191.4 ms  …   209.0 ms
  Memory        37.8 MiB ±     0.1 MiB     37.6 MiB …    38.1 MiB

Benchmark 2: swc_core@82.1.0; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time    720.5 ms  ±    11.2 ms     706.7 ms  …   780.6 ms          +271.8%  (3.7x slower)
  Memory        98.9 MiB ±     0.1 MiB     98.7 MiB …    99.1 MiB         +161.6%  (2.6x more)
```

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@e06ead61165c3a6de44205764ef0da2973c34189; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      4.7 ms  ±     0.1 ms       4.5 ms  …     5.8 ms
  Memory         4.1 MiB ±     0.1 MiB      3.9 MiB …     4.3 MiB

Benchmark 2: swc_core@82.1.0; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time     12.6 ms  ±     0.1 ms      12.3 ms  …    13.6 ms          +166.6%  (2.7x slower)
  Memory         7.0 MiB ±     0.1 MiB      6.8 MiB …     7.2 MiB          +68.3%  (1.7x more)
```

## Isolated declaration

### hono-types.ts

## Type strip

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@e06ead61165c3a6de44205764ef0da2973c34189; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      5.9 ms  ±     0.1 ms       5.7 ms  …     6.6 ms
  Memory         7.6 MiB ±     0.1 MiB      7.4 MiB …     7.8 MiB

Benchmark 2: swc_ts_fast_strip@59.0.1; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time     14.3 ms  ±     0.1 ms      14.1 ms  …    15.0 ms          +140.5%  (2.4x slower)
  Memory         7.7 MiB ±     0.1 MiB      7.4 MiB …     7.9 MiB           +0.9%
```

## JS parser

### typescript.js

```text
Benchmark 1: @swc-next/core@e06ead61165c3a6de44205764ef0da2973c34189; 23 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time   3269.9 ms  ±   121.6 ms    3183.9 ms  …  3355.9 ms
  Memory      1277.3 MiB ±     1.9 MiB   1275.9 MiB …  1278.6 MiB

Benchmark 2: @swc/core@1.15.46; 23 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time  19229.6 ms  ±   111.3 ms   19150.9 ms  … 19308.4 ms          +488.1%  (5.9x slower)
  Memory      2600.0 MiB ±     1.7 MiB   2598.8 MiB …  2601.1 MiB         +103.6%  (2.0x more)
```

### effect-schema-ast.ts

```text
Benchmark 1: @swc-next/core@e06ead61165c3a6de44205764ef0da2973c34189; 737 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time   2922.5 ms  ±   168.5 ms    2803.4 ms  …  3041.7 ms
  Memory       652.9 MiB ±     3.7 MiB    650.3 MiB …   655.6 MiB

Benchmark 2: @swc/core@1.15.46; 737 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time   6488.3 ms  ±     8.0 ms    6482.7 ms  …  6494.0 ms          +122.0%  (2.2x slower)
  Memory       101.5 MiB ±     0.0 MiB    101.5 MiB …   101.5 MiB          -84.4%  (6.4x less)
```
