---
name: js-parser-benchmark
description: Compare the JavaScript API parser performance of two swc-experimental builds with repeatable fixtures and reduced timing noise. Use when measuring a parser change against main or another branch, investigating a JS parser regression, or checking whether a small wall-time difference is reproducible.
---

# JS parser benchmark

Build both revisions with the same release profile and Node.js version. Give the script the absolute path to each revision's `npm/core/src-js/index.js`. Run it from the workspace root, using one shared fixture directory for both builds.

```sh
node .agents/skills/js-parser-benchmark/scripts/bench.mjs \
  --baseline /absolute/path/to/main/npm/core/src-js/index.js \
  --candidate "$PWD/npm/core/src-js/index.js" \
  --file underscore-1.5.2.js \
  --file jquery-1.9.1.js
```

Omit `--file` to run every JavaScript and TypeScript fixture in `tasks/benchmark/files`. Use `--fixtures <directory>` for a different shared corpus, `--trials <count>` for more independent runs, and `--time-ms <milliseconds>` to change each fixture's timed duration.

The script warms each fixture, measures several rounds, and reports the median of the best round from each fresh Node.js process. It alternates baseline and candidate order. A negative change means the candidate is faster. Report the fixture, both medians, percentage change, trial ranges, Node.js version, build profile, and revision IDs. Treat changes smaller than the observed trial spread as inconclusive; rerun those cases under a quieter machine load.
