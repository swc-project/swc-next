#!/usr/bin/env node

// Timing approach adapted from:
// https://github.com/overlookmotel/oxc-raw-str-bench/blob/main/bench.ts
// Warm up, target ~10 ms per round, and keep the fastest round to reduce GC and
// scheduler noise. Fresh processes and alternating order reduce JIT and drift noise.

import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

const SCRIPT = fileURLToPath(import.meta.url);
const DEFAULT_FIXTURES = path.resolve(
  path.dirname(SCRIPT),
  "../../../../tasks/benchmark/files",
);
const WARMUP_ROUNDS = 5;
const TARGET_ROUND_MS = 10;
const MIN_ROUNDS = 5;

function usage() {
  return `Usage: node ${path.relative(process.cwd(), SCRIPT)} \\
  --baseline /absolute/path/to/main/npm/core/src-js/index.js \\
  --candidate /absolute/path/to/branch/npm/core/src-js/index.js \\
  [--fixtures ${DEFAULT_FIXTURES}] [--file name.js ...] \\
  [--trials 3] [--time-ms 1000]`;
}

function parseArgs(args) {
  const options = { files: [], trials: 3, timeMs: 1000 };
  for (let i = 0; i < args.length; i++) {
    const key = args[i];
    if (key === "--help") {
      console.log(usage());
      process.exit(0);
    }
    if (!["--baseline", "--candidate", "--fixtures", "--file", "--trials", "--time-ms"].includes(key)) {
      throw new Error(`Unknown option: ${key}\n${usage()}`);
    }
    const value = args[++i];
    if (value === undefined) throw new Error(`Missing value for ${key}`);
    if (key === "--file") options.files.push(value);
    else if (key === "--time-ms") options.timeMs = Number(value);
    else if (key === "--trials") options.trials = Number(value);
    else options[key.slice(2)] = value;
  }
  if (!options.baseline || !options.candidate) throw new Error(usage());
  if (!Number.isInteger(options.trials) || options.trials < 1) {
    throw new Error("--trials must be a positive integer");
  }
  if (!Number.isFinite(options.timeMs) || options.timeMs <= 0) {
    throw new Error("--time-ms must be a positive number");
  }
  options.baseline = path.resolve(options.baseline);
  options.candidate = path.resolve(options.candidate);
  options.fixtures = path.resolve(options.fixtures ?? DEFAULT_FIXTURES);
  for (const entry of [options.baseline, options.candidate]) {
    if (!existsSync(entry)) throw new Error(`Parser entry does not exist: ${entry}`);
  }
  if (!existsSync(options.fixtures)) {
    throw new Error(`Fixture directory does not exist: ${options.fixtures}`);
  }
  if (options.files.length === 0) {
    options.files = readdirSync(options.fixtures)
      .filter((name) => /\.(?:js|jsx|ts|tsx)$/.test(name))
      .sort();
  }
  if (options.files.length === 0) throw new Error("No fixtures selected");
  for (const name of options.files) {
    if (path.basename(name) !== name || !existsSync(path.join(options.fixtures, name))) {
      throw new Error(`Invalid fixture name: ${name}`);
    }
  }
  return options;
}

function median(numbers) {
  const values = [...numbers].sort((a, b) => a - b);
  const middle = Math.floor(values.length / 2);
  return values.length % 2 ? values[middle] : (values[middle - 1] + values[middle]) / 2;
}

async function worker(config) {
  const { parseSync, CommentMode, langFromPath, sourceTypeFromPath } =
    await import(pathToFileURL(config.entry).href);
  if (typeof parseSync !== "function") {
    throw new Error(`parseSync is missing from ${config.entry}`);
  }
  const results = [];
  let checksum = 0;
  for (const name of config.files) {
    const file = path.join(config.fixtures, name);
    const source = readFileSync(file, "utf8");
    const options = {
      comments: CommentMode.None,
      lang: langFromPath(file),
      preserveParens: true,
      sourceType: sourceTypeFromPath(file),
    };
    const run = () => {
      checksum += parseSync(source, options).program.body.length;
    };

    let warmupMs = 0;
    for (let i = 0; i < WARMUP_ROUNDS; i++) {
      const start = performance.now();
      run();
      warmupMs += performance.now() - start;
    }
    const cyclesPerRound = Math.max(
      1,
      Math.round(TARGET_ROUND_MS / Math.max(warmupMs / WARMUP_ROUNDS, 0.001)),
    );
    const samples = [];
    const deadline = performance.now() + config.timeMs;
    while (samples.length < MIN_ROUNDS || performance.now() < deadline) {
      const start = performance.now();
      for (let i = 0; i < cyclesPerRound; i++) run();
      samples.push((performance.now() - start) / cyclesPerRound);
    }
    results.push({
      name,
      bytes: Buffer.byteLength(source),
      bestMs: Math.min(...samples),
      rounds: samples.length,
      cyclesPerRound,
    });
  }
  process.stdout.write(JSON.stringify({ results, checksum }));
}

function runTrial(options, entry, label, trial) {
  process.stderr.write(`${label} trial ${trial}/${options.trials}\n`);
  const config = {
    entry,
    fixtures: options.fixtures,
    files: options.files,
    timeMs: options.timeMs,
  };
  const child = spawnSync(process.execPath, [SCRIPT, "--worker", JSON.stringify(config)], {
    encoding: "utf8",
    maxBuffer: 1024 * 1024,
  });
  if (child.error) throw child.error;
  if (child.status !== 0) {
    throw new Error(`${label} trial ${trial} failed:\n${child.stderr || child.stdout}`);
  }
  const output = JSON.parse(child.stdout);
  if (!Number.isFinite(output.checksum)) throw new Error("Invalid worker checksum");
  return output.results;
}

function printResults(options, trials) {
  console.log(`Node ${process.version}; ${options.trials} trials; ${options.timeMs} ms per fixture and trial; lower is better`);
  console.log(`Baseline:  ${options.baseline}`);
  console.log(`Candidate: ${options.candidate}`);
  console.log("| Fixture | Bytes | Baseline ms | Candidate ms | Change | Baseline range ms | Candidate range ms |");
  console.log("| --- | ---: | ---: | ---: | ---: | ---: | ---: |");
  for (let i = 0; i < options.files.length; i++) {
    const baseline = trials.baseline.map((trial) => trial[i].bestMs);
    const candidate = trials.candidate.map((trial) => trial[i].bestMs);
    const baseMedian = median(baseline);
    const candidateMedian = median(candidate);
    const percent = (candidateMedian / baseMedian - 1) * 100;
    const range = (values) => `${Math.min(...values).toFixed(3)}–${Math.max(...values).toFixed(3)}`;
    console.log(`| ${options.files[i]} | ${trials.baseline[0][i].bytes} | ${baseMedian.toFixed(3)} | ${candidateMedian.toFixed(3)} | ${percent >= 0 ? "+" : ""}${percent.toFixed(1)}% | ${range(baseline)} | ${range(candidate)} |`);
  }
}

try {
  if (process.argv[2] === "--worker") {
    await worker(JSON.parse(process.argv[3]));
  } else {
    const options = parseArgs(process.argv.slice(2));
    const trials = { baseline: [], candidate: [] };
    for (let trial = 1; trial <= options.trials; trial++) {
      const order = trial % 2 ? ["baseline", "candidate"] : ["candidate", "baseline"];
      for (const label of order) {
        trials[label].push(runTrial(options, options[label], label, trial));
      }
    }
    printResults(options, trials);
  }
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exitCode = 1;
}
