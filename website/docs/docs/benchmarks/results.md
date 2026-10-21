# Benchmark

## Rust parse

### typescript.js

```text
Benchmark 1: swc-next@d5047c8c18023566b844a81e6bf5020b86924ea2; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time    157.2 ms  ±     4.5 ms     149.7 ms  …   191.5 ms
  Memory        34.6 MiB ±     0.0 MiB     34.6 MiB …    34.7 MiB

Benchmark 2: swc_core@82.1.0; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time    547.5 ms  ±    11.5 ms     526.0 ms  …   638.5 ms          +248.3%  (3.5x slower)
  Memory        89.8 MiB ±     0.1 MiB     89.7 MiB …    90.8 MiB         +159.3%  (2.6x more)
```

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@d5047c8c18023566b844a81e6bf5020b86924ea2; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      4.3 ms  ±     0.1 ms       4.1 ms  …     4.7 ms
  Memory         3.3 MiB ±     0.0 MiB      3.3 MiB …     3.3 MiB

Benchmark 2: swc_core@82.1.0; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time     10.9 ms  ±     0.2 ms      10.5 ms  …    11.8 ms          +155.7%  (2.6x slower)
  Memory         5.5 MiB ±     0.0 MiB      5.5 MiB …     5.6 MiB          +65.1%  (1.7x more)
```

## Isolated declaration

### hono-types.ts

```text
Benchmark 1: swc-next@d5047c8c18023566b844a81e6bf5020b86924ea2; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      8.1 ms  ±     0.1 ms       7.8 ms  …     8.5 ms
  Memory         4.9 MiB ±     0.0 MiB      4.9 MiB …     4.9 MiB

Benchmark 2: swc_core@82.1.0; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time     16.2 ms  ±     0.3 ms      15.7 ms  …    18.5 ms           +99.6%  (2.0x slower)
  Memory         6.5 MiB ±     0.0 MiB      6.4 MiB …     6.6 MiB          +33.0%  (1.3x more)
```

## Type strip

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@d5047c8c18023566b844a81e6bf5020b86924ea2; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      5.1 ms  ±     0.1 ms       4.9 ms  …     5.5 ms
  Memory         3.8 MiB ±     0.0 MiB      3.8 MiB …     3.8 MiB

Benchmark 2: swc_ts_fast_strip@59.0.1; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time     12.2 ms  ±     0.2 ms      11.8 ms  …    13.1 ms          +139.1%  (2.4x slower)
  Memory         6.2 MiB ±     0.1 MiB      6.1 MiB …     6.3 MiB          +64.6%  (1.6x more)
```

## JS parser

### typescript.js

```text
Benchmark 1: @swc-next/core@d5047c8c18023566b844a81e6bf5020b86924ea2; 19 parses + 1 validation + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time   1733.8 ms  ±     2.8 ms    1731.8 ms  …  1735.8 ms
  Memory      1114.1 MiB ±     8.1 MiB   1108.4 MiB …  1119.9 MiB

Benchmark 2: @swc/core@1.15.46; 19 parses + 1 validation + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time  16375.2 ms  ±   173.7 ms   16252.4 ms  … 16498.0 ms          +844.5%  (9.4x slower)
  Memory      2614.4 MiB ±     0.5 MiB   2614.1 MiB …  2614.7 MiB         +134.7%  (2.3x more)
```

### effect-schema-ast.ts

```text
Benchmark 1: @swc-next/core@d5047c8c18023566b844a81e6bf5020b86924ea2; 1377 parses + 1 validation + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time   1673.7 ms  ±   155.7 ms    1563.6 ms  …  1783.8 ms
  Memory       886.3 MiB ±    13.1 MiB    877.0 MiB …   895.5 MiB

Benchmark 2: @swc/core@1.15.46; 1377 parses + 1 validation + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time  13331.2 ms  ±    71.9 ms   13280.3 ms  … 13382.1 ms          +696.5%  (8.0x slower)
  Memory        85.0 MiB ±     0.4 MiB     84.7 MiB …    85.2 MiB          -90.4%  (10.4x less)
```
