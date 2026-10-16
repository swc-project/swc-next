# Benchmark

## Rust parse

### typescript.js

```text
Benchmark 1: swc-next@9941e887213a783b08546a65ca609e97af7451e5 (50 runs)
                mean     ±       σ          min     …     max
  Wall Time     21.9 ms  ±     0.2 ms      21.4 ms  …    22.5 ms
  Memory        37.8 MiB ±     0.1 MiB     37.6 MiB …    37.9 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                mean     ±       σ          min     …     max
  Wall Time     93.5 ms  ±     0.8 ms      92.5 ms  …    97.2 ms          +327.6%  (4.3x slower)
  Memory        97.8 MiB ±     0.1 MiB     97.6 MiB …    98.0 MiB         +158.8%  (2.6x more)
```

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@9941e887213a783b08546a65ca609e97af7451e5 (50 runs)
                mean     ±       σ          min     …     max
  Wall Time      1.0 ms  ±     0.0 ms       1.0 ms  …     1.1 ms
  Memory         4.1 MiB ±     0.1 MiB      3.9 MiB …     4.3 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                mean     ±       σ          min     …     max
  Wall Time      2.0 ms  ±     0.0 ms       2.0 ms  …     2.1 ms           +96.3%  (2.0x slower)
  Memory         6.2 MiB ±     0.1 MiB      6.0 MiB …     6.4 MiB          +51.4%  (1.5x more)
```

## Isolated declaration

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@9941e887213a783b08546a65ca609e97af7451e5 (50 runs)
                mean     ±       σ          min     …     max
  Wall Time      1.6 ms  ±     0.0 ms       1.6 ms  …     1.8 ms
  Memory         5.3 MiB ±     0.1 MiB      5.1 MiB …     5.4 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                mean     ±       σ          min     …     max
  Wall Time      2.6 ms  ±     0.0 ms       2.5 ms  …     2.7 ms           +57.7%  (1.6x slower)
  Memory         6.9 MiB ±     0.1 MiB      6.6 MiB …     7.0 MiB          +30.9%  (1.3x more)
```

## Type strip

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@9941e887213a783b08546a65ca609e97af7451e5 (50 runs)
                mean     ±       σ          min     …     max
  Wall Time      1.2 ms  ±     0.0 ms       1.1 ms  …     1.3 ms
  Memory         7.6 MiB ±     0.1 MiB      7.4 MiB …     7.7 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                mean     ±       σ          min     …     max
  Wall Time      2.3 ms  ±     0.1 ms       2.3 ms  …     3.1 ms           +95.1%  (2.0x slower)
  Memory         6.9 MiB ±     0.1 MiB      6.8 MiB …     7.2 MiB           -8.4%
```

## JS parser

### typescript.js

```text
Benchmark 1: @swc-next/core@0.2.4 (9941e887213a783b08546a65ca609e97af7451e5); 24 parses + 5 warmups (3 runs)
                mean     ±       σ          min     …     max
  Wall Time   3253.5 ms  ±    45.4 ms    3225.6 ms  …  3305.8 ms
  Memory      1262.0 MiB ±    50.2 MiB   1222.1 MiB …  1318.3 MiB

Benchmark 2: @swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1); 24 parses + 5 warmups (3 runs)
                mean     ±       σ          min     …     max
  Wall Time  19921.2 ms  ±    51.5 ms   19862.1 ms  … 19956.6 ms          +512.3%  (6.1x slower)
  Memory      2601.3 MiB ±     0.9 MiB   2600.3 MiB …  2602.0 MiB         +106.1%  (2.1x more)

Benchmark 1: @swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1); 24 parses + 5 warmups (3 runs)
                mean     ±       σ          min     …     max
  Wall Time  19825.7 ms  ±    82.1 ms   19765.3 ms  … 19919.1 ms
  Memory      2601.1 MiB ±     0.3 MiB   2600.9 MiB …  2601.4 MiB

Benchmark 2: @swc-next/core@0.2.4 (9941e887213a783b08546a65ca609e97af7451e5); 24 parses + 5 warmups (3 runs)
                mean     ±       σ          min     …     max
  Wall Time   3561.0 ms  ±   362.8 ms    3198.8 ms  …  3924.4 ms           -82.0%  (5.6x faster)
  Memory      1245.0 MiB ±    99.6 MiB   1130.1 MiB …  1304.9 MiB          -52.1%  (2.1x less)
```

### effect-schema-ast.ts

```text
Benchmark 1: @swc-next/core@0.2.4 (9941e887213a783b08546a65ca609e97af7451e5); 712 parses + 5 warmups (3 runs)
                mean     ±       σ          min     …     max
  Wall Time   2845.8 ms  ±   149.8 ms    2734.0 ms  …  3016.0 ms
  Memory       632.8 MiB ±     2.3 MiB    630.8 MiB …   635.3 MiB

Benchmark 2: @swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1); 712 parses + 5 warmups (3 runs)
                mean     ±       σ          min     …     max
  Wall Time   6127.7 ms  ±    27.0 ms    6102.9 ms  …  6156.5 ms          +115.3%  (2.2x slower)
  Memory       101.8 MiB ±     0.0 MiB    101.7 MiB …   101.8 MiB          -83.9%  (6.2x less)

Benchmark 1: @swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1); 712 parses + 5 warmups (3 runs)
                mean     ±       σ          min     …     max
  Wall Time   6094.2 ms  ±    22.3 ms    6069.5 ms  …  6112.8 ms
  Memory       101.8 MiB ±     0.1 MiB    101.7 MiB …   101.9 MiB

Benchmark 2: @swc-next/core@0.2.4 (9941e887213a783b08546a65ca609e97af7451e5); 712 parses + 5 warmups (3 runs)
                mean     ±       σ          min     …     max
  Wall Time   2821.5 ms  ±   137.3 ms    2724.3 ms  …  2978.5 ms           -53.7%  (2.2x faster)
  Memory       636.8 MiB ±     2.9 MiB    634.1 MiB …   639.9 MiB         +525.5%  (6.3x more)
```
