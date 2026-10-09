// Compare with an ast.js snapshot plus its adjacent visitor-keys.js:
// node npm/core/benches/walk.bench.mjs /absolute/path/to/baseline/ast.js
// Parsing/cloning and validation are outside walk timings. The pipeline case
// includes parse, four walks and print. Each sample runs in an isolated process;
// variant order alternates to reduce warmup/thermal bias. No timing assertions.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { cpus } from "node:os";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { performance } from "node:perf_hooks";

const scenarios = ["no-hooks", "sparse", "dense", "edits", "four-passes", "pipeline"];
const median = (values) => [...values].sort((a, b) => a - b)[values.length >> 1];

if (process.argv[2] === "--worker") {
  const { walk } = await import(pathToFileURL(process.argv[3]));
  const { parseSync, printSync } = await import("../src-js/index.js");
  const scenario = process.argv[4];
  const source = scenario === "edits"
    ? Array.from({ length: 1000 }, (_, i) =>
      `function f${i}(x) { debugger; keep(); return x + 1; }`).join("\n")
    : readFileSync(new URL("../../../tasks/benchmark/files/jquery-1.9.1.js", import.meta.url), "utf8");
  const parsed = parseSync(source);
  assert.equal(parsed.diagnostics.filter((d) => d.severity === "error").length, 0);
  const original = parsed.program;
  let visits = 0;
  const hooks = scenario === "no-hooks" ? {} : scenario === "sparse" ? {
    FunctionDeclaration() { visits++; },
  } : scenario === "edits" ? {
    DebuggerStatement(node, ctx) { ctx.remove(); },
    Literal(node, ctx) {
      if (node.value === 1) ctx.replace({ ...node, value: 2 });
    },
    ExpressionStatement(node, ctx) {
      if (node.expression.type === "CallExpression" && node.expression.callee.name === "keep") {
        ctx.insertBefore({ type: "EmptyStatement" });
        ctx.insertAfter({ type: "EmptyStatement" });
      }
    },
  } : { enter() { visits++; } };
  const passes = scenario === "four-passes" || scenario === "pipeline" ? 4 : 1;
  const run = (ast) => {
    for (let i = 0; i < passes; i++) walk(ast, hooks);
  };
  function timed() {
    // Edits always start from an identical AST; cloning is not timed.
    const ast = scenario === "edits" ? structuredClone(original) : original;
    const start = performance.now();
    let result = ast;
    if (scenario === "pipeline") {
      const next = parseSync(source);
      run(next.program);
      result = printSync(next.program, undefined, next.meta).code;
    } else run(ast);
    return { ms: performance.now() - start, result };
  }
  for (let i = 0; i < 12; i++) timed();
  global.gc();
  visits = 0;
  const heapBefore = process.memoryUsage().heapUsed;
  let heapMax = heapBefore;
  const timings = [];
  let result;
  for (let i = 0; i < 25; i++) {
    const sample = timed();
    timings.push(sample.ms);
    result = sample.result;
    heapMax = Math.max(heapMax, process.memoryUsage().heapUsed);
  }
  const peakRssMiB = process.resourceUsage().maxRSS / 1024;
  const digest = createHash("sha256").update(JSON.stringify(result)).digest("hex");
  if (scenario === "edits") {
    assert.equal(result.body[0].body.body.length, 4);
    assert.equal(result.body[0].body.body[3].argument.right.value, 2);
  }
  console.log(JSON.stringify({
    ms: median(timings), minMs: Math.min(...timings), maxMs: Math.max(...timings),
    peakRssMiB, observedHeapMiB: heapMax / 1048576,
    heapBeforeMiB: heapBefore / 1048576, digest, visits,
  }));
} else {
  const baseline = process.argv[2];
  if (!baseline) throw new Error("Pass a baseline ast.js path (with adjacent visitor-keys.js)");
  const variants = {
    before: resolve(baseline),
    after: fileURLToPath(new URL("../src-js/ast.js", import.meta.url)),
  };
  const rows = [];
  for (const scenario of scenarios) {
    const samples = { before: [], after: [] };
    for (let round = 0; round < 3; round++) {
      const names = round % 2 ? ["after", "before"] : ["before", "after"];
      for (const name of names) {
        const child = spawnSync(process.execPath,
          ["--expose-gc", fileURLToPath(import.meta.url), "--worker", variants[name], scenario],
          { encoding: "utf8" });
        if (child.status !== 0) throw new Error(child.stderr || child.stdout);
        samples[name].push(JSON.parse(child.stdout));
      }
    }
    for (const sample of [...samples.before, ...samples.after]) {
      assert.equal(sample.digest, samples.before[0].digest, `${scenario}: output differs`);
      assert.equal(sample.visits, samples.before[0].visits, `${scenario}: visit count differs`);
    }
    const summary = {};
    for (const name of ["before", "after"]) {
      summary[name] = {
        ms: median(samples[name].map((s) => s.ms)),
        peakRssMiB: median(samples[name].map((s) => s.peakRssMiB)),
        observedHeapMiB: median(samples[name].map((s) => s.observedHeapMiB)),
      };
    }
    rows.push({ scenario, ...summary, speedup: summary.before.ms / summary.after.ms, samples });
    console.error(`${scenario}: ${summary.before.ms.toFixed(2)} -> ${summary.after.ms.toFixed(2)} ms`);
  }
  console.log(JSON.stringify({
    node: process.version, platform: `${process.platform}/${process.arch}`, cpu: cpus()[0].model,
    samples: "3 processes per variant/scenario; 12 warmups + 25 iterations each",
    memory: "process peak RSS and maximum heap sampled after each iteration; includes setup; no separate client/server",
    rows,
  }, null, 2));
}
