import assert from "node:assert/strict";
import test from "node:test";

import {
  parseSummaries,
  renderFailureSummary,
  renderSummary,
} from "./render-gungraun-summary.mjs";

function metric(merge, base) {
  return {
    metrics: { Both: [{ Int: merge }, { Int: base }] },
    diffs: { diff_pct: ((merge - base) / base) * 100, factor: 1 },
  };
}

function summary(profiles) {
  return {
    version: 6,
    module_path: "gungraun::semantic::bench_semantic",
    function_name: "bench_semantic",
    id: "typescript_js",
    profiles,
  };
}

test("renders exact instruction counts", () => {
  const output = renderSummary([
    summary([
      {
        tool: "Callgrind",
        summaries: {
          total: { summary: { Callgrind: { Ir: metric(90, 100) } } },
        },
      },
    ]),
  ]);

  assert.match(output, /Instructions \(head \\\| base\)/);
  assert.match(output, /90 \\\| 100/);
  assert.match(output, /−10\.0000% 🟢/);
});

test("renders aggregated cache and DHAT metrics", () => {
  const output = renderSummary([
    summary([
      {
        tool: "Callgrind",
        summaries: {
          total: {
            summary: {
              Callgrind: {
                Ir: metric(90, 100),
                EstimatedCycles: metric(450, 500),
                D1mr: metric(20, 25),
                D1mw: metric(10, 15),
                DLmr: metric(4, 5),
                DLmw: metric(2, 3),
              },
            },
          },
        },
      },
      {
        tool: "DHAT",
        summaries: {
          total: {
            summary: {
              Dhat: {
                TotalBytes: metric(900, 1_000),
                TotalBlocks: metric(90, 100),
                AtTGmaxBytes: metric(450, 500),
                AtTGmaxBlocks: metric(45, 50),
              },
            },
          },
        },
      },
    ]),
  ]);

  assert.match(output, /D1 data misses/);
  assert.match(output, /30 \\\| 40/);
  assert.match(output, /LL data misses/);
  assert.match(output, /6 \\\| 8/);
  assert.match(output, /Total bytes/);
  assert.match(output, /900 \\\| 1,000/);
  assert.match(output, /Peak blocks/);
});

test("parses JSON arrays and JSON lines", () => {
  const value = summary([]);
  assert.deepEqual(parseSummaries(JSON.stringify([value])), [value]);
  assert.deepEqual(
    parseSummaries(`${JSON.stringify(value)}\n${JSON.stringify(value)}`),
    [value, value],
  );
});

test("rejects unsupported summary schemas", () => {
  assert.throws(
    () => renderSummary([{ ...summary([]), version: 5 }]),
    /Unsupported Gungraun summary schema version/,
  );
});

test("sanitizes renderer failures", () => {
  assert.doesNotMatch(renderFailureSummary("bad `value`\nnext"), /`value`/);
});
