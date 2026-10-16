# Benchmark

## Rust parse

### typescript.js

```text
Benchmark 1: swc-next@a4c7f78959abefca0510a3def442dec07e129635; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time    193.1 ms  ±     1.4 ms     191.0 ms  …   204.0 ms
  Memory        37.8 MiB ±     0.1 MiB     37.6 MiB …    38.1 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time    711.1 ms  ±     4.5 ms     704.5 ms  …   732.6 ms          +268.3%  (3.7x slower)
  Memory        98.3 MiB ±     0.1 MiB     98.1 MiB …    98.4 MiB         +159.9%  (2.6x more)
```

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@a4c7f78959abefca0510a3def442dec07e129635; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      4.7 ms  ±     0.1 ms       4.5 ms  …     4.9 ms
  Memory         4.1 MiB ±     0.1 MiB      4.0 MiB …     4.3 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time     12.5 ms  ±     0.1 ms      12.2 ms  …    13.2 ms          +166.3%  (2.7x slower)
  Memory         6.2 MiB ±     0.1 MiB      6.0 MiB …     6.4 MiB          +48.7%  (1.5x more)
```

## Isolated declaration

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@a4c7f78959abefca0510a3def442dec07e129635; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      9.2 ms  ±     0.1 ms       9.0 ms  …    10.0 ms
  Memory         5.3 MiB ±     0.1 MiB      5.2 MiB …     5.5 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time     17.6 ms  ±     0.1 ms      17.4 ms  …    18.4 ms           +92.3%  (1.9x slower)
  Memory         7.0 MiB ±     0.1 MiB      6.7 MiB …     7.2 MiB          +30.6%  (1.3x more)
```

## Type strip

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@a4c7f78959abefca0510a3def442dec07e129635; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      5.9 ms  ±     0.1 ms       5.7 ms  …     6.3 ms
  Memory         7.6 MiB ±     0.1 MiB      7.4 MiB …     7.8 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da; 10 iterations (200 runs)
                mean     ±       σ          min     …     max
  Wall Time     14.1 ms  ±     0.1 ms      13.8 ms  …    14.6 ms          +137.6%  (2.4x slower)
  Memory         7.3 MiB ±     0.1 MiB      7.0 MiB …     7.6 MiB           -4.0%
```

## JS parser

### typescript.js

```text
Benchmark 1: @swc-next/core@0.2.4 (a4c7f78959abefca0510a3def442dec07e129635); 24 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time   3773.4 ms  ±   596.0 ms    3351.9 ms  …  4194.8 ms
  Memory      1263.5 MiB ±    38.8 MiB   1236.0 MiB …  1290.9 MiB

Benchmark 2: @swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1); 24 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time  19873.7 ms  ±    51.3 ms   19837.4 ms  … 19910.0 ms          +426.7%  (5.3x slower)
  Memory      2600.4 MiB ±     0.7 MiB   2599.9 MiB …  2600.9 MiB         +105.8%  (2.1x more)
```

### effect-schema-ast.ts

```text
Benchmark 1: @swc-next/core@0.2.4 (a4c7f78959abefca0510a3def442dec07e129635); 743 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time   2925.7 ms  ±    35.3 ms    2900.8 ms  …  2950.6 ms
  Memory       651.5 MiB ±     1.6 MiB    650.4 MiB …   652.7 MiB

Benchmark 2: @swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1); 743 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time   6396.3 ms  ±    11.2 ms    6388.4 ms  …  6404.2 ms          +118.6%  (2.2x slower)
  Memory       101.5 MiB ±     0.1 MiB    101.5 MiB …   101.6 MiB          -84.4%  (6.4x less)
```
