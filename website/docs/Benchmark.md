# Benchmark

## Rust parse

### astro-config.ts

```text
Benchmark 1: swc-next@575988266d7be2bf76483304546202c11af8160f (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         1.5 ms  ±     0.2 ms       1.1 ms  …     1.9 ms
  Instructions      1.1 M   ±     0.0 M        1.1 M   …     1.1 M
  Memory            3.6 MiB ±     0.0 MiB      3.6 MiB …     3.6 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         0.8 ms  ±     0.1 ms       0.7 ms  …     1.0 ms           -49.9%  (2.0x faster)
  Instructions      2.1 M   ±     0.0 M        2.1 M   …     2.1 M            +84.9%  (1.8x more)
  Memory            4.6 MiB ±     0.1 MiB      4.5 MiB …     4.7 MiB          +29.8%
```

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@575988266d7be2bf76483304546202c11af8160f (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         1.1 ms  ±     0.1 ms       1.0 ms  …     1.2 ms
  Instructions      4.4 M   ±     0.0 M        4.4 M   …     4.4 M
  Memory            4.1 MiB ±     0.1 MiB      4.0 MiB …     4.2 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         2.1 ms  ±     0.1 ms       2.0 ms  …     2.3 ms           +99.3%  (2.0x slower)
  Instructions     12.5 M   ±     0.0 M       12.5 M   …    12.5 M           +184.5%  (2.8x more)
  Memory            6.2 MiB ±     0.1 MiB      6.0 MiB …     6.3 MiB          +51.4%  (1.5x more)
```

### hono-types.ts

```text
Benchmark 1: swc-next@575988266d7be2bf76483304546202c11af8160f (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         0.7 ms  ±     0.0 ms       0.7 ms  …     0.8 ms
  Instructions      4.6 M   ±     0.0 M        4.6 M   …     4.6 M
  Memory            3.8 MiB ±     0.1 MiB      3.7 MiB …     3.9 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         1.6 ms  ±     0.1 ms       1.5 ms  …     1.7 ms          +115.7%  (2.2x slower)
  Instructions     14.5 M   ±     0.0 M       14.5 M   …    14.5 M           +215.3%  (3.2x more)
  Memory            5.8 MiB ±     0.1 MiB      5.7 MiB …     5.9 MiB          +50.9%  (1.5x more)
```

## Isolated declaration

### astro-config.ts

```text
Benchmark 1: swc-next@575988266d7be2bf76483304546202c11af8160f (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         0.6 ms  ±     0.0 ms       0.6 ms  …     0.7 ms
  Instructions      1.5 M   ±     0.0 M        1.5 M   …     1.5 M
  Memory            4.2 MiB ±     0.1 MiB      4.0 MiB …     4.3 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         0.8 ms  ±     0.0 ms       0.8 ms  …     0.8 ms           +34.2%  (1.3x slower)
  Instructions      2.7 M   ±     0.0 M        2.7 M   …     2.7 M            +81.6%  (1.8x more)
  Memory            5.2 MiB ±     0.1 MiB      5.1 MiB …     5.3 MiB          +24.9%
```

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@575988266d7be2bf76483304546202c11af8160f (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         1.6 ms  ±     0.0 ms       1.6 ms  …     1.7 ms
  Instructions      8.3 M   ±     0.0 M        8.3 M   …     8.3 M
  Memory            5.2 MiB ±     0.1 MiB      5.1 MiB …     5.4 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         2.6 ms  ±     0.1 ms       2.5 ms  …     2.8 ms           +60.3%  (1.6x slower)
  Instructions     16.3 M   ±     0.0 M       16.3 M   …    16.3 M            +96.2%  (2.0x more)
  Memory            6.8 MiB ±     0.1 MiB      6.7 MiB …     6.9 MiB          +30.4%  (1.3x more)
```

### hono-types.ts

```text
Benchmark 1: swc-next@575988266d7be2bf76483304546202c11af8160f (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         1.4 ms  ±     0.0 ms       1.4 ms  …     1.5 ms
  Instructions     12.8 M   ±     0.0 M       12.8 M   …    12.8 M
  Memory            5.3 MiB ±     0.1 MiB      5.1 MiB …     5.5 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         2.3 ms  ±     0.0 ms       2.2 ms  …     2.3 ms           +62.3%  (1.6x slower)
  Instructions     23.9 M   ±     0.0 M       23.9 M   …    23.9 M            +87.1%  (1.9x more)
  Memory            6.7 MiB ±     0.1 MiB      6.5 MiB …     6.8 MiB          +26.3%
```

## Type strip

### astro-config.ts

```text
Benchmark 1: swc-next@575988266d7be2bf76483304546202c11af8160f (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         0.6 ms  ±     0.0 ms       0.6 ms  …     0.7 ms
  Instructions      1.3 M   ±     0.0 M        1.3 M   …     1.3 M
  Memory            5.4 MiB ±     0.0 MiB      5.3 MiB …     5.5 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         1.0 ms  ±     0.0 ms       1.0 ms  …     1.1 ms           +70.4%  (1.7x slower)
  Instructions      5.8 M   ±     0.0 M        5.8 M   …     5.8 M           +352.4%  (4.5x more)
  Memory            5.2 MiB ±     0.1 MiB      5.1 MiB …     5.4 MiB           -3.3%
```

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@575988266d7be2bf76483304546202c11af8160f (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         1.2 ms  ±     0.0 ms       1.2 ms  …     1.3 ms
  Instructions      5.3 M   ±     0.0 M        5.3 M   …     5.3 M
  Memory            7.6 MiB ±     0.1 MiB      7.5 MiB …     7.7 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         2.4 ms  ±     0.1 ms       2.3 ms  …     2.5 ms           +94.0%  (1.9x slower)
  Instructions     13.9 M   ±     0.0 M       13.9 M   …    13.9 M           +159.9%  (2.6x more)
  Memory            6.9 MiB ±     0.1 MiB      6.7 MiB …     7.1 MiB           -9.0%
```

### hono-types.ts

```text
Benchmark 1: swc-next@575988266d7be2bf76483304546202c11af8160f (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         0.8 ms  ±     0.0 ms       0.8 ms  …     0.9 ms
  Instructions      5.0 M   ±     0.0 M        5.0 M   …     5.0 M
  Memory            4.6 MiB ±     0.7 MiB      4.0 MiB …     5.7 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time         1.9 ms  ±     0.0 ms       1.8 ms  …     2.0 ms          +132.7%  (2.3x slower)
  Instructions     18.3 M   ±     0.0 M       18.3 M   …    18.3 M           +267.1%  (3.7x more)
  Memory            6.5 MiB ±     0.1 MiB      6.4 MiB …     6.6 MiB          +42.4%  (1.4x more)
```

## JS parser

### astro-config.ts

```text
Benchmark 1: swc-next@575988266d7be2bf76483304546202c11af8160f (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time        28.1 ms  ±     1.1 ms      26.4 ms  …    29.4 ms
  Instructions    181.5 M   ±     4.1 M      174.9 M   …   186.9 M
  Memory           57.8 MiB ±     0.1 MiB     57.4 MiB …    57.9 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time        26.5 ms  ±     4.3 ms      23.5 ms  …    37.1 ms            -6.0%
  Instructions    144.8 M   ±     3.8 M      141.2 M   …   152.0 M            -20.2%
  Memory           65.8 MiB ±     0.1 MiB     65.7 MiB …    65.9 MiB          +13.8%
```

### effect-schema-ast.ts

```text
Benchmark 1: swc-next@575988266d7be2bf76483304546202c11af8160f (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time        40.7 ms  ±     1.0 ms      39.4 ms  …    42.5 ms
  Instructions    291.8 M   ±     3.4 M      288.2 M   …   299.1 M
  Memory           66.5 MiB ±     0.2 MiB     66.2 MiB …    66.8 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time        34.4 ms  ±     1.0 ms      32.8 ms  …    36.3 ms           -15.4%
  Instructions    297.9 M   ±     2.3 M      294.3 M   …   302.9 M             +2.1%
  Memory           78.1 MiB ±     0.1 MiB     78.0 MiB …    78.2 MiB          +17.4%
```

### hono-types.ts

```text
Benchmark 1: swc-next@575988266d7be2bf76483304546202c11af8160f (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time        39.8 ms  ±     2.9 ms      36.8 ms  …    46.6 ms
  Instructions    278.2 M   ±    14.7 M      255.8 M   …   302.0 M
  Memory           67.1 MiB ±     1.7 MiB     64.4 MiB …    68.5 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (10 runs)
                   mean     ±       σ          min     …     max
  Wall Time        37.1 ms  ±     3.0 ms      34.1 ms  …    44.6 ms            -6.7%
  Instructions    338.5 M   ±     4.6 M      333.5 M   …   349.0 M            +21.7%
  Memory           80.2 MiB ±     0.0 MiB     80.2 MiB …    80.3 MiB          +19.7%
```
