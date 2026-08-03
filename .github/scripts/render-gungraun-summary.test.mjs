import assert from "node:assert/strict";
import test from "node:test";

import { parseSummaries, renderSummary } from "./render-gungraun-summary.mjs";

function count(merge, base) {
  return {
    metrics: { Both: [{ Int: merge }, { Int: base }] },
  };
}

function summary(id, values) {
  const defaults = {
    D1mr: [100, 110],
    D1mw: [20, 20],
    DLmr: [10, 12],
    DLmw: [2, 2],
    I1mr: [30, 30],
    ILmr: [3, 3],
    EstimatedCycles: [1_000, 1_100],
    Ir: [500, 500],
  };
  const metrics = Object.fromEntries(
    Object.entries({ ...defaults, ...values }).map(([event, pair]) => [
      event,
      count(pair[0], pair[1]),
    ]),
  );
  return {
    version: 6,
    module_path: "gungraun::semantic::bench_semantic",
    function_name: "bench_semantic",
    id,
    profiles: [
      {
        tool: "Callgrind",
        summaries: { total: { summary: { Callgrind: metrics } } },
      },
    ],
  };
}

test("renders exact combined cache counts and profiling metrics", () => {
  const output = renderSummary([summary("typescript")]);
  assert.match(output, /D1 data misses/);
  assert.match(output, /120 \\| 130/);
  assert.match(output, /LL data misses/);
  assert.match(output, /12 \\| 14/);
  assert.match(output, /1,000 \\| 1,100/);
  assert.match(output, /500 \\| 500/);
  assert.match(output, /−7\.69231% 🟢/);
});

test("parses newline-delimited schema summaries", () => {
  const input = `${JSON.stringify(summary("typescript"))}\n${JSON.stringify(summary("cal_com_tsx"))}`;
  assert.equal(parseSummaries(input).length, 2);
});

test("rejects summaries without cache metrics", () => {
  const invalid = summary("typescript");
  delete invalid.profiles[0].summaries.total.summary.Callgrind.D1mr;
  assert.throws(() => renderSummary([invalid]), /Missing D1mr values/);
});
