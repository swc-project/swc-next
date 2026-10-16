import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { appendFileSync, closeSync, copyFileSync, mkdirSync, openSync, readFileSync, writeFileSync } from "node:fs";
import { cpus } from "node:os";
import { join, resolve, dirname } from "node:path";
import { selectCases, fixtures } from "../../tasks/benchmark/js/runner.mjs";
import { loadSummaries, renderFailureSummary } from "./render-gungraun-summary.mjs";
import { compare, render, validateRepeatability } from "./render-gungraun-js-summary.mjs";

const root = process.cwd();
const prepared = resolve(process.env.JS_BENCH_PREPARED ?? "target/gungraun-js-prepared");
const output = resolve(process.env.JS_BENCH_OUTPUT ?? "target/gungraun-js-ci");
const harness = join(prepared, "harness");
const data = join(output, "gungraun");
const filter = process.env.JS_BENCH_FILTER ?? "";
const selected = selectCases(process.env.JS_BENCH_SUITE ?? "full", filter);
const run = (exe, args, options = {}) => execFileSync(exe, args, { cwd: root, stdio: "inherit", ...options });
const read = (exe, args) => run(exe, args, { encoding: "utf8", stdio: ["ignore", "pipe", "inherit"] }).trim();
const sha = ref => read("git", ["rev-parse", "--verify", `${ref}^{commit}`]);
const hash = path => createHash("sha256").update(readFileSync(path)).digest("hex");

function prepare() {
  mkdirSync(harness, { recursive: true });
  const base = sha(process.env.BASE_REF);
  const head = sha(process.env.HEAD_REF);
  const metadata = {
    schema_version: 1, base_sha: base, head_sha: head, harness_sha: head,
    node: process.version, node_flags: ["--jitless", "--predictable", "--expose-gc", "--random-seed=42"], jit: false,
    rustc: read("rustc", ["--version"]), gungraun: read("gungraun-runner", ["--version"]),
    pnpm: read("pnpm", ["--version"]), platform: process.platform, arch: process.arch,
    warmup: 100, iterations: { ast: 10, path: 10000 }, trials: 3,
    profile: { release: true, debug: 2, strip: "none" }, hashes: {},
  };
  const copyRevisionFile = (source, destination, name) => {
    const fd = openSync(destination, "wx");
    try {
      run("git", ["show", `${head}:${source}`], { stdio: ["ignore", fd, "inherit"] });
    } finally {
      closeSync(fd);
    }
    metadata.hashes[name] = hash(destination);
  };
  for (const file of ["Cargo.toml", "Cargo.lock", "gungraun_js.rs", "runner.mjs", "measure.c"]) {
    copyRevisionFile(`tasks/benchmark/js/${file}`, join(harness, file), file);
  }
  mkdirSync(join(harness, "files"));
  for (const file of fixtures) {
    copyRevisionFile(`tasks/benchmark/files/${file}`, join(harness, "files", file), `files/${file}`);
  }
  const include = resolve(dirname(process.execPath), "../include/node");
  run("cc", ["-shared", "-fPIC", "-O2", "-g", `-I${include}`, "-o", join(harness, "measure.node"), join(harness, "measure.c")]);
  // Compile the driver and runner once, then reuse exactly these binaries in every shard.
  const artifacts = read("cargo", ["bench", "--locked", "--manifest-path", join(harness, "Cargo.toml"),
    "--bench", "gungraun_js", "--no-run", "--message-format=json"]);
  const binary = artifacts.split("\n").map(line => JSON.parse(line)).find(item => item.executable)?.executable;
  assert(binary, "benchmark executable was not produced");
  copyFileSync(binary, join(harness, "gungraun-js-bench"));
  copyFileSync(read("which", ["gungraun-runner"]), join(harness, "gungraun-runner"));
  for (const file of ["gungraun-js-bench", "gungraun-runner", "measure.node"]) metadata.hashes[file] = hash(join(harness, file));
  for (const [side, revision] of [["base", base], ["head", head]]) {
    const checkout = join(prepared, side);
    run("git", ["worktree", "add", "--detach", checkout, revision]);
    const env = { ...process.env, CARGO_PROFILE_RELEASE_DEBUG: "2", CARGO_PROFILE_RELEASE_STRIP: "none" };
    run("pnpm", ["install", "--frozen-lockfile"], { cwd: checkout, env });
    run("pnpm", ["--filter", "@swc-next/core", "build", "--release"], { cwd: checkout, env });
  }
  writeFileSync(join(prepared, "build-metadata.json"), JSON.stringify(metadata, null, 2));
  if (process.env.GITHUB_OUTPUT) {
    const matrix = { include: selected.map((name, index) => ({ id: index, case: name })) };
    appendFileSync(process.env.GITHUB_OUTPUT, `matrix=${JSON.stringify(matrix)}\n`);
  }
}

async function measure() {
  mkdirSync(output, { recursive: true });
  const metadata = JSON.parse(readFileSync(join(prepared, "build-metadata.json"), "utf8"));
  assert.equal(process.version, metadata.node);
  assert.equal(process.arch, metadata.arch);
  for (const file of ["gungraun-js-bench", "gungraun-runner", "measure.node"]) assert.equal(hash(join(harness, file)), metadata.hashes[file]);
  const sameSha = process.argv.includes("--same-sha");
  if (sameSha) metadata.base_sha = metadata.head_sha;
  Object.assign(metadata, { filter, case: process.env.JS_BENCH_CASE ?? null, status: "running",
    valgrind: read("valgrind", ["--version"]), cpu: cpus()[0]?.model,
    run_url: `${process.env.GITHUB_SERVER_URL}/${process.env.GITHUB_REPOSITORY}/actions/runs/${process.env.GITHUB_RUN_ID}` });
  const saveMetadata = () => writeFileSync(join(output, "metadata.json"), JSON.stringify(metadata, null, 2));
  saveMetadata();
  try {
    for (const side of ["base", "head"]) {
      const entry = join(prepared, sameSha ? "head" : side, "npm/core/src-js/index.js");
      const log = join(output, `${side}.log`);
      writeFileSync(log, "");
      // Stream progress to Actions while preserving the exact output in artifacts.
      await new Promise((accept, reject) => {
        const child = spawn(join(harness, "gungraun-js-bench"), ["--bench", `--workspace-root=${harness}`, `--home=${data}`, "--save-summary=json",
          side === "base" ? "--save-baseline=base" : "--baseline=base"], {
          cwd: root, env: { ...process.env, PATH: `${harness}:${process.env.PATH}`, GUNGRAUN_COLOR: "never",
            JS_BENCH_HARNESS: harness, JS_BENCH_ENTRY: entry, JS_BENCH_NODE: process.execPath },
        });
        for (const stream of [child.stdout, child.stderr]) stream.on("data", bytes => {
          appendFileSync(log, bytes);
          process.stdout.write(bytes);
        });
        child.on("error", reject);
        child.on("close", code => code === 0 ? accept() : reject(new Error(`${side} benchmark failed: ${code}`)));
      });
      if (side === "base") writeFileSync(join(output, "base-summaries.json"), JSON.stringify(await loadSummaries(data), null, 2));
    }
    const rows = compare(await loadSummaries(data));
    writeFileSync(join(output, "comparison.json"), JSON.stringify(rows, null, 2));
    if (sameSha) validateRepeatability(rows);
    writeFileSync(join(output, "summary.md"), render(rows));
    metadata.status = "passed";
  } catch (error) {
    metadata.status = "failed";
    metadata.error = error.message;
    writeFileSync(join(output, "summary.md"), renderFailureSummary(error.message));
    throw error;
  } finally {
    saveMetadata();
    if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, readFileSync(join(output, "summary.md")));
  }
}

if (!process.argv.includes("--measure")) prepare();
if (!process.argv.includes("--prepare")) await measure();
