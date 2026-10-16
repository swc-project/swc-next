import assert from "node:assert/strict";
import { pathToFileURL } from "node:url";
import { loadSummaries, renderGroupTable } from "./render-gungraun-summary.mjs";

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
  const groups = new Map();
  for (const row of rows) {
    const [operation, ...fixture] = row.name.split("/");
    const group = groups.get(operation) ?? [];
    group.push({
      benchmark: `js_api::${operation}`,
      caseName: fixture.join("/"),
      compareInstructions: row.head.median,
      baseInstructions: row.base.median,
      comparison: "both",
      change: row.change,
      inconclusive: row.assessment === "Inconclusive (ranges overlap)",
    });
    groups.set(operation, group);
  }
  const interval = stats => `${stats.median} (${stats.min}–${stats.max})`;
  return [
    "### Callgrind metric changes", "",
    "Median instruction counts from three processes, shown as compare / base in K (1,000), truncated to whole thousands. Percentages are rounded to at most three decimal places. Lower is better: 🟢 improvement, 🔴 regression, ⚪ unchanged or overlapping ranges (inconclusive).", "",
    "JIT disabled (--jitless); predictable mode enabled (--predictable). Counts are not wall time. Exact counts, ranges and profiles are retained in artifacts.", "",
    ...[...groups].map(([operation, group]) => renderGroupTable(operation, group)),
    "<details>",
    "<summary>Benchmark results</summary>", "",
    "Exact instruction counts: median (min–max).", "",
    "| Benchmark | Compare Ir | Base Ir | Assessment |",
    "| --- | ---: | ---: | --- |",
    ...rows.map(row => `| \`${row.name}\` | ${interval(row.head)} | ${interval(row.base)} | ${row.assessment} |`), "",
    "</details>", "",
  ].join("\n");
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  console.log(render(compare(await loadSummaries(process.argv[2]))));
}
