// Paired wall-time comparison of the Criterion benchmarks.
//
//   node paired-walltime.mjs build <bin-dir>
//   node paired-walltime.mjs run <base-bin-dir> <head-bin-dir> <results.json>
//   node paired-walltime.mjs render <results.json>
//
// `build` compiles the benchmark targets of the current checkout and copies the
// executables to <bin-dir>. `run` measures every benchmark that exists in both
// builds in rounds; each round runs the base binary and the head binary on the
// same machine, so both see the same runner noise. The order alternates between
// rounds (base first, then head first), so an even number of rounds cancels the
// effect of the run order. `render` prints Markdown.
//
// This file must not import repository files: the workflow copies it out of the
// checkout before it checks out the base revision.

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { cpus, release } from "node:os";
import { join, resolve } from "node:path";

const PACKAGE = "benchmark";
const TARGETS = ["parser", "visit", "semantic"];
// A change is reported only when every round agrees on its direction and the
// median paired change is at least this large.
const THRESHOLD = 0.02;

const env = (name, fallback) => process.env[name] || fallback;
const read = (exe, args, options = {}) =>
  execFileSync(exe, args, { encoding: "utf8", stdio: ["ignore", "pipe", "inherit"], maxBuffer: 1 << 28, ...options });

function build(binDir) {
  const metadata = JSON.parse(read("cargo", ["metadata", "--locked", "--no-deps", "--format-version=1"]));
  const pkg = metadata.packages.find(p => p.name === PACKAGE);
  assert(pkg, `package ${PACKAGE} was not found`);
  const available = new Set(pkg.targets.filter(t => t.kind.includes("bench")).map(t => t.name));
  const targets = TARGETS.filter(t => available.has(t));
  assert(targets.length > 0, "no benchmark targets to build");

  const args = ["bench", "--locked", "-p", PACKAGE, "--no-run", "--message-format=json"];
  for (const target of targets) args.push("--bench", target);
  const messages = read("cargo", args, { stdio: ["ignore", "pipe", "inherit"] })
    .split("\n")
    .filter(Boolean)
    .map(line => JSON.parse(line));

  mkdirSync(binDir, { recursive: true });
  for (const target of targets) {
    const artifact = messages.find(m => m.executable && m.target?.name === target && m.target.kind.includes("bench"));
    assert(artifact, `executable for ${target} was not produced`);
    copyFileSync(artifact.executable, join(binDir, target));
  }
  console.log(`Built ${targets.join(", ")} into ${binDir}`);
}

function listBenchmarks(binary) {
  return read(binary, ["--bench", "--list"])
    .split("\n")
    .map(line => line.match(/^(.*): benchmark$/)?.[1])
    .filter(Boolean);
}

// Criterion stores each benchmark in a directory derived from its id; map the
// ids back to their estimates through `benchmark.json`.
function collectEstimates(home) {
  const found = new Map();
  const walk = dir => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      if (!entry.isDirectory()) continue;
      const path = join(dir, entry.name);
      if (entry.name === "new" && existsSync(join(path, "benchmark.json"))) {
        const { full_id } = JSON.parse(readFileSync(join(path, "benchmark.json"), "utf8"));
        const estimates = JSON.parse(readFileSync(join(path, "estimates.json"), "utf8"));
        found.set(full_id, estimates.median.point_estimate);
      } else {
        walk(path);
      }
    }
  };
  if (existsSync(home)) walk(home);
  return found;
}

function measure(binary, id, home, timing) {
  // Criterion writes progress to stderr; show it only when the run fails.
  try {
    execFileSync(
      binary,
      ["--bench", "--exact", id, "--noplot", "--warm-up-time", timing.warmUp, "--measurement-time", timing.measurement],
      { stdio: ["ignore", "ignore", "pipe"], env: { ...process.env, CRITERION_HOME: home }, maxBuffer: 1 << 24 },
    );
  } catch (error) {
    process.stderr.write(error.stderr ?? "");
    throw error;
  }
  const value = collectEstimates(home).get(id);
  assert(Number.isFinite(value), `no estimate was recorded for ${id}`);
  return value;
}

// Records the machine that ran both builds. GitHub-hosted runners do not all
// use the same CPU, so compare results from different runs with care.
function describeEnvironment() {
  const lscpu = new Map();
  try {
    const walk = items => {
      for (const item of items ?? []) {
        lscpu.set(item.field.replace(/:$/, ""), item.data);
        walk(item.children);
      }
    };
    walk(JSON.parse(read("lscpu", ["-J"], { env: { ...process.env, LC_ALL: "C" } })).lscpu);
  } catch {
    // lscpu is optional; fall back to Node.js data below.
  }
  const flags = new Set((lscpu.get("Flags") ?? "").split(/\s+/));
  const simd = ["sse4_2", "avx", "avx2", "bmi2", "avx512f", "avx512bw", "avx512vl"].filter(flag => flags.has(flag));
  const osRelease = existsSync("/etc/os-release") ? readFileSync("/etc/os-release", "utf8") : "";
  const image = [process.env.ImageOS, process.env.ImageVersion].filter(Boolean).join(" ");
  return {
    cpu_model: lscpu.get("Model name") ?? cpus()[0]?.model ?? "unknown",
    vendor: lscpu.get("Vendor ID") ?? null,
    family_model_stepping: ["CPU family", "Model", "Stepping"].map(key => lscpu.get(key) ?? "?").join("/"),
    logical_cpus: cpus().length,
    sockets: lscpu.get("Socket(s)") ?? null,
    cores_per_socket: lscpu.get("Core(s) per socket") ?? null,
    threads_per_core: lscpu.get("Thread(s) per core") ?? null,
    max_mhz: lscpu.get("CPU max MHz") ?? null,
    caches: ["L1d", "L1i", "L2", "L3"]
      .filter(level => lscpu.has(`${level} cache`))
      .map(level => `${level} ${lscpu.get(`${level} cache`)}`),
    simd: lscpu.has("Flags") ? simd : null,
    hypervisor: lscpu.get("Hypervisor vendor") ?? null,
    os: osRelease.match(/^PRETTY_NAME="?([^"\n]*)"?$/m)?.[1] ?? process.platform,
    kernel: release(),
    runner_image: image || null,
    runner_name: process.env.RUNNER_NAME ?? null,
    rustc: read("rustc", ["--version"]).trim(),
  };
}

function runComparison(baseDir, headDir, output) {
  const rounds = Number(env("PAIRED_WALLTIME_ROUNDS", "6"));
  const timing = {
    warmUp: env("PAIRED_WALLTIME_WARM_UP", "1"),
    measurement: env("PAIRED_WALLTIME_MEASUREMENT", "2"),
  };
  // Each binary must run first in the same number of rounds.
  assert(Number.isInteger(rounds) && rounds > 0 && rounds % 2 === 0, "PAIRED_WALLTIME_ROUNDS must be a positive even integer");

  const work = resolve(output, "..", "criterion");
  const benchmarks = [];
  const headOnly = [];
  const baseOnly = [];
  for (const target of TARGETS) {
    const base = join(baseDir, target);
    const head = join(headDir, target);
    const baseIds = existsSync(base) ? listBenchmarks(base) : [];
    const headIds = existsSync(head) ? listBenchmarks(head) : [];
    for (const id of headIds) {
      if (baseIds.includes(id)) benchmarks.push({ target, id, base: [], head: [] });
      else headOnly.push(id);
    }
    baseOnly.push(...baseIds.filter(id => !headIds.includes(id)));
  }
  assert(benchmarks.length > 0, "no benchmark exists in both builds");

  for (let round = 1; round <= rounds; round++) {
    const order = round % 2 ? ["base", "head"] : ["head", "base"];
    console.log(`Round ${round}/${rounds} (${order.join(" first, then ")})`);
    for (const bench of benchmarks) {
      const dirs = { base: baseDir, head: headDir };
      for (const side of order) {
        const home = join(work, `round-${round}`, side, bench.target);
        bench[side].push(measure(join(dirs[side], bench.target), bench.id, home, timing));
      }
      const change = (bench.head.at(-1) / bench.base.at(-1) - 1) * 100;
      console.log(`  ${bench.id}: ${change >= 0 ? "+" : ""}${change.toFixed(2)}%`);
    }
  }

  const results = {
    schema_version: 1,
    base_sha: env("BASE_SHA", ""),
    head_sha: env("HEAD_SHA", ""),
    environment: describeEnvironment(),
    rounds,
    warm_up_seconds: Number(timing.warmUp),
    measurement_seconds: Number(timing.measurement),
    threshold: THRESHOLD,
    benchmarks,
    head_only: headOnly,
    base_only: baseOnly,
  };
  writeFileSync(output, `${JSON.stringify(results, null, 2)}\n`);
}

const median = values => {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = sorted.length >> 1;
  return sorted.length % 2 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
};

function formatTime(ns) {
  if (ns >= 1e9) return `${(ns / 1e9).toFixed(3)} s`;
  if (ns >= 1e6) return `${(ns / 1e6).toFixed(3)} ms`;
  if (ns >= 1e3) return `${(ns / 1e3).toFixed(2)} µs`;
  return `${ns.toFixed(1)} ns`;
}

const formatChange = ratio => {
  const percent = (ratio - 1) * 100;
  return `${percent >= 0 ? "+" : ""}${percent.toFixed(2)}%`;
};

function classify(ratios, threshold) {
  const change = median(ratios) - 1;
  if (ratios.every(r => r > 1) && change >= threshold) return "slower";
  if (ratios.every(r => r < 1) && change <= -threshold) return "faster";
  return "unchanged";
}

function renderEnvironment(env) {
  const layout = env.sockets && env.cores_per_socket && env.threads_per_core
    ? ` (${env.sockets} socket × ${env.cores_per_socket} cores × ${env.threads_per_core} threads)`
    : "";
  const rows = [
    ["CPU", `${env.cpu_model}${env.vendor ? ` (${env.vendor}, family/model/stepping ${env.family_model_stepping})` : ""}`],
    ["Topology", `${env.logical_cpus} logical CPUs${layout}`],
    ["Max frequency", env.max_mhz ? `${Number(env.max_mhz).toFixed(0)} MHz` : null],
    ["Caches", env.caches.length ? env.caches.join(", ") : null],
    ["SIMD", env.simd ? env.simd.join(", ") || "none detected" : null],
    ["Hypervisor", env.hypervisor],
    ["OS", `${env.os}, kernel ${env.kernel}`],
    ["Runner image", env.runner_image],
    ["Runner", env.runner_name],
    ["Toolchain", env.rustc],
  ].filter(([, value]) => value);
  return [
    "### Environment",
    "",
    "Base and head ran on this machine in the same job.",
    "",
    "| | |",
    "| --- | --- |",
    ...rows.map(([key, value]) => `| ${key} | ${String(value).replaceAll("|", "\\|")} |`),
  ];
}

function render(path) {
  const results = JSON.parse(readFileSync(path, "utf8"));
  const icons = { slower: "🔴 Slower", faster: "🟢 Faster" };
  const counts = { slower: 0, faster: 0, unchanged: 0 };
  // Show only changed benchmarks to keep comments short. The artifact's
  // results.json keeps every measurement.
  const rows = results.benchmarks.flatMap(bench => {
    const ratios = bench.head.map((head, i) => head / bench.base[i]);
    const verdict = classify(ratios, results.threshold);
    counts[verdict]++;
    if (verdict === "unchanged") return [];
    return [[
      `\`${bench.id}\``,
      formatTime(median(bench.base)),
      formatTime(median(bench.head)),
      `**${formatChange(median(ratios))}**`,
      `${formatChange(Math.min(...ratios))} … ${formatChange(Math.max(...ratios))}`,
      icons[verdict],
    ]];
  });

  const short = sha => (sha ? `\`${sha.slice(0, 12)}\`` : "unknown");
  const lines = [
    "## Paired wall-time benchmark (x86_64)",
    "",
    `Base ${short(results.base_sha)} → head ${short(results.head_sha)}: ` +
      `${counts.slower} slower, ${counts.faster} faster, ${counts.unchanged} unchanged.`,
    "",
    ...(rows.length
      ? [
          "| Benchmark | Base median | Head median | Change | Range of rounds | Result |",
          "| --- | ---: | ---: | ---: | --- | --- |",
          ...rows.map(row => `| ${row.join(" | ")} |`),
        ]
      : ["No benchmark changed."]),
    "",
  ];
  lines.push(...renderEnvironment(results.environment), "");
  if (results.head_only.length) lines.push(`New benchmarks (not compared): ${results.head_only.map(id => `\`${id}\``).join(", ")}`, "");
  if (results.base_only.length) lines.push(`Removed benchmarks: ${results.base_only.map(id => `\`${id}\``).join(", ")}`, "");
  lines.push(
    "<details>",
    "<summary>Method</summary>",
    "",
    `Each of ${results.rounds} rounds runs every benchmark with the base binary and with the head binary ` +
      `in the same job (Criterion, ${results.warm_up_seconds} s warm-up, ${results.measurement_seconds} s measurement). ` +
      "Odd rounds run base first and even rounds run head first, so the run order does not bias the change. " +
      "Change is the median of the per-round head/base ratios of the Criterion median times. " +
      `A benchmark is marked slower or faster only when all rounds agree on the direction and the change is at least ${(results.threshold * 100).toFixed(0)}%. ` +
      "Unchanged benchmarks are not listed; the `paired-walltime` artifact has all results.",
    "",
    "</details>",
  );
  process.stdout.write(`${lines.join("\n")}\n`);
}

const [command, ...args] = process.argv.slice(2);
if (command === "build" && args.length === 1) build(resolve(args[0]));
else if (command === "run" && args.length === 3) runComparison(resolve(args[0]), resolve(args[1]), resolve(args[2]));
else if (command === "render" && args.length === 1) render(resolve(args[0]));
else {
  console.error("usage: paired-walltime.mjs build <bin-dir> | run <base-bin-dir> <head-bin-dir> <results.json> | render <results.json>");
  process.exit(2);
}
