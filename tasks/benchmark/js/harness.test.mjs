import assert from "node:assert/strict";
import test from "node:test";
import { apiNames, cases, execute, prepare, selectCases, touch } from "./runner.mjs";
import { compare, render, validateRepeatability } from "../../../.github/scripts/render-gungraun-js-summary.mjs";

const api = Object.fromEntries(apiNames.map(name => [name, () => ({ diagnostics: [], program: { type: "Program", body: [] } })]));

test("measurement excludes warmup, GC and validation and consumes each result", () => {
  let active = false;
  let calls = 0;
  let validations = 0;
  execute({ iterations: 10, run() { calls++; assert.equal(active, calls > 100); return calls; },
    validate(value) { assert(!active); assert(value >= 100); validations++; } },
  callback => { active = true; callback(); active = false; }, () => { assert(!active); assert.equal(calls, 100); });
  assert.equal(calls, 110);
  assert.equal(validations, 11);
});

test("lazy traversal and case validation", () => {
  let reads = 0;
  assert.equal(touch({ get nested() { reads++; return { value: "text" }; } }), 3);
  assert.equal(reads, 1);
  assert.throws(() => prepare(api, "invalid", "underscore-1.5.2.js", ""));
  assert.throws(() => prepare(api, "parse", "missing.js", ""));
  assert.throws(() => prepare({ ...api, extra() {} }, "parse", "underscore-1.5.2.js", ""));
  assert.equal(prepare(api, "lang", "js").iterations, 10000);
  assert.equal(prepare(api, "parse", "underscore-1.5.2.js", "").iterations, 10);
});

test("measurement and validation errors propagate", () => {
  const failure = new Error("failure");
  assert.throws(() => execute({ iterations: 1, run() {}, validate() {} }, () => { throw failure; }, () => {}), failure);
  assert.throws(() => execute({ iterations: 1, run() {}, validate() { throw failure; } }, () => {}, () => {}), failure);
});

function summary(trial, base, head) {
  return { version: "7", function_name: "parse", id: `fixture_trial_${trial}`,
    profiles: [{ tool: "Callgrind", data: { total: { metrics: { Ir: { values: { old: base, new: head } } } } } }] };
}

test("three trial comparison retains exact counts and marks overlap inconclusive", () => {
  const rows = compare([summary(1, 100, 90), summary(2, 102, 110), summary(3, 101, 95)]);
  assert.deepEqual(rows[0].base, { min: 100, median: 101, max: 102 });
  assert.equal(rows[0].head.median, 95);
  assert.equal(rows[0].assessment, "Inconclusive (ranges overlap)");
  assert.match(render(rows), /100–102/);
  assert.equal(compare([summary(1, 100, 90), summary(2, 102, 91), summary(3, 101, 92)])[0].assessment, "Lower Ir");
});

test("invalid schema, missing baseline and failed or duplicate trials are rejected", () => {
  assert.throws(() => compare([]));
  assert.throws(() => compare([summary(1, 100, 90)]));
  assert.throws(() => compare([summary(1, undefined, 90)]));
  assert.throws(() => compare([summary(1, 100, 0)]));
  assert.throws(() => compare([{ ...summary(1, 100, 90), version: "8" }]));
  assert.throws(() => compare([summary(1, 100, 90), summary(1, 100, 90)]));
});

test("parallel matrix covers every operation and fixture exactly once", () => {
  assert.equal(cases.length, 24);
  assert.equal(new Set(cases).size, 24);
  assert.equal(cases.filter(name => name.startsWith("parse/")).length, 4);
  assert.equal(cases.filter(name => name.startsWith("parse_full/")).length, 4);
  assert.equal(cases.filter(name => name.startsWith("print/")).length, 4);
  assert.equal(cases.filter(name => name.startsWith("lang/")).length, 6);
  assert.equal(cases.filter(name => name.startsWith("source_type/")).length, 6);
});

test("collector rejects missing shards and combines actual summary files", async () => {
  const { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync } = await import("node:fs");
  const { tmpdir } = await import("node:os");
  const { join, resolve } = await import("node:path");
  const { spawnSync } = await import("node:child_process");
  const directory = mkdtempSync(join(tmpdir(), "js-bench-report-"));
  const artifact = join(directory, "gungraun-js-results-0");
  mkdirSync(artifact);
  writeFileSync(join(artifact, "metadata.json"), JSON.stringify({ case: "parse/fixture", status: "passed", base_sha: "base", head_sha: "head", hashes: { driver: "same" } }));
  for (let trial = 1; trial <= 3; trial++) {
    mkdirSync(join(artifact, String(trial)));
    writeFileSync(join(artifact, String(trial), "summary.json"), JSON.stringify(summary(trial, 100, 90)));
  }
  const collect = (names, repeatability = "success") => spawnSync(process.execPath, [resolve(".github/scripts/collect-gungraun-js.mjs"), directory], {
    env: { ...process.env, GITHUB_STEP_SUMMARY: "", JS_BENCH_REPEATABILITY: repeatability, JS_BENCH_MATRIX: JSON.stringify({ include: names.map(name => ({ case: name })) }) },
  });
  try {
    assert.equal(collect(["parse/fixture"]).status, 0);
    assert.equal(JSON.parse(readFileSync(join(directory, "comparison.json"), "utf8")).length, 1);
    assert.notEqual(collect(["parse/fixture"], "failure").status, 0);
    assert.notEqual(collect(["parse/fixture", "print/fixture"]).status, 0);
    assert.match(readFileSync(join(directory, "summary.md"), "utf8"), /incomplete/);
  } finally {
    rmSync(directory, { recursive: true });
  }
});


test("PR suite covers all operations while manual runs default to the full suite", () => {
  const selected = selectCases("representative");
  assert.equal(selected.length, 5);
  assert.equal(new Set(selected).size, 5);
  assert.deepEqual(selected.map(name => name.split("/")[0]).sort(), ["lang", "parse", "parse_full", "print", "source_type"]);
  assert(selected.every(name => cases.includes(name)));
  assert(!selected.some(name => /cal\.com|jquery/.test(name)));
  assert.deepEqual(selectCases(), cases);
  assert.deepEqual(selectCases("full", "parse/cal.com"), ["parse/cal.com.tsx"]);
  assert.throws(() => selectCases("unknown"));
  assert.throws(() => selectCases("representative", "cal.com"));
});


test("same-SHA control rejects disjoint ranges but permits overlapping ranges", () => {
  const overlap = compare([summary(1, 100, 99), summary(2, 102, 101), summary(3, 101, 103)]);
  assert.doesNotThrow(() => validateRepeatability(overlap));
  const touching = compare([summary(1, 100, 102), summary(2, 101, 103), summary(3, 102, 104)]);
  assert.doesNotThrow(() => validateRepeatability(touching));
  const lower = compare([summary(1, 100, 90), summary(2, 101, 91), summary(3, 102, 92)]);
  assert.throws(() => validateRepeatability(lower), /same-SHA ranges do not overlap: parse\/fixture/);
  const higher = compare([summary(1, 100, 110), summary(2, 101, 111), summary(3, 102, 112)]);
  assert.throws(() => validateRepeatability(higher), /same-SHA ranges do not overlap/);
  assert.throws(() => validateRepeatability([]), /no same-SHA comparisons/);
});
