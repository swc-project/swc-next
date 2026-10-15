import assert from "node:assert/strict";
import { pathToFileURL } from "node:url";
import { loadSummaries } from "./render-gungraun-summary.mjs";

export function compare(summaries) {
  const groups = new Map();
  for (const summary of summaries) {
    assert.equal(String(summary.version), "7", "unsupported Gungraun schema");
    const match = /^(.*)_trial_([123])$/.exec(summary.id);
    assert(match, `invalid trial identity: ${summary.id}`);
    const key = `${summary.function_name}/${match[1]}`;
    const metrics = summary.profiles?.find(p => p.tool === "Callgrind")?.data?.total?.metrics;
    const values = metrics?.Ir?.values;
    assert(values && Number.isSafeInteger(values.new) && Number.isSafeInteger(values.old), `missing comparison: ${key}`);
    assert(values.new > 0 && values.old > 0, `empty measurement: ${key}`);
    const trials = groups.get(key) ?? new Map();
    assert(!trials.has(match[2]), `duplicate trial: ${key}`);
    trials.set(match[2], { base: values.old, head: values.new });
    groups.set(key, trials);
  }
  assert(groups.size > 0, "no JS comparisons");
  const stats = values => {
    values.sort((a, b) => a - b);
    return { min: values[0], median: values[1], max: values[2] };
  };
  return [...groups].sort(([a], [b]) => a.localeCompare(b)).map(([name, trials]) => {
    assert.equal(trials.size, 3, `incomplete trials: ${name}`);
    const base = stats([...trials.values()].map(t => t.base));
    const head = stats([...trials.values()].map(t => t.head));
    const overlap = base.min <= head.max && head.min <= base.max;
    return { name, base, head, change: (head.median / base.median - 1) * 100,
      assessment: overlap ? "Inconclusive (ranges overlap)" : head.median < base.median ? "Lower Ir" : "Higher Ir",
      trials: Object.fromEntries(trials) };
  });
}

export function validateRepeatability(rows) {
  assert(rows.length > 0, "no same-SHA comparisons");
  for (const row of rows) {
    assert(row.base.min <= row.head.max && row.head.min <= row.base.max,
      `same-SHA ranges do not overlap: ${row.name}`);
  }
}

export function render(rows) {
  const interval = s => `${s.median} (${s.min}–${s.max})`;
  return [
    "## JavaScript API Gungraun comparison", "",
    "JIT disabled (--jitless); V8 predictable mode enabled (--predictable). Do not compare these counts with JIT-enabled runs or interpret them as normal JIT-enabled execution speed. Main-thread Callgrind Ir includes synchronous JS, N-API and Rust work, plus the measurement callback/loop. Node startup, fixture setup and warmup are excluded. These are instruction counts, not elapsed time. Each row shows the median and range of three fresh processes; overlapping ranges are inconclusive.", "",
    "| Case | Base Ir: median (range) | Head Ir: median (range) | Change | Assessment |",
    "| --- | ---: | ---: | ---: | --- |",
    ...rows.map(r => `| ${r.name} | ${interval(r.base)} | ${interval(r.head)} | ${r.change.toFixed(3)}% | ${r.assessment} |`), "",
  ].join("\n");
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  console.log(render(compare(await loadSummaries(process.argv[2]))));
}
