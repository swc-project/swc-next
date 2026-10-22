#!/usr/bin/env node
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { parseArgs } from "node:util";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../..",
);
const cargoPackages = [...readFileSync(path.join(root, "Cargo.lock"), "utf8")
  .matchAll(/\[\[package\]\]\r?\nname = "([^"]+)"\r?\nversion = "([^"]+)"/g)];
function crateVersion(name) {
  const packages = cargoPackages.filter(([, crate]) => crate === name);
  assert.equal(packages.length, 1, `Expected one locked version of ${name}`);
  return packages[0][2];
}
const swcCoreVersion = crateVersion("swc_core");
const fastStripVersion = crateVersion("swc_ts_fast_strip");
// Linux only: set BENCH_CPU to an available CPU, preferably a performance core.
const cpu = process.env.BENCH_CPU ?? "0";
const { values } = parseArgs({ options: { output: { type: "string" } } });
const output = path.resolve(root, values.output ?? "website/docs/data/benchmarks.json");
// Counts follow engine order: SWC Next, SWC, then Babel (JS only).
// Fixed from observed timings to target roughly five seconds per engine and case.
const benchmarks = [
  {
    title: "Rust parse",
    type: "native",
    operation: "parse",
    files: [
      { file: "typescript.js", runs: [250, 50] },
      { file: "effect-schema-ast.ts", runs: [5000, 2500] },
    ],
  },
  {
    title: "Isolated declaration",
    type: "native",
    operation: "isolated-declaration",
    files: [{ file: "hono-types.ts", runs: [3600, 2200] }],
  },
  {
    title: "Type strip",
    type: "native",
    operation: "type-strip",
    files: [{ file: "effect-schema-ast.ts", runs: [4300, 2100] }],
  },
  {
    title: "JS parser",
    type: "js",
    operation: "parse",
    files: [
      { file: "typescript.js", rounds: [12, 2, 5], iterations: [1, 1, 1] },
      { file: "effect-schema-ast.ts", rounds: [170, 200, 150], iterations: [10, 1, 4] },
    ],
  },
  {
    title: "JS parse + walk",
    type: "js",
    operation: "parse-walk",
    // SWC's JS Visitor does not implement TypeScript type traversal.
    files: [
      { file: "typescript.js", rounds: [16, 2, 2], iterations: [1, 1, 1] },
      { file: "jquery-1.9.1.js", rounds: [170, 80, 140], iterations: [4, 1, 1] },
    ],
  },
];

function run(command, args, options = {}) {
  const result = spawnSync(command, args.map(String), {
    cwd: root,
    encoding: "utf8",
    ...options,
  });
  if (result.error) throw result.error;
  if (result.status !== 0)
    throw new Error(
      `${command} failed (${result.signal ?? result.status})\n${result.stderr ?? ""}`,
    );
  return result.stdout?.trimEnd() ?? "";
}

function measure(command, args) {
  if (process.platform === "linux") {
    return run("taskset", ["--cpu-list", cpu, command, ...args]);
  }
  return run(command, args);
}

function build() {
  run(
    "cargo",
    ["build", "--locked", "-p", "benchmark", "--release", "--bins"],
    {
      stdio: "inherit",
    },
  );
  run("pnpm", ["--filter", "@swc-next/core", "build", "--release"], {
    stdio: "inherit",
  });
}

function main() {
  const commit = run("git", [
    "describe",
    "--always",
    "--abbrev=40",
    "--dirty",
    "--exclude=*",
  ]);
  const require = createRequire(path.join(root, "tasks/benchmark/package.json"));
  const coreVersion = require("@swc/core/package.json").version;
  const babelVersion = require("@babel/parser/package.json").version;
  const traverseVersion = require("@babel/traverse/package.json").version;
  const jsNames = [`@swc-next/core@${commit}`, `@swc/core@${coreVersion}`, `@babel/parser@${babelVersion}`];
  build();
  console.error(`Node ${process.version}; release build; revision ${commit}`);
  const report = [];
  const temporaryDirectory = mkdtempSync(path.join(tmpdir(), "swc-next-benchmark-"));
  try {
    for (const { title, type, operation, files } of benchmarks) {
      const isJs = type === "js";
      for (const { file, runs, rounds, iterations } of files) {
        console.error(`${title}: ${file}`);
        const input = path.join("tasks/benchmark/files", file);
        const engines = isJs ? ["swc-next", "swc", "babel"] : ["swc-next", "swc"];
        const commands = engines.map((engine) =>
          isJs
            ? [
                process.execPath,
                `tasks/benchmark/bin/${engine}.js`,
                input,
                operation,
              ]
            : [
                `target/release/${engine}`,
                file,
                operation,
              ],
        );
        const swcName =
          operation === "type-strip"
            ? `swc_ts_fast_strip@${fastStripVersion}`
            : `swc_core@${swcCoreVersion}`;
        const names = isJs
          ? jsNames.map((name, index) =>
              operation === "parse-walk" && index === 2
                ? `${name} + @babel/traverse@${traverseVersion}`
                : name,
            )
          : [`swc-next@${commit}`, swcName];
        let results;
        if (isJs) {
          const trials = commands.map(() => []);
          const trialCount = 3;
          for (let trial = 0; trial < trialCount; trial++) {
            // Rotate the starting engine between trials.
            for (let offset = 0; offset < commands.length; offset++) {
              const index = (trial + offset) % commands.length;
              const [command, ...args] = commands[index];
              const result = JSON.parse(measure(command, [...args, rounds[index], iterations[index]]));
              assert(Number.isFinite(result.bestMs) && result.bestMs > 0);
              trials[index].push(result.bestMs);
              console.error(`  ${names[index]} trial ${trial + 1}/${trialCount}: ${result.bestMs.toFixed(3)} ms (${rounds[index]} rounds × ${iterations[index]} iterations)`);
            }
          }
          results = trials.map((values, index) => {
            values.sort((a, b) => a - b);
            const middle = Math.floor(values.length / 2);
            const median = values.length % 2 ? values[middle] : (values[middle - 1] + values[middle]) / 2;
            return {
              name: names[index],
              time: median,
            };
          });
        } else {
          results = commands.map(([command, ...args], index) => {
            console.error(`  ${names[index]}`);
            const exportPath = path.join(temporaryDirectory, "result.json");
            const quote = (value) => `'${String(value).replaceAll("'", "'\\''")}'`;
            measure("hyperfine", [
              "--style", "basic",
              "--shell", "none",
              "--warmup", 10,
              "--runs", runs[index],
              "--metrics", "time_wall_clock:ms,memory_peak_resident:MiB",
              "--export-json", exportPath,
              [command, ...args].map(quote).join(" "),
            ]);
            const result = JSON.parse(readFileSync(exportPath, "utf8"));
            assert.equal(result.schema_version, 2, "Hyperfine 2 JSON is required");
            const [entry] = result.results;
            assert.equal(entry.measurements.length, runs[index]);
            assert(entry.measurements.every((sample) => sample.exit_code === 0));
            const summary = entry.summary;
            return {
              name: names[index],
              time: summary.time_wall_clock.mean * 1000,
              memory: summary.memory_peak_resident.mean / 1048576,
            };
          });
        }
        report.push({
          id: `${isJs ? `js-${operation}` : operation}-${file.replaceAll(".", "-")}`,
          title,
          type,
          file,
          results,
        });
      }
    }
    mkdirSync(path.dirname(output), { recursive: true });
    const pendingOutput = `${output}.tmp`;
    try {
      writeFileSync(pendingOutput, `${JSON.stringify(report, null, 2)}\n`);
      renameSync(pendingOutput, output);
    } finally {
      rmSync(pendingOutput, { force: true });
    }
    console.error(`Wrote ${output}`);
  } finally {
    rmSync(temporaryDirectory, { recursive: true, force: true });
  }
}

main();
