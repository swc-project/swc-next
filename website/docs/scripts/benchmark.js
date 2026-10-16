#!/usr/bin/env node
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../..",
);
const work = path.join(root, "target/website-benchmark");
const files = ["typescript.js", "effect-schema-ast.ts"];
const features = [
  ["Rust parse", "parse"],
  ["Isolated declaration", "isolated-declaration"],
  ["Type strip", "type-strip"],
  ["JS parser", "parse"],
];

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
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

function build(swc) {
  const collector = path.join(work, "cli");
  mkdirSync(collector, { recursive: true });
  const crates = [
    "allocator",
    "ecma_ast",
    "ecma_parser",
    "ecma_codegen",
    "ecma_semantic",
    "isolated_declarations",
    "type_strip",
  ];
  const dependencies = crates.map(
    (name) =>
      `swc_next_${name} = { path = ${JSON.stringify(path.join(root, "crates", `swc_${name}`))} }`,
  );
  dependencies.push(
    `swc_core = { path = ${JSON.stringify(path.join(swc, "crates/swc_core"))}, features = ["common", "ecma_ast", "ecma_parser", "ecma_parser_typescript", "ecma_codegen", "ecma_transforms", "ecma_visit", "typescript"] }`,
  );
  dependencies.push(
    `swc_ts_fast_strip = { path = ${JSON.stringify(path.join(swc, "crates/swc_ts_fast_strip"))} }`,
  );
  const bins = [
    ["swc-next", "swc_next.rs"],
    ["swc", "swc.rs"],
  ].map(
    ([name, file]) =>
      `[[bin]]\nname = "${name}"\npath = ${JSON.stringify(path.join(root, "tasks/benchmark/bin", file))}`,
  );
  const manifest = path.join(collector, "Cargo.toml");
  writeFileSync(
    manifest,
    `[workspace]\n[package]\nname = "benchmark-cli"\nversion = "0.0.0"\nedition = "2024"\npublish = false\n${bins.join("\n")}\n[dependencies]\n${dependencies.join("\n")}\n[profile.release]\nopt-level = 3\nlto = "fat"\ncodegen-units = 1\npanic = "abort"\nstrip = "symbols"\n`,
  );
  if (!existsSync(path.join(collector, "Cargo.lock"))) {
    copyFileSync(
      path.join(swc, "Cargo.lock"),
      path.join(collector, "Cargo.lock"),
    );
  }
  const env = { ...process.env, RUSTFLAGS: "", CARGO_TARGET_DIR: work };
  run("cargo", ["build", "--release", "--manifest-path", manifest, "-j", "6"], {
    env,
    stdio: "inherit",
  });
  run("pnpm", ["--filter", "@swc-next/core", "build", "--release"], {
    stdio: "inherit",
  });
}

function main() {
  const options = {
    "--swc-dir": path.resolve(root, "../swc"),
    "--runs": "50",
    "--warmup": "20",
    "--js-runs": "3",
    "--js-time-ms": "1000",
    "--cpu": undefined,
    "--output": path.join(root, "website/docs/Benchmark.md"),
  };
  const args = process.argv.slice(2);
  if (args.includes("--help")) {
    console.log(
      "Usage: node website/docs/scripts/benchmark.js [--swc-dir ../swc] [--runs 50] [--warmup 20] [--js-runs 3] [--js-time-ms 1000] [--cpu N] [--output website/docs/Benchmark.md]",
    );
    return;
  }
  for (let i = 0; i < args.length; i += 2) {
    assert(
      Object.hasOwn(options, args[i]) && args[i + 1],
      `Invalid option: ${args[i]}`,
    );
    options[args[i]] = args[i + 1];
  }
  assert(
    Number.isInteger(Number(options["--runs"])) &&
      Number(options["--runs"]) >= 2,
    "--runs must be at least 2",
  );
  assert(
    Number.isInteger(Number(options["--warmup"])) &&
      Number(options["--warmup"]) >= 0,
    "--warmup must be non-negative",
  );
  assert(
    Number.isInteger(Number(options["--js-runs"])) &&
      Number(options["--js-runs"]) >= 2,
    "--js-runs must be at least 2",
  );
  assert(
    Number.isInteger(Number(options["--js-time-ms"])) &&
      Number(options["--js-time-ms"]) > 0,
    "--js-time-ms must be positive",
  );
  const swc = path.resolve(options["--swc-dir"]);
  const commits = [root, swc].map((dir) => {
    assert.equal(
      run("git", [
        "-C",
        dir,
        "status",
        "--porcelain",
        "--untracked-files=no",
        "--",
        "crates",
        "npm",
        "bindings",
        "packages",
        "Cargo.toml",
        "Cargo.lock",
      ]),
      "",
      `Tracked sources must be clean: ${dir}`,
    );
    return run("git", ["-C", dir, "rev-parse", "HEAD"]);
  });
  const require = createRequire(
    path.join(root, "tasks/benchmark/package.json"),
  );
  const coreVersion = require("@swc/core/package.json").version;
  const nextVersion = JSON.parse(
    readFileSync(path.join(root, "npm/core/package.json"), "utf8"),
  ).version;
  const coreCommit = run("git", [
    "-C", swc, "rev-parse", `v${coreVersion}^{commit}`,
  ]);
  const jsNames = [
    `@swc-next/core@${nextVersion} (${commits[0]})`,
    `@swc/core@${coreVersion} (${coreCommit})`,
  ];
  run("hyperfine", ["--version"]);
  build(swc);

  // Keep the benchmark commands on the same CPU.
  const expand = (list) =>
    list
      .trim()
      .split(",")
      .flatMap((range) => {
        const [first, last = first] = range.split("-").map(Number);
        return Array.from({ length: last - first + 1 }, (_, i) => first + i);
      });
  const allowed = expand(
    readFileSync("/proc/self/status", "utf8").match(
      /^Cpus_allowed_list:\s+(\S+)/m,
    )[1],
  );
  const coreCpus = "/sys/bus/event_source/devices/cpu_core/cpus";
  const preferred = existsSync(coreCpus)
    ? expand(readFileSync(coreCpus, "utf8"))
    : allowed;
  const cpu = Number(
    options["--cpu"] ??
      preferred.find((cpu) => allowed.includes(cpu)) ??
      allowed[0],
  );
  assert(allowed.includes(cpu), `CPU ${cpu} is not available`);
  console.error(`Benchmark CPU: ${cpu}`);

  // Hyperfine executes these arguments directly, without a shell.
  const quote = (value) => `'${value.replaceAll("'", "'\\''")}'`;
  const lines = ["# Benchmark", ""];
  for (const [title, feature] of features) {
    lines.push(`## ${title}`, "");
    for (const file of files) {
      if (file === "typescript.js" && feature !== "parse") continue;
      console.error(`${title}: ${file}`);
      const isJs = title === "JS parser";
      const input = path.join("tasks/benchmark/files", file);
      const commands = ["swc_next", "swc"].map((engine) =>
        isJs
          ? [
              process.execPath,
              `tasks/benchmark/bin/${engine}.js`,
              input,
              feature,
            ]
          : [
              `target/website-benchmark/release/${engine.replace("_", "-")}`,
              input,
              feature,
            ],
      );
      let names = [`swc-next@${commits[0]}`, `swc@${commits[1]}`];
      if (isJs) {
        const timings = commands.map((command) =>
          Number(
            run("taskset", [
              "--cpu-list",
              String(cpu),
              ...command,
              "--calibrate",
            ]),
          ),
        );
        assert(
          timings.every((ms) => Number.isFinite(ms) && ms > 0),
          "Invalid JS calibration",
        );
        const iterations = Math.max(
          5,
          Math.ceil(Number(options["--js-time-ms"]) / Math.min(...timings)),
        );
        for (const command of commands) command.push(String(iterations));
        names = jsNames.map((name) => `${name}; ${iterations} parses + 5 warmups`);
        console.error(
          `JS batch: ${iterations} parses per engine; calibration ${timings.map((ms) => ms.toFixed(3)).join(" / ")} ms/parse`,
        );
      }
      // Each JS process warms its own V8 state. Repeat in reverse order to
      // expose drift without relying on hyperfine warmups in other processes.
      const orders = isJs ? [[0, 1], [1, 0]] : [[0, 1]];
      const output = orders
        .map((order) =>
          run("taskset", [
            "--cpu-list",
            String(cpu),
            "hyperfine",
            "--style",
            "basic",
            "--shell",
            "none",
            "--warmup",
            isJs ? "0" : options["--warmup"],
            "--runs",
            isJs ? options["--js-runs"] : options["--runs"],
            "--metrics",
            "time_wall_clock:ms,memory_peak_resident:MiB",
            ...order.flatMap((index) => [
              "--command-name",
              names[index],
              commands[index].map(quote).join(" "),
            ]),
          ]),
        )
        .join("\n\n");
      lines.push(
        `### ${file}`,
        "",
        "```text",
        output.replace(/[ \t]+$/gm, ""),
        "```",
        "",
      );
    }
  }
  const output = path.resolve(options["--output"]);
  mkdirSync(path.dirname(output), { recursive: true });
  writeFileSync(output, lines.join("\n"));
  console.error(`Wrote ${output}`);
}

try {
  main();
} catch (error) {
  console.error(error.stack ?? error);
  process.exitCode = 1;
}
