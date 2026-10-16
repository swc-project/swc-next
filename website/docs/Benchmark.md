# Benchmark

## Rust parse

### astro-config.ts

```text
Benchmark 1: swc-next@f110c5e1dd62288e6b6a74f8c846bc972b1225f4 (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         0.5 ms  ±     0.1 ms       0.5 ms  …     0.8 ms
  Instructions      1.1 M   ±     0.0 M        1.1 M   …     1.1 M
  Memory            3.5 MiB ±     0.0 MiB      3.5 MiB …     3.5 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         0.7 ms  ±     0.0 ms       0.6 ms  …     0.8 ms           +31.9%  (1.3x slower)
  Instructions      2.1 M   ±     0.0 M        2.1 M   …     2.1 M            +84.9%  (1.8x more)
  Memory            4.6 MiB ±     0.1 MiB      4.5 MiB …     4.8 MiB          +32.3%  (1.3x more)
```

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@f110c5e1dd62288e6b6a74f8c846bc972b1225f4 (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         1.0 ms  ±     0.0 ms       1.0 ms  …     1.1 ms
  Instructions      4.4 M   ±     0.0 M        4.4 M   …     4.4 M
  Memory            4.1 MiB ±     0.1 MiB      3.9 MiB …     4.3 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         2.0 ms  ±     0.0 ms       2.0 ms  …     2.1 ms           +97.1%  (2.0x slower)
  Instructions     12.5 M   ±     0.0 M       12.5 M   …    12.5 M           +184.5%  (2.8x more)
  Memory            6.1 MiB ±     0.1 MiB      6.0 MiB …     6.4 MiB          +50.1%  (1.5x more)
```

### hono-types.ts

```text
Benchmark 1: swc-next@f110c5e1dd62288e6b6a74f8c846bc972b1225f4 (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         0.7 ms  ±     0.0 ms       0.7 ms  …     0.8 ms
  Instructions      4.6 M   ±     0.0 M        4.6 M   …     4.6 M
  Memory            3.8 MiB ±     0.1 MiB      3.7 MiB …     3.9 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         1.5 ms  ±     0.0 ms       1.5 ms  …     1.7 ms          +115.9%  (2.2x slower)
  Instructions     14.5 M   ±     0.0 M       14.5 M   …    14.5 M           +215.4%  (3.2x more)
  Memory            5.8 MiB ±     0.1 MiB      5.7 MiB …     6.0 MiB          +51.9%  (1.5x more)
```

## Isolated declaration

### astro-config.ts

```text
Benchmark 1: swc-next@f110c5e1dd62288e6b6a74f8c846bc972b1225f4 (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         0.6 ms  ±     0.0 ms       0.6 ms  …     0.7 ms
  Instructions      1.5 M   ±     0.0 M        1.5 M   …     1.5 M
  Memory            4.2 MiB ±     0.1 MiB      4.0 MiB …     4.4 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         0.8 ms  ±     0.0 ms       0.8 ms  …     0.9 ms           +34.6%  (1.3x slower)
  Instructions      2.7 M   ±     0.0 M        2.7 M   …     2.7 M            +81.7%  (1.8x more)
  Memory            5.2 MiB ±     0.1 MiB      5.0 MiB …     5.3 MiB          +25.2%
```

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@f110c5e1dd62288e6b6a74f8c846bc972b1225f4 (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         1.6 ms  ±     0.0 ms       1.6 ms  …     1.7 ms
  Instructions      8.3 M   ±     0.0 M        8.3 M   …     8.3 M
  Memory            5.3 MiB ±     0.1 MiB      5.1 MiB …     5.4 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         2.6 ms  ±     0.0 ms       2.5 ms  …     2.6 ms           +59.5%  (1.6x slower)
  Instructions     16.3 M   ±     0.0 M       16.3 M   …    16.3 M            +96.3%  (2.0x more)
  Memory            6.8 MiB ±     0.1 MiB      6.6 MiB …     7.1 MiB          +30.2%  (1.3x more)
```

### hono-types.ts

```text
Benchmark 1: swc-next@f110c5e1dd62288e6b6a74f8c846bc972b1225f4 (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         1.6 ms  ±     1.2 ms       1.3 ms  …    10.1 ms
  Instructions     12.8 M   ±     0.0 M       12.8 M   …    12.8 M
  Memory            5.3 MiB ±     0.1 MiB      5.1 MiB …     5.5 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         2.3 ms  ±     0.0 ms       2.2 ms  …     2.4 ms           +44.8%  (1.4x slower)
  Instructions     23.9 M   ±     0.0 M       23.9 M   …    23.9 M            +87.1%  (1.9x more)
  Memory            6.7 MiB ±     0.1 MiB      6.6 MiB …     6.9 MiB          +27.9%
```

## Type strip

### astro-config.ts

```text
Benchmark 1: swc-next@f110c5e1dd62288e6b6a74f8c846bc972b1225f4 (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         0.6 ms  ±     0.0 ms       0.6 ms  …     0.7 ms
  Instructions      1.3 M   ±     0.0 M        1.3 M   …     1.3 M
  Memory            5.4 MiB ±     0.1 MiB      5.4 MiB …     5.6 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         1.0 ms  ±     0.0 ms       1.0 ms  …     1.1 ms           +74.8%  (1.7x slower)
  Instructions      5.8 M   ±     0.0 M        5.8 M   …     5.8 M           +352.5%  (4.5x more)
  Memory            5.2 MiB ±     0.1 MiB      5.0 MiB …     5.4 MiB           -4.2%
```

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@f110c5e1dd62288e6b6a74f8c846bc972b1225f4 (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         1.2 ms  ±     0.0 ms       1.1 ms  …     1.3 ms
  Instructions      5.3 M   ±     0.0 M        5.3 M   …     5.3 M
  Memory            7.6 MiB ±     0.1 MiB      7.4 MiB …     7.7 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         2.3 ms  ±     0.0 ms       2.2 ms  …     2.4 ms           +94.4%  (1.9x slower)
  Instructions     13.9 M   ±     0.0 M       13.9 M   …    13.9 M           +160.0%  (2.6x more)
  Memory            7.0 MiB ±     0.1 MiB      6.7 MiB …     7.2 MiB           -8.5%
```

### hono-types.ts

```text
Benchmark 1: swc-next@f110c5e1dd62288e6b6a74f8c846bc972b1225f4 (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         0.8 ms  ±     0.0 ms       0.7 ms  …     0.9 ms
  Instructions      5.0 M   ±     0.0 M        5.0 M   …     5.0 M
  Memory            4.7 MiB ±     0.7 MiB      4.0 MiB …     5.8 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time         1.8 ms  ±     0.0 ms       1.8 ms  …     1.9 ms          +135.3%  (2.4x slower)
  Instructions     18.3 M   ±     0.0 M       18.3 M   …    18.3 M           +267.2%  (3.7x more)
  Memory            6.5 MiB ±     0.1 MiB      6.4 MiB …     6.7 MiB          +40.0%  (1.4x more)
```

## JS parser

### astro-config.ts

```text
Benchmark 1: @swc-next/core@0.2.4 (f110c5e1dd62288e6b6a74f8c846bc972b1225f4) (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time        28.4 ms  ±     1.6 ms      26.2 ms  …    35.4 ms
  Instructions    182.7 M   ±     3.9 M      174.4 M   …   196.9 M
  Memory           57.8 MiB ±     0.1 MiB     57.4 MiB …    58.0 MiB

Benchmark 2: @swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1) (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time        25.9 ms  ±     2.0 ms      23.3 ms  …    35.9 ms            -9.0%
  Instructions    151.9 M   ±     3.8 M      146.7 M   …   165.7 M            -16.8%
  Memory           67.1 MiB ±     0.1 MiB     67.0 MiB …    67.2 MiB          +16.0%
```

### effect-schema-ast.ts

```text
Benchmark 1: @swc-next/core@0.2.4 (f110c5e1dd62288e6b6a74f8c846bc972b1225f4) (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time        41.3 ms  ±     2.6 ms      36.4 ms  …    52.7 ms
  Instructions    295.0 M   ±    15.2 M      243.4 M   …   346.6 M
  Memory           66.9 MiB ±     1.6 MiB     61.0 MiB …    72.5 MiB

Benchmark 2: @swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1) (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time        36.4 ms  ±     1.4 ms      33.9 ms  …    40.1 ms           -11.8%
  Instructions    316.8 M   ±     4.3 M      310.2 M   …   330.2 M             +7.4%
  Memory           79.7 MiB ±     0.1 MiB     79.5 MiB …    79.8 MiB          +19.0%
```

### hono-types.ts

```text
Benchmark 1: @swc-next/core@0.2.4 (f110c5e1dd62288e6b6a74f8c846bc972b1225f4) (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time        38.6 ms  ±     2.5 ms      34.5 ms  …    53.3 ms
  Instructions    275.6 M   ±    15.1 M      251.4 M   …   317.5 M
  Memory           67.1 MiB ±     1.8 MiB     64.4 MiB …    72.2 MiB

Benchmark 2: @swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1) (50 runs)
                   mean     ±       σ          min     …     max
  Wall Time        37.4 ms  ±     1.9 ms      35.1 ms  …    46.4 ms            -3.2%
  Instructions    357.5 M   ±     3.5 M      352.2 M   …   366.1 M            +29.7%
  Memory           81.7 MiB ±     0.1 MiB     81.5 MiB …    81.8 MiB          +21.6%
```
