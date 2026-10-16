# Benchmark

## Rust parse

### typescript.js

```text
Benchmark 1: swc-next@961215dbfb4035fe63d711bc801d59d58ff4236b (50 runs)
                mean     ±       σ          min     …     max
  Wall Time     22.0 ms  ±     1.3 ms      21.5 ms  …    30.9 ms
  Memory        37.8 MiB ±     0.1 MiB     37.6 MiB …    38.0 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                mean     ±       σ          min     …     max
  Wall Time     94.7 ms  ±     2.2 ms      92.6 ms  …   105.8 ms          +330.4%  (4.3x slower)
  Memory        97.8 MiB ±     0.1 MiB     97.6 MiB …    97.9 MiB         +158.8%  (2.6x more)
```

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@961215dbfb4035fe63d711bc801d59d58ff4236b (50 runs)
                mean     ±       σ          min     …     max
  Wall Time      1.0 ms  ±     0.0 ms       1.0 ms  …     1.1 ms
  Memory         4.1 MiB ±     0.1 MiB      3.9 MiB …     4.2 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                mean     ±       σ          min     …     max
  Wall Time      2.0 ms  ±     0.0 ms       2.0 ms  …     2.1 ms           +96.0%  (2.0x slower)
  Memory         6.2 MiB ±     0.1 MiB      6.0 MiB …     6.4 MiB          +51.1%  (1.5x more)
```

## Isolated declaration

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@961215dbfb4035fe63d711bc801d59d58ff4236b (50 runs)
                mean     ±       σ          min     …     max
  Wall Time      1.6 ms  ±     0.0 ms       1.6 ms  …     1.8 ms
  Memory         5.3 MiB ±     0.1 MiB      5.1 MiB …     5.4 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                mean     ±       σ          min     …     max
  Wall Time      2.6 ms  ±     0.1 ms       2.5 ms  …     2.9 ms           +58.2%  (1.6x slower)
  Memory         6.9 MiB ±     0.1 MiB      6.6 MiB …     7.0 MiB          +30.4%  (1.3x more)
```

## Type strip

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@961215dbfb4035fe63d711bc801d59d58ff4236b (50 runs)
                mean     ±       σ          min     …     max
  Wall Time      1.2 ms  ±     0.0 ms       1.1 ms  …     1.3 ms
  Memory         7.6 MiB ±     0.1 MiB      7.4 MiB …     7.7 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                mean     ±       σ          min     …     max
  Wall Time      2.3 ms  ±     0.0 ms       2.3 ms  …     2.5 ms           +92.2%  (1.9x slower)
  Memory         6.9 MiB ±     0.1 MiB      6.7 MiB …     7.2 MiB           -8.8%
```

## JS parser

### typescript.js

```text
Benchmark 1: @swc-next/core@0.2.4 (961215dbfb4035fe63d711bc801d59d58ff4236b) (50 runs)
                mean     ±       σ          min     …     max
  Wall Time    356.5 ms  ±    25.1 ms     301.9 ms  …   409.0 ms
  Memory       233.8 MiB ±     9.5 MiB    221.3 MiB …   271.5 MiB

Benchmark 2: @swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1) (50 runs)
                mean     ±       σ          min     …     max
  Wall Time    763.0 ms  ±     4.5 ms     752.6 ms  …   773.1 ms          +114.1%  (2.1x slower)
  Memory       823.9 MiB ±     0.0 MiB    823.8 MiB …   824.0 MiB         +252.3%  (3.5x more)
```

### effect-schema-ast.ts

```text
Benchmark 1: @swc-next/core@0.2.4 (961215dbfb4035fe63d711bc801d59d58ff4236b) (50 runs)
                mean     ±       σ          min     …     max
  Wall Time     40.8 ms  ±     2.7 ms      34.6 ms  …    48.7 ms
  Memory        67.1 MiB ±     2.3 MiB     60.8 MiB …    72.7 MiB

Benchmark 2: @swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1) (50 runs)
                mean     ±       σ          min     …     max
  Wall Time     36.2 ms  ±     1.2 ms      34.2 ms  …    39.3 ms           -11.2%
  Memory        79.7 MiB ±     0.1 MiB     79.6 MiB …    79.8 MiB          +18.7%
```
