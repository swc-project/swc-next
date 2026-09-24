---
name: js-benchmark-stability
description: Stabilize and interpret JavaScript or Node.js performance comparisons when V8 warmup, polymorphic call sites, garbage collection, timer overhead, or host load make results inconsistent. Use for JS microbenchmarks and before/after decoder or parser benchmarks.
---

# Stable JavaScript benchmarks

Measure the operation the user cares about. Keep fixture loading, parsing, module imports, and other setup outside the timed region unless they are explicitly part of the workload. Force lazy work inside it and consume a cheap, deterministic result so the benchmark cannot skip the work. Check that variants produce equivalent results before comparing times.

## Compare variants fairly

- Give each version its own hot call site. Passing several implementations through one shared wrapper can make V8's inline cache polymorphic and change inlining behavior. For small functions, create a separate runner or compilation unit per version; a `new Function` loop with a unique comment is one option. Use separate processes if global state or caches still leak across versions.
- Use the same fixtures, Node version, runtime flags, and measured stage for every version. Warm each version before timing. Interleave or balance version order across rounds to reduce drift from CPU frequency and background work.
- Calibrate batches so a timed round lasts roughly 10 ms, making timer overhead small. Five warmup rounds, at least five timed rounds, and about one second per version per fixture are useful starting points, not fixed requirements. Use one operation per round when the operation itself is slower than the target.

## Interpret noise

- Retain per-round measurements. The fastest round can estimate low-noise execution when GC and scheduling mainly add delay; also report a typical value or spread and repeat the whole benchmark in independent processes. A single minimum does not establish a speedup.
- Run an A/A comparison when the apparent difference is close to the noise floor. If the relative result changes sign across independent runs or is no larger than the observed variation, call it inconclusive. Do not hide such fixtures in an aggregate average.
- Whole-program AST construction allocates heavily. Its GC behavior can dominate a small decoder improvement, so a method that stabilizes a tiny string microbenchmark may still leave full decode results noisy. Measure both an isolated hot path and the full workload when attribution matters.
- Keep default heap settings for representative results. Larger young-generation or heap limits, explicit GC outside timed rounds, CPU affinity, and background-load controls can help diagnose noise, but record them and recheck conclusions under normal settings. CPU pinning can worsen results on heterogeneous or busy cores.
