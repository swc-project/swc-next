# Benchmark

## Rust parse

### typescript.js

```text
Benchmark 1: swc-next@952cba094a75fa3a0a7bbe1bc763c813eb75d663 (200 runs)
                mean     ±       σ          min     …     max
  Wall Time     21.8 ms  ±     0.2 ms      21.3 ms  …    22.5 ms
  Memory        37.8 MiB ±     0.1 MiB     37.6 MiB …    38.0 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (200 runs)
                mean     ±       σ          min     …     max
  Wall Time     94.0 ms  ±     1.2 ms      92.7 ms  …   102.6 ms          +332.0%  (4.3x slower)
  Memory        97.8 MiB ±     0.1 MiB     97.6 MiB …    98.0 MiB         +158.8%  (2.6x more)
```

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@952cba094a75fa3a0a7bbe1bc763c813eb75d663 (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      1.0 ms  ±     0.0 ms       1.0 ms  …     1.2 ms
  Memory         4.1 MiB ±     0.1 MiB      3.9 MiB …     4.3 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      2.1 ms  ±     0.1 ms       2.0 ms  …     2.4 ms          +101.0%  (2.0x slower)
  Memory         6.2 MiB ±     0.1 MiB      6.0 MiB …     6.4 MiB          +51.3%  (1.5x more)
```

## Isolated declaration

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@952cba094a75fa3a0a7bbe1bc763c813eb75d663 (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      1.7 ms  ±     0.1 ms       1.6 ms  …     2.0 ms
  Memory         5.3 MiB ±     0.1 MiB      5.0 MiB …     5.4 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      2.7 ms  ±     0.1 ms       2.5 ms  …     2.9 ms           +57.5%  (1.6x slower)
  Memory         6.9 MiB ±     0.1 MiB      6.6 MiB …     7.1 MiB          +30.0%
```

## Type strip

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@952cba094a75fa3a0a7bbe1bc763c813eb75d663 (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      1.2 ms  ±     0.1 ms       1.1 ms  …     1.9 ms
  Memory         7.6 MiB ±     0.1 MiB      7.4 MiB …     7.7 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      2.4 ms  ±     0.1 ms       2.3 ms  …     2.5 ms           +94.2%  (1.9x slower)
  Memory         7.0 MiB ±     0.1 MiB      6.7 MiB …     7.2 MiB           -8.3%
```

## JS parser

### typescript.js

```text
Benchmark 1: @swc-next/core@0.2.4 (952cba094a75fa3a0a7bbe1bc763c813eb75d663); 23 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time   3482.9 ms  ±    52.4 ms    3445.8 ms  …  3520.0 ms
  Memory      1292.8 MiB ±    16.4 MiB   1281.2 MiB …  1304.4 MiB

Benchmark 2: @swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1); 23 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time  19291.5 ms  ±    46.8 ms   19258.4 ms  … 19324.6 ms          +453.9%  (5.5x slower)
  Memory      2600.5 MiB ±     0.5 MiB   2600.2 MiB …  2600.9 MiB         +101.2%  (2.0x more)
```

### effect-schema-ast.ts

```text
Benchmark 1: @swc-next/core@0.2.4 (952cba094a75fa3a0a7bbe1bc763c813eb75d663); 688 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time   2813.5 ms  ±    11.8 ms    2805.2 ms  …  2821.8 ms
  Memory       631.3 MiB ±     2.8 MiB    629.4 MiB …   633.3 MiB

Benchmark 2: @swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1); 688 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time   5979.4 ms  ±     0.6 ms    5979.0 ms  …  5979.9 ms          +112.5%  (2.1x slower)
  Memory       101.9 MiB ±     0.0 MiB    101.9 MiB …   101.9 MiB          -83.9%  (6.2x less)
```
