# JavaScript walker: SWC Next vs yuku-ast

Measured on 2026-10-09, Apple M5 Max, macOS arm64, Node v24.20.0.
Baseline: published **yuku-ast 0.18.0**. Candidate: PR #840 with generated child
accessors, prototype traversal methods and context storage populated only for
hook phases. Exact candidate source hashes and all per-process summaries are in
[walk-results.json](./walk-results.json).

**All independent walker workloads in this matrix are faster than yuku-ast.**
This is a bounded result on these inputs and this runtime, not a guarantee for
every AST, plugin or JavaScript engine. End-to-end pipeline gains are smaller;
the Backbone pipeline is effectively tied and slightly slower in this run.

## Method and correctness

- Three fresh processes per library/workload; alternate execution order.
- Warm up for at least 12 iterations **and 300 ms**. Measure 25 iterations,
  reporting the median of the three process medians.
- Small inputs batch 1,000 walks per sample, amortizing clock overhead. Hook
  compilation and per-walk setup remain inside the timer.
- GC runs before warmup. A separate jQuery run forces GC after warmup as well;
  its results are retained below. Earlier measurements using only 12 warmups
  and GC immediately before timing are not directly comparable to this table.
- Both walkers receive identical ASTs parsed by SWC Next. Parsing/cloning are
  outside walker timings. The pipeline includes SWC Next parse, four complete
  walker passes and SWC Next print for **both** libraries; it does not compare
  their parser/codegen products.
- Assert identical output hashes, callback counts, and a separate traversal
  trace containing node type/span and context key/index. Trace validation runs
  outside timings and after peak memory collection. Editing starts with a fresh
  clone each iteration and uses operations shared by both libraries.
- Edit workload: 1,000 generated functions, deleting debugger statements,
  replacing numeric literals and inserting statements before/after a call.
  It is the same synthetic input in every fixture run, not a transform of the
  named fixture. Tiny input: `function add(a, b) { return a + b; }` plus newline.
- Imports are outside timing. No competing build/test/benchmark ran during the
  final matrix. Real plugin work can dominate dispatch and reduce these gains.

## Walker timings

Milliseconds per walk (or per four-walk batch); lower is better. Speedup is
`yuku time / SWC Next time`. The table excludes parse/print.

| Input | Workload | yuku-ast ms | SWC Next ms | Speedup |
| --- | --- | ---: | ---: | ---: |
| jquery-1.9.1.js | No hooks | 1.192 | 0.680 | 1.75x |
| jquery-1.9.1.js | Sparse typed hooks | 1.261 | 0.768 | 1.64x |
| jquery-1.9.1.js | Global enter | 1.186 | 0.864 | 1.37x |
| jquery-1.9.1.js | Read context | 1.260 | 1.059 | 1.19x |
| jquery-1.9.1.js | Four walks | 4.766 | 3.653 | 1.30x |
| backbone-1.1.0.js | No hooks | 0.210 | 0.138 | 1.52x |
| backbone-1.1.0.js | Sparse typed hooks | 0.231 | 0.155 | 1.49x |
| backbone-1.1.0.js | Global enter | 0.214 | 0.172 | 1.24x |
| backbone-1.1.0.js | Read context | 0.227 | 0.195 | 1.17x |
| backbone-1.1.0.js | Four walks | 0.881 | 0.704 | 1.25x |
| cal.com.tsx | No hooks | 4.226 | 2.801 | 1.51x |
| cal.com.tsx | Sparse typed hooks | 4.556 | 3.073 | 1.48x |
| cal.com.tsx | Global enter | 4.397 | 3.818 | 1.15x |
| cal.com.tsx | Read context | 4.623 | 3.779 | 1.22x |
| cal.com.tsx | Four walks | 16.594 | 14.748 | 1.13x |
| Tiny function | No hooks | 0.000313 | 0.000199 | 1.58x |
| Tiny function | Sparse typed hooks | 0.000429 | 0.000361 | 1.19x |
| Tiny function | Global enter | 0.000387 | 0.000328 | 1.18x |
| Tiny function | Read context | 0.000401 | 0.000369 | 1.09x |
| Tiny function | Four walks | 0.001581 | 0.001299 | 1.22x |
| Synthetic functions | Replace / remove / insert | 0.869 | 0.751 | 1.16x |

## Pipeline timings

These include the same parser/printer around each walker. Do not interpret the
small pipeline differences as isolated parser or codegen performance.

| Input | yuku walker pipeline ms | SWC Next walker pipeline ms | Speedup |
| --- | ---: | ---: | ---: |
| jquery-1.9.1.js | 27.1542 | 25.8132 | 1.052x |
| backbone-1.1.0.js | 4.7279 | 4.7591 | 0.993x |
| cal.com.tsx | 135.6982 | 133.5335 | 1.016x |
| Tiny function | 0.0148 | 0.0144 | 1.027x |

## Memory

MiB, median across three processes. Peak RSS is the **whole process**, including
imports, parsing, warmup and cloning. Observed heap is the maximum `heapUsed`
sampled after measured iterations, not the instantaneous peak or total allocated
bytes. These are single-process benchmarks, with no separate client/server.
Every workload's memory figures are included in the JSON; representative cases:

| Input / workload | Peak RSS yuku | Peak RSS SWC Next | Observed heap yuku | Observed heap SWC Next |
| --- | ---: | ---: | ---: | ---: |
| jquery-1.9.1.js / Global enter | 78.0 | 72.1 | 12.9 | 10.2 |
| jquery-1.9.1.js / Replace / remove / insert | 121.4 | 105.8 | 44.5 | 28.0 |
| jquery-1.9.1.js / Parse + four walks + print | 202.6 | 203.0 | 50.4 | 44.3 |
| cal.com.tsx / Global enter | 131.7 | 113.9 | 36.8 | 22.3 |
| cal.com.tsx / Parse + four walks + print | 491.3 | 451.7 | 110.6 | 86.5 |

## Forced-GC sensitivity

The same jQuery comparison, additionally forcing GC between warmup and timing.
Both libraries use the same setting. All walker cases still favor SWC Next.

| Workload | yuku-ast ms | SWC Next ms | Speedup |
| --- | ---: | ---: | ---: |
| No hooks | 1.186 | 0.737 | 1.61x |
| Sparse typed hooks | 1.468 | 0.817 | 1.80x |
| Global enter | 1.196 | 0.984 | 1.22x |
| Read context | 1.249 | 1.171 | 1.07x |
| Replace / remove / insert | 0.928 | 0.782 | 1.19x |
| Four walks | 4.895 | 3.897 | 1.26x |
| Parse + four walks + print | 27.660 | 26.639 | 1.04x |

## Task-specific handwritten editing baseline

The additional `--handwritten-edits` case compares the same synthetic edit task
across three implementations: yuku-ast, SWC Next and a handwritten recursive
function in [handwritten-edit.mjs](./handwritten-edit.mjs). The handwritten code
imports no packages and calls no visitor/walker APIs, including during its
independent output trace verification. It edits parent fields and arrays directly,
recurses through the fixture's AST nodes, and throws on unsupported node types.
The common harness still uses SWC Next to parse input outside the timer.

All three delete debugger statements, replace literal `1` with a new literal `2`
node, and insert empty statements before/after `keep()`. They use the same
replacement objects and splice operations; before-inserted nodes are skipped and
after-inserted nodes are traversed. The handwritten version has no configurable
hooks, aliases, reusable context API or general JS/TS syntax support. This is a
**task-specialized reference**, not a drop-in replacement or a universal lower bound.

The fresh comparison rotates the three variants through all three execution
positions. Each runs in three fresh processes, with the same warmup, cloning,
timing and memory rules above. Output hashes, matched edit-node counts and
post-edit position traces agree. Figures below come from this new three-way run,
not a combination with older two-way timings.

| Implementation | Edit time ms | Relative to handwritten | Process peak RSS MiB | Observed heap MiB |
| --- | ---: | ---: | ---: | ---: |
| Handwritten | 0.203 | 1.00x | 123.2 | 40.6 |
| SWC Next | 0.688 | 3.39x | 102.5 | 24.5 |
| yuku-ast 0.18.0 | 0.802 | 3.94x | 120.9 | 44.5 |

The gap includes task specialization and omitted generic dispatch/context checks;
it does not isolate the cost of the API's surface syntax. Process memory includes
parser setup, warmup and input cloning, so faster editing does not imply a lower
whole-process RSS. Per-process data and code hashes:
[handwritten-edit-results.json](./handwritten-edit-results.json).

```sh
node npm/core/benches/walk.bench.mjs /tmp/swc-walk-yuku/node_modules/yuku-ast/dist/index.js --handwritten-edits
```

## Why this implementation can improve on yuku-ast

The source of yuku-ast 0.18.0 still leaves these optimization opportunities:

| Existing mechanism | Applied alternative |
| --- | --- |
| Generic child-key loops and `node[key]` property loads | Generate fixed property accessors from the same schema as visitor keys |
| Position context updated at entry and exit even when no hook matches | Populate the shared position record only for phases with callbacks |
| Runtime Map lookup plus separate global/typed dispatch | Shared type dispatch entries, premerged hook order and direct single-function calls |
| A cursor object for each array field | Keep array indexes in locals and return the updated index after edits |
| A recursive traversal closure created for every walk | Prototype traversal methods reused across walks |

These mechanisms are supported by source inspection and incremental experiments;
the benchmark measures their combined effect, not a causal percentage for each.
The generated module adds about 24.5 KB of JS source and some import-time work.
It uses no `eval` or runtime code generation. A single position record plus an
ancestor stack preserves replacement/removal/insertion and skip/stop behavior.

The prior context contract remains: do not retain ctx; old references alias later
hooks. Outside-hook accesses still throw. Sequential plugin passes are unchanged.

## Reproduce

From the workspace root, with the native core binding built:

```sh
npm install --prefix /tmp/swc-walk-yuku --ignore-scripts --no-package-lock yuku-ast@0.18.0
node npm/core/benches/walk.bench.mjs /tmp/swc-walk-yuku/node_modules/yuku-ast/dist/index.js
node npm/core/benches/walk.bench.mjs /tmp/swc-walk-yuku/node_modules/yuku-ast/dist/index.js tasks/benchmark/files/cal.com.tsx
WALK_BENCH_GC_AFTER_WARMUP=1 node npm/core/benches/walk.bench.mjs /tmp/swc-walk-yuku/node_modules/yuku-ast/dist/index.js
```

Use `backbone-1.1.0.js` or a file containing the tiny input as the optional second
argument for the other cases. The script prints JSON with per-process summaries,
output and traversal hashes, memory figures and the exact measurement mode.
