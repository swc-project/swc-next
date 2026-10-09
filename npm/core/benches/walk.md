# JavaScript walker benchmark

Local measurement on 2026-10-09, Apple M5 Max, macOS arm64, Node v24.20.0.
Baseline: PR #840 commit `7533e35c5c8e368a4ec96f22f352a7e5ed36b593`.
Candidate: one context per walk, prototype methods, and reusable depth frames.

## Method

Each variant/scenario uses three fresh processes, alternating baseline/candidate
order. Each process performs 12 warmups and 25 timed iterations. Times below are
the median of the three process medians; no outlier filtering is applied.
Parsing, input cloning and output validation are outside traversal timings.
The pipeline case includes parse, four complete walks and print.
All output hashes and callback counts match across variants and samples.

Traversal scenarios use the repository's `jquery-1.9.1.js` fixture. The editing
scenario uses 1,000 generated functions, removing debugger statements, replacing
numeric literals and inserting statements before/after a call. Each iteration
starts with a fresh clone. These workloads isolate hook dispatch and structural
editing; they do not establish a ranking against other libraries or guarantee
the same speedup for real plugins. Heavy plugin work can dominate traversal.

| Workload | Before (ms) | After (ms) | Speedup |
| --- | ---: | ---: | ---: |
| No hooks | 2.45 | 2.49 | 0.98x |
| FunctionDeclaration hook | 2.71 | 2.55 | 1.06x |
| Global enter hook | 21.23 | 2.70 | 7.86x |
| Replace / remove / insert | 2.84 | 1.51 | 1.88x |
| Four walks, global enter | 80.66 | 10.27 | 7.85x |
| Parse + four walks + print | 106.25 | 34.06 | 3.12x |

The no-hook result is essentially unchanged (slightly slower in this run).
Most improvement is in callback-heavy traversal, where the old implementation
constructed and froze a new context with closures for every callback.

## Memory

All values are MiB. Peak RSS is the entire process high-water mark, including
imports, parsing, warmup and cloning. Heap is the maximum `heapUsed` sampled after
each measured iteration, **not** the true instantaneous peak or total allocated
bytes. Values are medians across the three processes. These are single-process
measurements; there are no separate client/server processes. The benchmark runs
GC after warmup, but not between measured iterations, so normal GC costs remain.

| Workload | Peak RSS before | Peak RSS after | Observed heap before | Observed heap after |
| --- | ---: | ---: | ---: | ---: |
| No hooks | 76.9 | 77.0 | 12.4 | 12.6 |
| FunctionDeclaration hook | 86.2 | 77.0 | 17.1 | 12.6 |
| Global enter hook | 317.6 | 77.5 | 144.7 | 12.6 |
| Replace / remove / insert | 283.2 | 106.3 | 134.3 | 23.1 |
| Four walks, global enter | 317.4 | 77.3 | 162.9 | 12.7 |
| Parse + four walks + print | 384.2 | 303.8 | 169.1 | 107.5 |

## Reproduce

From the workspace root with the native core binding built:

```sh
mkdir -p /tmp/swc-walk-before
git show 7533e35c5c8e368a4ec96f22f352a7e5ed36b593:npm/core/src-js/ast.js > /tmp/swc-walk-before/ast.js
git show 7533e35c5c8e368a4ec96f22f352a7e5ed36b593:npm/core/src-js/visitor-keys.js > /tmp/swc-walk-before/visitor-keys.js
node npm/core/benches/walk.bench.mjs /tmp/swc-walk-before/ast.js > /tmp/swc-walk-benchmark.json
```

The JSON contains all per-process timings, output hashes, callback counts and
memory samples. Keep competing benchmarks/builds stopped during measurements.

## Context contract

A context is now reused across callbacks within one walk. Retaining it is
unsupported: an old reference used during a later callback aliases that callback's
position. Use outside a callback still throws. Nested walks have independent
contexts. Replacement, removal, insertion, skip/stop and sequential plugin-pass
semantics are unchanged. This lifecycle change is intentional, not transparent
preservation of the previous per-callback stale-reference guarantee.
