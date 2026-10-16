import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export const apiNames = ["langFromPath", "parseSync", "printSync", "sourceTypeFromPath"];
export const fixtures = ["underscore-1.5.2.js", "jquery-1.9.1.js", "astro-config.ts", "cal.com.tsx"];
export const extensions = ["js", "ts", "tsx", "mjs", "cjs", "unknown"];

export const cases = [
  ...[...fixtures].reverse().flatMap(name => ["parse", "print"].map(operation => `${operation}/${name}`)),
  ...["lang", "source_type"].flatMap(operation => extensions.map(name => `${operation}/${name}`)),
];

// Keep PR coverage of all four operations without the large profiling fixtures.
export const representativeCases = [
  "parse/underscore-1.5.2.js",
  "print/underscore-1.5.2.js",
  "lang/tsx",
  "source_type/mjs",
];

export function selectCases(suite = "full", filter = "") {
  assert(["representative", "full"].includes(suite), "unknown benchmark suite");
  const selected = (suite === "representative" ? representativeCases : cases)
    .filter(name => name.includes(filter));
  assert(selected.length > 0, "JS_BENCH_FILTER matched no cases");
  return selected;
}

// Reading enumerable properties evaluates lazy AST getters without JSON conversion.
export function touch(value) {
  if (value === null || typeof value !== "object") return 1;
  let count = 1;
  for (const key of Object.keys(value)) count += touch(value[key]);
  return count;
}

export function prepare(api, operation, name, source) {
  assert.deepEqual(Object.keys(api).sort(), apiNames);
  if (operation === "lang" || operation === "source_type") {
    assert(extensions.includes(name), "unknown extension case");
    const fn = operation === "lang" ? api.langFromPath : api.sourceTypeFromPath;
    return { iterations: 10_000, run: () => fn(`input.${name}`), validate: value => assert.equal(typeof value, "string") };
  }
  assert(fixtures.includes(name), "unknown fixture");
  assert(["parse", "print"].includes(operation), "unknown operation");
  const options = { lang: name.endsWith(".tsx") ? "tsx" : name.endsWith(".ts") ? "ts" : "js" };
  const parsed = api.parseSync(source, options);
  assert.equal(parsed.diagnostics.length, 0);
  if (operation === "print") {
    // Materialize input outside the measured region, including lazy strings.
    touch(parsed.program);
    return { iterations: 10, run: () => api.printSync(parsed.program, {}, parsed.meta), validate: value => assert(value.code.length > 0) };
  }
  return { iterations: 10, run: () => touch(api.parseSync(source, options).program), validate: value => assert(value > 1) };
}

export function execute(workload, measure, gc) {
  let result;
  for (let i = 0; i < 100; i++) result = workload.run();
  workload.validate(result);
  result = undefined;
  gc();
  // Retain return values until after collection; validate outside the measured region.
  const results = new Array(workload.iterations);
  measure(() => {
    for (let i = 0; i < workload.iterations; i++) results[i] = workload.run();
  });
  for (const value of results) workload.validate(value);
}

async function main() {
  assert(process.execArgv.includes("--jitless"), "benchmark requires --jitless");
  const [entry, operation, name] = process.argv.slice(2);
  assert(entry && operation && name, "usage: runner.mjs <entry> <operation> <case>");
  const harness = process.env.JS_BENCH_HARNESS ?? dirname(fileURLToPath(import.meta.url));
  const api = await import(pathToFileURL(entry).href);
  const source = fixtures.includes(name) ? readFileSync(join(harness, "files", name), "utf8") : undefined;
  const workload = prepare(api, operation, name, source);
  const { measure } = createRequire(import.meta.url)(join(harness, "measure.node"));
  execute(workload, measure, globalThis.gc);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await main();
