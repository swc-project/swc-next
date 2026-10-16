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
const files = ["astro-config.ts", "effect-schema-ast.ts", "hono-types.ts"];
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
  const coreJs = path.join(work, "core-js");
  mkdirSync(collector, { recursive: true });
  mkdirSync(coreJs, { recursive: true });
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
  run(
    "cargo",
    [
      "build",
      "--locked",
      "--release",
      "--manifest-path",
      path.join(swc, "Cargo.toml"),
      "-p",
      "binding_core_node",
      "--no-default-features",
      "--features",
      "swc_v1",
      "-j",
      "6",
    ],
    { env, stdio: "inherit" },
  );
  run("pnpm", ["--filter", "@swc-next/core", "build", "--release"], {
    stdio: "inherit",
  });

  // Build the public JS wrapper from the same checkout as the native binding.
  const require = createRequire(path.join(root, "npm/core/package.json"));
  const transpiler = require("@swc/core");
  for (const name of ["index", "spack"]) {
    const source = readFileSync(
      path.join(swc, `packages/core/src/${name}.ts`),
      "utf8",
    );
    const { code } = transpiler.transformSync(source, {
      filename: `${name}.ts`,
      jsc: { parser: { syntax: "typescript" }, target: "es2022" },
      module: { type: "commonjs" },
    });
    writeFileSync(path.join(coreJs, `${name}.js`), code);
  }
  copyFileSync(
    path.join(work, "release/libbinding_core_node.so"),
    path.join(coreJs, "swc.node"),
  );
  writeFileSync(
    path.join(coreJs, "binding.js"),
    'module.exports = require("./swc.node");\n',
  );
  const { version } = JSON.parse(
    readFileSync(path.join(swc, "packages/core/package.json"), "utf8"),
  );
  writeFileSync(
    path.join(coreJs, "package.json"),
    JSON.stringify({ type: "commonjs", version }),
  );
}

function main() {
  const options = {
    "--swc-dir": path.resolve(root, "../swc"),
    "--runs": "10",
    "--warmup": "3",
    "--cpu": undefined,
    "--output": path.join(root, "website/docs/Benchmark.md"),
  };
  const args = process.argv.slice(2);
  if (args.includes("--help")) {
    console.log(
      "Usage: node website/docs/scripts/benchmark.js [--swc-dir ../swc] [--runs 10] [--warmup 3] [--cpu N] [--output website/docs/Benchmark.md]",
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
  run("hyperfine", ["--version"]);
  build(swc);

  // Keep counters on one CPU; generic perf events can return zero after
  // migration between core and atom PMUs on Intel hybrid processors.
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
      console.error(`${title}: ${file}`);
      const input = path.join("tasks/benchmark/files", file);
      const commands = ["swc_next", "swc"].map((engine) =>
        title === "JS parser"
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
      const output = run(
        "taskset",
        [
          "--cpu-list",
          String(cpu),
          "hyperfine",
          "--style",
          "basic",
          "--shell",
          "none",
          "--warmup",
          options["--warmup"],
          "--runs",
          options["--runs"],
          "--metrics",
          "time_wall_clock:ms,instructions,memory_peak_resident:MiB",
          "--command-name",
          `swc-next@${commits[0]}`,
          commands[0].map(quote).join(" "),
          "--command-name",
          `swc@${commits[1]}`,
          commands[1].map(quote).join(" "),
        ],
        {
          env: {
            ...process.env,
            SWC_CORE_ENTRY: path.join(work, "core-js/index.js"),
            SWC_BINARY_PATH: path.join(work, "core-js/swc.node"),
          },
        },
      );
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
