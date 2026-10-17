#!/usr/bin/env node
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
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
const output = path.join(root, "website/docs/Benchmark.md");
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
  measure("hyperfine", ["--version"]);
  build();

  // Hyperfine executes these arguments directly, without a shell.
  const quote = (value) => `'${String(value).replaceAll("'", "'\\''")}'`;
  const lines = ["# Benchmark", ""];
  for (const { title, type, operation, files } of benchmarks) {
    lines.push(`## ${title}`, "");
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
        const iterations = Math.max(
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
      lines.push(`### ${file}`, "");
      const [result, reverseResult] = [[0, 1], [1, 0]].map((order) =>
        measure("hyperfine", [
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
          ...order.flatMap((index) => [
            "--command-name",
            names[index],
            commands[index].map(quote).join(" "),
          ]),
        ]),
      );
      // Keep the reverse-order check in the run log.
      console.error(reverseResult);
      lines.push(
        "```text",
        result.replace(/[ \t]+$/gm, ""),
        "```",
        "",
      );
    }
  }
  writeFileSync(output, lines.join("\n"));
  console.error(`Wrote ${output}`);
}

main();
