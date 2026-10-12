// Walker-only comparison using the real application adapters in visitor-benchmark.
// node npm/core/benches/visitor.bench.mjs <benchmark-directory> <baseline-ast.js> <results.json>
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { cpus } from "node:os";
import { dirname, resolve } from "node:path";
import { spawnSync, execFileSync } from "node:child_process";
import { pathToFileURL, fileURLToPath } from "node:url";
import { performance } from "node:perf_hooks";

const candidate = fileURLToPath(new URL("../src-js/ast.js", import.meta.url));
const median = (values) => {
  const sorted = values.slice().sort((a, b) => a - b);
  return (sorted[(sorted.length - 1) >> 1] + sorted[sorted.length >> 1]) / 2;
};
const hash = (value) => createHash("sha256").update(value).digest("hex");
const moduleHashes = (file) => Object.fromEntries(
  ["ast.js", "child-visitors.js", "visitor-keys.js"].map((name) =>
    [name, hash(readFileSync(resolve(dirname(file), name)))])
);

if (process.argv[2] === "--worker") {
  const [, bench, modulePath, scenario, kind] = process.argv.slice(2);
  const adapter = resolve(bench, `src/${scenario}/plugins/swc-next.mjs`);
  const source = readFileSync(adapter, "utf8");
  let replacements = 0;
  // Load the exact business adapter, changing only its walker import. Resolve
  // other imports against the adapter's original location, not the data URL.
  const redirected = source.replace(/from\s+(['"])([^'"]+)\1/g, (_, quote, specifier) => {
    if (specifier.endsWith("/vendor/swc-next/ast.js")) {
      replacements++;
      return `from ${JSON.stringify(pathToFileURL(modulePath).href)}`;
    }
    return `from ${JSON.stringify(specifier.startsWith(".")
      ? pathToFileURL(resolve(dirname(adapter), specifier)).href : specifier)}`;
  });
  assert.equal(replacements, 1);
  const plugin = await import(`data:text/javascript;base64,${Buffer.from(redirected).toString("base64")}`);
  let fixtures, prepare, run, validate;
  if (scenario === "i18n") {
    const { loadCorpus } = await import(pathToFileURL(resolve(bench, "src/i18n/corpus.mjs")));
    const { engine } = await import(pathToFileURL(resolve(bench, "src/i18n/engine.mjs")));
    const corpus = loadCorpus();
    const keys = new Set(corpus.keys);
    const implementation = await engine("swc-next", kind);
    fixtures = corpus.files.map((file) => ({
      filename: file.filename,
      source: kind === "original" ? file.original : file.javascript,
      expected: kind === "original" ? file.expectedOriginal : file.expectedJavascript,
    }));
    prepare = (file) => implementation.prepare(file);
    run = (ast) => plugin.collect(ast, keys);
    validate = (result, file) => {
      assert.deepEqual(result, file.expected);
      return hash(JSON.stringify(result));
    };
  } else {
    const workload = await import(pathToFileURL(resolve(bench, `src/${scenario}/workload.mjs`)));
    const { load } = await import(pathToFileURL(resolve(bench, `src/${scenario}/execute.mjs`)));
    const implementation = await load({ id: "swc-next" });
    fixtures = workload.corpus(kind);
    prepare = implementation.prepare;
    run = (ast, file) => plugin.run(ast, scenario === "rstest-coverage" ? file : file.filename);
    validate = implementation.validate;
  }
  const batch = scenario === "i18n" ? 2 : 8;
  const outputHashes = new Set();
  const inputs = fixtures.map(prepare);
  const inputHash = hash(JSON.stringify(inputs));
  const { walk } = await import(pathToFileURL(modulePath));
  const trace = [];
  const record = (phase) => (node, ctx) => {
    trace.push([
      phase, node.type, node.start, node.end, ctx.parent?.type, ctx.key, ctx.index,
      ctx.ancestors().map((parent) => parent.type),
    ]);
  };
  for (const input of inputs) walk(input, { enter: record("enter"), leave: record("leave") });
  const traceHash = hash(JSON.stringify(trace));
  function sample(check) {
    // Every edit starts from a fresh AST. Batch construction and validation are
    // outside the timer; no iteration re-edits an already transformed tree.
    const trees = Array.from({ length: batch }, () => fixtures.map(prepare));
    const results = [];
    const start = performance.now();
    for (const group of trees) {
      for (let i = 0; i < fixtures.length; i++) results.push(run(group[i], fixtures[i]));
    }
    const ms = (performance.now() - start) / batch;
    if (check) {
      const hashes = results.map((result, i) => validate(result, fixtures[i % fixtures.length]));
      outputHashes.add(hash(hashes.join(",")));
    }
    return ms;
  }
  sample(true);
  global.gc?.();
  let warmups = 0;
  const start = performance.now();
  do { sample(false); warmups++; } while (warmups < 12 || performance.now() - start < 500);
  const samples = Array.from({ length: 25 }, () => sample(true));
  assert.equal(outputHashes.size, 1);
  console.log(JSON.stringify({
    scenario, kind, samples, median: median(samples), warmups, batch,
    adapterHash: hash(source), sourceHash: hash(fixtures.map((file) => file.source).join("\n")),
    inputHash, traceHash, outputHash: [...outputHashes][0],
  }));
} else {
  const [benchArg, baselineArg, output] = process.argv.slice(2);
  if (!benchArg || !baselineArg || !output) throw Error("Expected benchmark directory, baseline ast.js and output JSON path");
  const bench = resolve(benchArg), baseline = resolve(baselineArg);
  const tracks = [
    ["styled-components", "dense"], ["styled-components", "sparse"],
    ["remove-debugger", "dense"], ["remove-debugger", "sparse"],
    ["rstest-coverage", "runtime"], ["rstest-coverage", "tooling"],
    ["i18n", "original"], ["i18n", "javascript"],
  ];
  const report = {
    date: new Date().toISOString(), node: process.version, cpu: cpus()[0].model,
    platform: `${process.platform}/${process.arch}`, repetitions: 3, samples: 25,
    benchmarkCommit: execFileSync("git", ["rev-parse", "HEAD"], { cwd: bench, encoding: "utf8" }).trim(),
    adapterDependencies: JSON.parse(readFileSync(resolve(bench, "package.json"))).dependencies,
    sourceHashes: { baseline: moduleHashes(baseline), candidate: moduleHashes(candidate), harness: hash(readFileSync(import.meta.filename)) },
    runs: [],
  };
  const signatures = new Map();
  for (let round = 0; round < 3; round++) {
    const ordered = round % 2 ? tracks.slice().reverse() : tracks;
    for (const [scenario, kind] of ordered) {
      for (const id of round % 2 ? ["candidate", "baseline"] : ["baseline", "candidate"]) {
        const child = spawnSync(process.execPath, ["--expose-gc", import.meta.filename, "--worker", bench,
          id === "baseline" ? baseline : candidate, scenario, kind], { encoding: "utf8", timeout: 120000 });
        if (child.status !== 0) throw Error(child.stderr || child.stdout);
        const row = JSON.parse(child.stdout);
        const key = `${scenario}/${kind}`;
        const signature = [row.adapterHash, row.sourceHash, row.inputHash, row.traceHash, row.outputHash];
        if (signatures.has(key)) assert.deepEqual(signature, signatures.get(key));
        else signatures.set(key, signature);
        report.runs.push({ round, id, ...row });
        writeFileSync(output, JSON.stringify(report, null, 2) + "\n");
        console.log(round + 1, key, id, row.median.toFixed(3));
      }
    }
  }
  for (const [scenario, kind] of tracks) {
    const result = (id) => median(report.runs.filter((row) => row.id === id && row.scenario === scenario && row.kind === kind).map((row) => row.median));
    const before = result("baseline"), after = result("candidate");
    console.log(scenario, kind, before.toFixed(3), after.toFixed(3), `${((after / before - 1) * 100).toFixed(1)}%`);
  }
}
