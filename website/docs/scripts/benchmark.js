#!/usr/bin/env node
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { cpus, platform, release, tmpdir } from "node:os";
import { parseArgs } from "node:util";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../..",
);
const swcCoreVersion = "82.1.0";
const fastStripVersion = "59.0.1";
const runs = 200;
const warmup = 20;
const nativeIterations = 10;
const jsRuns = 2;
const jsTimeMs = 1000;
// Linux only: set BENCH_CPU to an available CPU, preferably a performance core.
const cpu = process.env.BENCH_CPU ?? "0";
const { values } = parseArgs({ options: { output: { type: "string" } } });
const output = path.resolve(root, values.output ?? "website/docs/data/benchmarks.json");
const parseFiles = ["typescript.js", "effect-schema-ast.ts"];
const benchmarks = [
  {
    title: "Rust parse",
    type: "native",
    operation: "parse",
    files: parseFiles,
  },
  {
    title: "Isolated declaration",
    type: "native",
    operation: "isolated-declaration",
    files: ["hono-types.ts"],
  },
  {
    title: "Type strip",
    type: "native",
    operation: "type-strip",
    files: ["effect-schema-ast.ts"],
  },
  {
    title: "JS parser",
    type: "js",
    files: parseFiles,
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
  const jsNames = [`@swc-next/core@${commit}`, `@swc/core@${coreVersion}`];
  const hyperfineVersion = measure("hyperfine", ["--version"]);
  build();

  // Hyperfine executes these arguments directly, without a shell.
  const quote = (value) => `'${String(value).replaceAll("'", "'\\''")}'`;
  const report = {
    formatVersion: 1,
    source: "hyperfine",
    metadata: {
      commit,
      generatedAt: new Date().toISOString(),
      hyperfineVersion,
      nodeVersion: process.version,
      platform: platform(),
      release: release(),
      architecture: process.arch,
      cpuModel: cpus()[0]?.model ?? null,
      cpuAffinity: process.platform === "linux" ? cpu : null,
    },
    benchmarks: [],
  };
  const temporaryDirectory = mkdtempSync(path.join(tmpdir(), "swc-next-benchmark-"));
  try {
    for (const { title, type, operation, files } of benchmarks) {
      const isJs = type === "js";
      for (const file of files) {
        console.error(`${title}: ${file}`);
        const input = path.join("tasks/benchmark/files", file);
        const commands = ["swc-next", "swc"].map((engine) =>
          isJs
            ? [
                process.execPath,
                `tasks/benchmark/bin/${engine}.js`,
                input,
              ]
            : [
                `target/release/${engine}`,
                input,
                operation,
                nativeIterations,
              ],
        );
        const swcName =
          operation === "type-strip"
            ? `swc_ts_fast_strip@${fastStripVersion}`
            : `swc_core@${swcCoreVersion}`;
        let iterations = nativeIterations;
        let names = [`swc-next@${commit}`, swcName].map(
          (name) => `${name}; ${nativeIterations} iterations`,
        );
        if (isJs) {
          const timings = commands.map(([command, ...args]) =>
            Number(measure(command, [...args, "--calibrate"])),
          );
          assert(
            timings.every((ms) => Number.isFinite(ms) && ms > 0),
            "Invalid JS calibration",
          );
          iterations = Math.max(
            5,
            Math.ceil(jsTimeMs / Math.min(...timings)),
          );
          for (const command of commands) command.push(String(iterations));
          names = jsNames.map(
            (name) => `${name}; ${iterations} parses + 1 validation + 5 warmups`,
          );
          console.error(
            `JS batch: ${iterations} parses per engine; calibration ${timings.map((ms) => ms.toFixed(3)).join(" / ")} ms/parse`,
          );
        }
        // Each JS process warms its own V8 state; hyperfine warmups would use
        // separate processes and would not warm the measured parser.
        const sessions = [[0, 1], [1, 0]].map((order, sessionIndex) => {
          const exportPath = path.join(temporaryDirectory, `${sessionIndex}.json`);
          const log = measure("hyperfine", [
            "--style",
            "basic",
            "--shell",
            "none",
            "--warmup",
            isJs ? 0 : warmup,
            "--runs",
            isJs ? jsRuns : runs,
            "--metrics",
            "time_wall_clock:ms,memory_peak_resident:MiB",
            "--export-json",
            exportPath,
            ...order.flatMap((index) => [
              "--command-name",
              names[index],
              commands[index].map(quote).join(" "),
            ]),
          ]);
          console.error(log);
          const result = JSON.parse(readFileSync(exportPath, "utf8"));
          assert.equal(result.schema_version, 2, "Hyperfine 2 JSON is required");
          assert.equal(result.results.length, commands.length);
          for (const index of order) {
            const measured = result.results.find((entry) => entry.name === names[index]);
            assert(measured, `Missing result: ${names[index]}`);
            assert.equal(measured.measurements.length, isJs ? jsRuns : runs);
            assert(measured.measurements.every((sample) => sample.exit_code === 0));
          }
          return { order: order.map((index) => names[index]), report: result };
        });
        report.benchmarks.push({
          id: `${isJs ? "js-parse" : operation}-${file.replaceAll(".", "-")}`,
          title,
          type,
          operation: isJs ? "parse" : operation,
          file,
          iterations,
          runs: isJs ? jsRuns : runs,
          warmup: isJs ? 0 : warmup,
          processWarmups: isJs ? 5 : 0,
          validations: isJs ? 1 : 0,
          sessions,
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
