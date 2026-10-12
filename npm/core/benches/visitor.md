# Application walker optimization

This comparison changes only SWC Next's JavaScript walker implementation. It uses
the existing SWC Next adapters from
[visitor-benchmark](https://github.com/SoonIter/visitor-benchmark), without changing
their business logic, input ASTs, or editing APIs. No `transformSync`, parser,
printer, or public API changes are required.

## Implementation

- Generated child accessors check optional fields before calling `visitField`.
  A missing field no longer enters the generic array/scalar traversal helper.
  Each field is read once, immediately before visiting it, so edits to subsequent
  fields and early `stop()` retain their original behavior.
- Ancestor storage uses reusable depth slots. A parent is added while traversing
  a present child field; nodes with no present children do not update this stack.
  `ancestors()` returns a copy of its live prefix. Walk completion, stop and errors
  still invalidate the context and release stored ancestor references.
- Replacement, removal, insertion, skip, stop, alias ordering, callback return
  validation and context lifetime retain their existing contracts. Hook-phase
  specialization and copying the full dispatch table were tried and not retained.

The generated child module grows from 24,489 to 43,654 source bytes (19,165 extra
bytes, uncompressed). This trades source size for fewer generic helper calls;
there is no runtime code generation. The measurements below cover warm traversal,
not import/startup costs or bundle compression.

## Results

Measured on 2026-10-12, Node v24.20.0, Apple M5 Max, macOS arm64.
Milliseconds per corpus; parentheses contain min–max process medians.
Editing corpora contain four files; i18n corpora contain 64.

| Scenario / input | Baseline ms | Optimized ms | Median time reduction |
| :-- | --: | --: | --: |
| styled-components/dense | 1.303 (1.259–1.474) | 1.182 (1.141–1.210) | 9.3% |
| styled-components/sparse | 1.992 (1.979–2.059) | 1.911 (1.887–1.946) | 4.1% |
| remove-debugger/dense | 0.596 (0.575–0.598) | 0.567 (0.563–0.578) | 4.8% |
| remove-debugger/sparse | 1.136 (1.038–1.321) | 1.002 (0.971–1.077) | 11.8% |
| rstest-coverage/runtime | 0.556 (0.549–0.558) | 0.520 (0.515–0.549) | 6.4% |
| rstest-coverage/tooling | 0.492 (0.460–0.525) | 0.463 (0.462–0.466) | 5.9% |
| i18n/original | 1.781 (1.780–1.810) | 1.647 (1.646–1.676) | 7.5% |
| i18n/javascript | 0.759 (0.754–0.990) | 0.740 (0.711–1.118) | 2.5% |

The i18n JavaScript group was noisy in the initial matrix, including a slower
candidate in one process pair. Its 2.5% aggregate difference is not a stable
win by itself. Five additional alternating process pairs with the same worker
produced **0.791 (0.762–0.811) ms baseline and 0.738 (0.724–0.772) ms optimized**
(6.7% lower median time). Both the original samples and this follow-up are retained;
other overlapping ranges also limit claims about small differences. No confidence
interval or universal speedup is claimed.

All **58 process runs, 1,450 timed batches and 8,300 corpus iterations** passed
output and trace parity. The existing walker tests plus the getter/stop regression
(16 tests), public API type checks, generator regeneration, Rust formatting and
ast_tools Clippy checks passed.

[Raw samples, traces and source hashes](./visitor-results.json). The JSON stores
trace hashes, not business source. The existing public multi-library benchmark
snapshots are unchanged; this is a separate optimization comparison.

## Measurement

The baseline is PR #840 at `a20f5fa4755e52e9760d51c41a1f7a02ba3e3316`.
The application adapters are pinned by commit and SHA-256 in the adjacent results.

- Three independent processes per version/input group, with reversed variant and
  input order in the middle repetition. Warm up for at least 12 batches and 500 ms
  of wall time, then record 25 batches per process.
- Each batch uses eight fresh four-file corpora for editing, or two fresh 64-file
  corpora for i18n. Divide its elapsed time by that batch size. All AST preparation
  happens before the timer; every output is checked after it. No edited AST is
  reused for a later edit iteration.
- Hook compilation, traversal setup, callbacks, context maintenance and edits are
  timed. Parsing/cloning, output validation and traversal-trace checks are not.
- Every measured result uses the existing scenario oracle. Input hashes, adapter
  hashes, output hashes, and a separate enter/leave trace (type, positions,
  key/index and ancestors) must match between versions and process repetitions.
- Report the median of three process medians, with their min–max range. These
  ranges are descriptive, not confidence intervals. Batched timing differs from
  the published multi-library harness; compare versions within this run, not
  absolute times against older charts.

## Reproduce

The benchmark checkout must have its locked npm dependencies installed. i18n
additionally needs the prepared private corpus in that checkout; no private source
or dictionary is copied into SWC Next or the result JSON. No native SWC Next
binding is needed.

```sh
mkdir -p /tmp/walker-baseline
git show a20f5fa:npm/core/src-js/ast.js > /tmp/walker-baseline/ast.js
git show a20f5fa:npm/core/src-js/child-visitors.js > /tmp/walker-baseline/child-visitors.js
git show a20f5fa:npm/core/src-js/visitor-keys.js > /tmp/walker-baseline/visitor-keys.js
node npm/core/benches/visitor.bench.mjs \
  /path/to/visitor-benchmark /tmp/walker-baseline/ast.js /tmp/visitor-results.json
```

Correctness checks:

```sh
node --test npm/core/test/ast.test.js
pnpm --filter @swc-next/core test:types
cargo run --locked -p ast_tools
cargo fmt --all --check
cargo clippy --locked -p ast_tools -- -D warnings
```

To repeat only the noisy group, run the existing worker in five alternating
baseline/candidate process pairs, with the final two arguments `i18n javascript`:

```sh
node --expose-gc npm/core/benches/visitor.bench.mjs --worker \
  /path/to/visitor-benchmark /path/to/version/ast.js i18n javascript
```
