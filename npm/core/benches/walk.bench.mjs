// Compare with an ast.js snapshot plus its adjacent visitor-keys.js:
// node npm/core/benches/walk.bench.mjs /absolute/path/to/baseline/ast.js
// Add --handwritten-edits for a three-way task-specific editing comparison.
// Parsing/cloning and validation are outside walk timings. The pipeline case
// includes parse, four walks and print. Each sample runs in an isolated process;
// variant order alternates to reduce warmup/thermal bias. No timing assertions.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { cpus } from "node:os";
import { resolve, extname } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { performance } from "node:perf_hooks";

const scenarios = ["no-hooks", "sparse", "dense", "context", "edits", "four-passes", "pipeline"];
const median = (values) => [...values].sort((a, b) => a - b)[values.length >> 1];

if (process.argv[2] === "--worker") {
  const handwritten = process.argv[3] === "handwritten";
  const { walk } = handwritten ? {} : await import(pathToFileURL(process.argv[3]));
  const { editByHand, traceByHand } = handwritten
    ? await import("./handwritten-edit.mjs")
    : {};
  const { parseSync, printSync } = await import("../src-js/index.js");
  const scenario = process.argv[4];
  if (handwritten) assert.equal(scenario, "edits");
  const fixture = process.argv[5] || fileURLToPath(new URL("../../../tasks/benchmark/files/jquery-1.9.1.js", import.meta.url));
  const parserOptions = { lang: scenario === "edits" ? "js" : extname(fixture).slice(1) };

  const source = scenario === "edits"
    ? Array.from({ length: 1000 }, (_, i) =>
      `function f${i}(x) { debugger; keep(); return x + 1; }`).join("\n")
    : readFileSync(fixture, "utf8");
  const parsed = parseSync(source, parserOptions);
  assert.equal(parsed.diagnostics.filter((d) => d.severity === "error").length, 0);
  const original = parsed.program;
  const batch = source.length < 5000 ? 1000 : 1;
  let visits = 0;
  const hooks = scenario === "no-hooks" ? {} : scenario === "sparse" ? {
    FunctionDeclaration() { visits++; },
  } : scenario === "context" ? {
    enter(node, ctx) {
      assert.equal(ctx.node, node);
      visits += (ctx.parent === null ? 0 : 1) + (ctx.index === null ? 0 : 1) + (ctx.key?.length ?? 0);
    },
  } : scenario === "edits" ? {
    DebuggerStatement(node, ctx) { visits++; ctx.remove(); },
    Literal(node, ctx) {
      visits++;
      if (node.value === 1) ctx.replace({ ...node, value: 2 });
    },
    ExpressionStatement(node, ctx) {
      visits++;
      if (node.expression.type === "CallExpression" && node.expression.callee.name === "keep") {
        ctx.insertBefore({ type: "EmptyStatement" });
        ctx.insertAfter({ type: "EmptyStatement" });
      }
    },
  } : { enter() { visits++; } };
  const passes = scenario === "four-passes" || scenario === "pipeline" ? 4 : 1;
  const run = (ast) => {
    for (let i = 0; i < passes; i++) {
      if (handwritten) visits += editByHand(ast);
      else walk(ast, hooks);
    }
  };
  function timed() {
    // Edits always start from an identical AST; cloning is not timed.
    const ast = scenario === "edits" ? structuredClone(original) : original;
    const start = performance.now();
    let result = ast;
    for (let iteration = 0; iteration < batch; iteration++) {
      if (scenario === "pipeline") {
        const next = parseSync(source, parserOptions);
        run(next.program);
        result = printSync(next.program, undefined, next.meta).code;
      } else run(ast);
    }
    return { ms: (performance.now() - start) / batch, result };
  }
  global.gc();
  const warmupUntil = performance.now() + 300;
  let warmups = 0;
  do {
    timed();
    warmups++;
  } while (warmups < 12 || performance.now() < warmupUntil);
  if (process.env.WALK_BENCH_GC_AFTER_WARMUP === "1") global.gc();
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
  const trace = createHash("sha256");
  if (handwritten) {
    for (const row of traceByHand(result)) trace.update(JSON.stringify(row));
  } else {
    walk(typeof result === "string" ? original : result, {
      enter(node, ctx) {
        trace.update(JSON.stringify([node.type, node.start, node.end, ctx.key, ctx.index]));
      },
    });
  }
  const traceDigest = trace.digest("hex");
  const digest = createHash("sha256").update(JSON.stringify(result)).digest("hex");
  if (scenario === "edits") {
    assert.equal(result.body[0].body.body.length, 4);
    assert.equal(result.body[0].body.body[3].argument.right.value, 2);
  }
  console.log(JSON.stringify({
    warmups, batch, ms: median(timings), minMs: Math.min(...timings), maxMs: Math.max(...timings),
    peakRssMiB, observedHeapMiB: heapMax / 1048576,
    heapBeforeMiB: heapBefore / 1048576, digest, traceDigest, visits,
  }));
} else {
  const handwritten = process.argv.includes("--handwritten-edits");
  const args = process.argv.slice(2).filter((arg) => arg !== "--handwritten-edits");
  const baseline = args[0];
  const fixture = args[1] ? resolve(args[1]) : "";
  if (!baseline) throw new Error("Pass a baseline ast.js path (with adjacent visitor-keys.js)");
  const variants = {
    before: resolve(baseline),
    after: fileURLToPath(new URL("../src-js/ast.js", import.meta.url)),
  };
  if (handwritten) variants.handwritten = "handwritten";
  const variantNames = Object.keys(variants);
  const rows = [];
  for (const scenario of handwritten ? ["edits"] : scenarios) {
    const samples = Object.fromEntries(variantNames.map((name) => [name, []]));
    for (let round = 0; round < 3; round++) {
      // In the three-way comparison each variant runs first, second and third.
      const names = handwritten
        ? [...variantNames.slice(round), ...variantNames.slice(0, round)]
        : round % 2 ? ["after", "before"] : ["before", "after"];
      for (const name of names) {
        const child = spawnSync(process.execPath,
          ["--expose-gc", fileURLToPath(import.meta.url), "--worker", variants[name], scenario, fixture],
          { encoding: "utf8" });
        if (child.status !== 0) throw new Error(child.stderr || child.stdout);
        samples[name].push(JSON.parse(child.stdout));
      }
    }
    for (const sample of Object.values(samples).flat()) {
      assert.equal(sample.traceDigest, samples.before[0].traceDigest, `${scenario}: visit order/positions differ`);
      assert.equal(sample.digest, samples.before[0].digest, `${scenario}: output differs`);
      assert.equal(sample.visits, samples.before[0].visits, `${scenario}: visit count differs`);
    }
    const summary = {};
    for (const name of variantNames) {
      summary[name] = {
        ms: median(samples[name].map((s) => s.ms)),
        peakRssMiB: median(samples[name].map((s) => s.peakRssMiB)),
        observedHeapMiB: median(samples[name].map((s) => s.observedHeapMiB)),
      };
    }
    rows.push({ scenario, ...summary, speedup: summary.before.ms / summary.after.ms, samples });
    console.error(`${scenario}: ${variantNames.map((name) => `${name}=${summary[name].ms.toFixed(3)} ms`).join(", ")}`);
  }
  console.log(JSON.stringify({
    comparison: handwritten ? "libraries and task-specific handwritten editing" : "two walkers",
    gcAfterWarmup: process.env.WALK_BENCH_GC_AFTER_WARMUP === "1",
    fixture: handwritten ? "synthetic functions (1000)" : fixture || "tasks/benchmark/files/jquery-1.9.1.js",
    baselineModule: variants.before, candidateModule: variants.after,
    node: process.version, platform: `${process.platform}/${process.arch}`, cpu: cpus()[0].model,
    samples: "3 processes per variant/scenario; at least 12 warmups / 300 ms + 25 iterations each",
    memory: "process peak RSS and maximum heap sampled after each iteration; includes setup; no separate client/server",
    rows,
  }, null, 2));
}
