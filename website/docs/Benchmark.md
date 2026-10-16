# Benchmark

## Rust parse

### typescript.js

```sh
taskset --cpu-list 0 hyperfine \
  --style basic \
  --shell none \
  --warmup 20 \
  --runs 200 \
  --metrics time_wall_clock:ms,memory_peak_resident:MiB \
  --command-name swc-next@82c870234d6c050577cbbca2fd59e2bd29055eee \
  'target/website-benchmark/release/swc-next tasks/benchmark/files/typescript.js parse' \
  --command-name swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da \
  'target/website-benchmark/release/swc tasks/benchmark/files/typescript.js parse'
```

```text
Benchmark 1: swc-next@82c870234d6c050577cbbca2fd59e2bd29055eee (200 runs)
                mean     ±       σ          min     …     max
  Wall Time     21.8 ms  ±     0.3 ms      21.3 ms  …    23.6 ms
  Memory        37.8 MiB ±     0.1 MiB     37.6 MiB …    38.0 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (200 runs)
                mean     ±       σ          min     …     max
  Wall Time     93.5 ms  ±     0.8 ms      92.4 ms  …   102.1 ms          +329.4%  (4.3x slower)
  Memory        97.8 MiB ±     0.1 MiB     97.5 MiB …    98.0 MiB         +159.0%  (2.6x more)
```

### effect-schema-ast.ts

```sh
taskset --cpu-list 0 hyperfine \
  --style basic \
  --shell none \
  --warmup 20 \
  --runs 200 \
  --metrics time_wall_clock:ms,memory_peak_resident:MiB \
  --command-name swc-next@82c870234d6c050577cbbca2fd59e2bd29055eee \
  'target/website-benchmark/release/swc-next tasks/benchmark/files/effect-schema-ast.ts parse' \
  --command-name swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da \
  'target/website-benchmark/release/swc tasks/benchmark/files/effect-schema-ast.ts parse'
```

```text
Benchmark 1: swc-next@82c870234d6c050577cbbca2fd59e2bd29055eee (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      1.0 ms  ±     0.0 ms       1.0 ms  …     1.1 ms
  Memory         4.1 MiB ±     0.1 MiB      3.9 MiB …     4.3 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      2.0 ms  ±     0.1 ms       1.9 ms  …     2.6 ms           +97.6%  (2.0x slower)
  Memory         6.2 MiB ±     0.1 MiB      6.0 MiB …     6.4 MiB          +51.2%  (1.5x more)
```

## Isolated declaration

### effect-schema-ast.ts

```sh
taskset --cpu-list 0 hyperfine \
  --style basic \
  --shell none \
  --warmup 20 \
  --runs 200 \
  --metrics time_wall_clock:ms,memory_peak_resident:MiB \
  --command-name swc-next@82c870234d6c050577cbbca2fd59e2bd29055eee \
  'target/website-benchmark/release/swc-next tasks/benchmark/files/effect-schema-ast.ts isolated-declaration' \
  --command-name swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da \
  'target/website-benchmark/release/swc tasks/benchmark/files/effect-schema-ast.ts isolated-declaration'
```

```text
Benchmark 1: swc-next@82c870234d6c050577cbbca2fd59e2bd29055eee (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      1.6 ms  ±     0.0 ms       1.6 ms  …     1.7 ms
  Memory         5.3 MiB ±     0.1 MiB      5.0 MiB …     5.4 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      2.6 ms  ±     0.1 ms       2.5 ms  …     3.4 ms           +58.8%  (1.6x slower)
  Memory         6.9 MiB ±     0.1 MiB      6.6 MiB …     7.1 MiB          +30.0%  (1.3x more)
```

## Type strip

### effect-schema-ast.ts

```sh
taskset --cpu-list 0 hyperfine \
  --style basic \
  --shell none \
  --warmup 20 \
  --runs 200 \
  --metrics time_wall_clock:ms,memory_peak_resident:MiB \
  --command-name swc-next@82c870234d6c050577cbbca2fd59e2bd29055eee \
  'target/website-benchmark/release/swc-next tasks/benchmark/files/effect-schema-ast.ts type-strip' \
  --command-name swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da \
  'target/website-benchmark/release/swc tasks/benchmark/files/effect-schema-ast.ts type-strip'
```

```text
Benchmark 1: swc-next@82c870234d6c050577cbbca2fd59e2bd29055eee (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      1.2 ms  ±     0.0 ms       1.1 ms  …     1.4 ms
  Memory         7.6 MiB ±     0.1 MiB      7.4 MiB …     7.7 MiB

Benchmark 2: swc@ff6a3c3b198d0d31a5e3a84dc0bf41542be534da (200 runs)
                mean     ±       σ          min     …     max
  Wall Time      2.3 ms  ±     0.0 ms       2.2 ms  …     2.4 ms           +92.9%  (1.9x slower)
  Memory         6.9 MiB ±     0.1 MiB      6.7 MiB …     7.2 MiB           -8.3%
```

## JS parser

### typescript.js

```sh
taskset --cpu-list 0 hyperfine \
  --style basic \
  --shell none \
  --warmup 0 \
  --runs 2 \
  --metrics time_wall_clock:ms,memory_peak_resident:MiB \
  --command-name '@swc-next/core@0.2.4 (82c870234d6c050577cbbca2fd59e2bd29055eee); 25 parses + 5 warmups' \
  '../../.local/share/mise/installs/node/24.21.0/bin/node tasks/benchmark/bin/swc_next.js tasks/benchmark/files/typescript.js parse 25' \
  --command-name '@swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1); 25 parses + 5 warmups' \
  '../../.local/share/mise/installs/node/24.21.0/bin/node tasks/benchmark/bin/swc.js tasks/benchmark/files/typescript.js parse 25'
```

```text
Benchmark 1: @swc-next/core@0.2.4 (82c870234d6c050577cbbca2fd59e2bd29055eee); 25 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time   4050.7 ms  ±   551.6 ms    3660.6 ms  …  4440.7 ms
  Memory      1300.1 MiB ±    39.4 MiB   1272.2 MiB …  1327.9 MiB

Benchmark 2: @swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1); 25 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time  20686.1 ms  ±   118.2 ms   20602.5 ms  … 20769.7 ms          +410.7%  (5.1x slower)
  Memory      2601.1 MiB ±     0.2 MiB   2601.0 MiB …  2601.3 MiB         +100.1%  (2.0x more)
```

### effect-schema-ast.ts

```sh
taskset --cpu-list 0 hyperfine \
  --style basic \
  --shell none \
  --warmup 0 \
  --runs 2 \
  --metrics time_wall_clock:ms,memory_peak_resident:MiB \
  --command-name '@swc-next/core@0.2.4 (82c870234d6c050577cbbca2fd59e2bd29055eee); 753 parses + 5 warmups' \
  '../../.local/share/mise/installs/node/24.21.0/bin/node tasks/benchmark/bin/swc_next.js tasks/benchmark/files/effect-schema-ast.ts parse 753' \
  --command-name '@swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1); 753 parses + 5 warmups' \
  '../../.local/share/mise/installs/node/24.21.0/bin/node tasks/benchmark/bin/swc.js tasks/benchmark/files/effect-schema-ast.ts parse 753'
```

```text
Benchmark 1: @swc-next/core@0.2.4 (82c870234d6c050577cbbca2fd59e2bd29055eee); 753 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time   3124.6 ms  ±    54.2 ms    3086.3 ms  …  3162.9 ms
  Memory       652.9 MiB ±     4.7 MiB    649.6 MiB …   656.2 MiB

Benchmark 2: @swc/core@1.15.46 (93831621baf6fe999ec5fcc6fc96388f072c19f1); 753 parses + 5 warmups (2 runs)
                mean     ±       σ          min     …     max
  Wall Time   6615.2 ms  ±    73.2 ms    6563.4 ms  …  6667.0 ms          +111.7%  (2.1x slower)
  Memory       101.5 MiB ±     0.0 MiB    101.5 MiB …   101.5 MiB          -84.5%  (6.4x less)
```
