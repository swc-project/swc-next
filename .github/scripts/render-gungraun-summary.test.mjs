import assert from "node:assert/strict";
import test from "node:test";

import {
  parseSummaries,
  renderSummary,
} from "./render-gungraun-summary.mjs";

function summary(head, base) {
  return {
    version: 6,
    module_path: "gungraun::semantic::bench_semantic",
    function_name: "bench_semantic",
    id: "typescript",
    profiles: [
      {
        tool: "Callgrind",
        summaries: {
          total: {
            summary: {
              Callgrind: {
                Ir: {
                  metrics: { Both: [{ Int: head }, { Int: base }] },
                  diffs: { diff_pct: ((head - base) / base) * 100 },
                },
              },
            },
          },
        },
      },
    ],
  };
}

test("renders exact head and base instruction counts", () => {
  const output = renderSummary([summary(90, 100)]);
  assert.match(output, /Head instructions/);
  assert.match(output, /90 \| 100/);
  assert.match(output, /−10\.0000% 🟢/);
});

test("parses arrays and JSON lines", () => {
  const value = summary(90, 100);
  assert.deepEqual(parseSummaries(JSON.stringify([value])), [value]);
  assert.deepEqual(
    parseSummaries(`${JSON.stringify(value)}\n${JSON.stringify(value)}`),
    [value, value],
  );
});

test("rejects unsupported schema versions", () => {
  assert.throws(
    () => renderSummary([{ ...summary(90, 100), version: 5 }]),
    /Unsupported Gungraun summary schema version/,
  );
});
