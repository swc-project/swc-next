// Minifier output size comparison.
//
//   node minifier-size.mjs build <minify.rs> <bin>
//   node minifier-size.mjs measure <bin> <input-dir> <results.json> [<output-dir>]
//   node minifier-size.mjs compare <base.json> <head.json> <summary.md>
//
// `build` compiles the minify example of the current checkout and copies the
// executable to <bin>. It first puts the given example source in place, so
// that the base and head builds use the same example, and restores the
// checkout afterwards. `measure` minifies every `.js` file in <input-dir> and
// records the raw, gzip, and Brotli sizes of each output. `compare` writes
// Markdown and prints `changed=true` or `changed=false`. The output size is
// deterministic, so any difference is a change.
//
// This file must not import repository files: the workflow copies it out of the
// checkout before it checks out the base revision.

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, unlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { brotliCompressSync, constants, gzipSync } from "node:zlib";

const PACKAGE = "swc_next_ecma_minifier";
const EXAMPLE = "minify";
const EXAMPLE_PATH = "crates/swc_ecma_minifier/examples/minify.rs";

function build(source, bin) {
  const original = existsSync(EXAMPLE_PATH) ? readFileSync(EXAMPLE_PATH) : null;
  mkdirSync(dirname(EXAMPLE_PATH), { recursive: true });
  copyFileSync(source, EXAMPLE_PATH);
  try {
    const messages = execFileSync(
      "cargo",
      ["build", "--release", "--locked", "-p", PACKAGE, "--example", EXAMPLE, "--message-format=json"],
      { encoding: "utf8", stdio: ["ignore", "pipe", "inherit"], maxBuffer: 1 << 28 },
    )
      .split("\n")
      .filter(Boolean)
      .map(line => JSON.parse(line));
    const artifact = messages.find(m => m.executable && m.target?.name === EXAMPLE && m.target.kind.includes("example"));
    assert(artifact, `executable for ${EXAMPLE} was not produced`);
    mkdirSync(dirname(resolve(bin)), { recursive: true });
    copyFileSync(artifact.executable, bin);
  } finally {
    if (original === null) unlinkSync(EXAMPLE_PATH);
    else writeFileSync(EXAMPLE_PATH, original);
  }
  console.log(`Built ${EXAMPLE} into ${bin}`);
}

function measure(bin, inputDir, results, outputDir) {
  const names = readdirSync(inputDir)
    .filter(name => name.endsWith(".js"))
    .sort();
  assert(names.length > 0, `no .js files in ${inputDir}`);
  const out = outputDir ?? mkdtempSync(join(tmpdir(), "minifier-size-"));
  execFileSync(resolve(bin), [out, ...names.map(name => join(inputDir, name))], { stdio: "inherit" });

  const files = names.map(name => {
    const code = readFileSync(join(out, name));
    return {
      name,
      input: readFileSync(join(inputDir, name)).length,
      minified: code.length,
      gzip: gzipSync(code, { level: 9 }).length,
      brotli: brotliCompressSync(code, {
        params: {
          [constants.BROTLI_PARAM_QUALITY]: constants.BROTLI_MAX_QUALITY,
          [constants.BROTLI_PARAM_SIZE_HINT]: code.length,
        },
      }).length,
    };
  });
  if (outputDir === undefined) rmSync(out, { recursive: true });
  writeFileSync(results, `${JSON.stringify({ files }, null, 2)}\n`);
}

const METRICS = [
  ["minified", "Minified"],
  ["gzip", "Gzip"],
  ["brotli", "Brotli"],
];

const bytes = value => value.toLocaleString("en-US");

function cell(base, head) {
  if (base === undefined) return `${bytes(head)} (new)`;
  if (head === undefined) return `${bytes(base)} (removed)`;
  if (base === head) return bytes(head);
  const delta = head - base;
  const sign = delta > 0 ? "+" : "";
  const ratio = (delta / base) * 100;
  const percent = Math.abs(ratio) < 0.005 ? "<0.01%" : `${sign}${ratio.toFixed(2)}%`;
  const icon = delta > 0 ? "🔺" : "🔻";
  return `${bytes(base)} → **${bytes(head)}** (${icon} ${sign}${bytes(delta)}, ${percent})`;
}

function compare(basePath, headPath, summary) {
  const base = new Map(JSON.parse(readFileSync(basePath, "utf8")).files.map(file => [file.name, file]));
  const head = new Map(JSON.parse(readFileSync(headPath, "utf8")).files.map(file => [file.name, file]));
  const names = [...new Set([...base.keys(), ...head.keys()])].sort();

  let changed = false;
  const rows = names.map(name => {
    const b = base.get(name);
    const h = head.get(name);
    if (!b || !h || METRICS.some(([key]) => b[key] !== h[key])) changed = true;
    const input = (h ?? b).input;
    return `| ${name} | ${bytes(input)} | ${METRICS.map(([key]) => cell(b?.[key], h?.[key])).join(" | ")} |`;
  });
  // Totals cover the files that both revisions measured.
  const common = names.filter(name => base.has(name) && head.has(name));
  const total = (map, key) => common.reduce((sum, name) => sum + map.get(name)[key], 0);
  rows.push(
    `| **Total** | ${bytes(total(head, "input"))} | ${METRICS.map(([key]) => cell(total(base, key), total(head, key))).join(" | ")} |`,
  );

  const lines = [
    "## Minifier output size",
    "",
    `Base: \`${process.env.BASE_SHA ?? basename(basePath)}\`, head: \`${process.env.HEAD_SHA ?? basename(headPath)}\``,
    "",
    changed ? "The minified output size changed." : "The minified output size did not change.",
    "",
    "Sizes are in bytes, shown as `base → head` when they differ. Gzip uses level 9 and Brotli uses quality 11.",
    "",
    `| File | Input | ${METRICS.map(([, label]) => label).join(" | ")} |`,
    `| --- | ---: | ${METRICS.map(() => "---:").join(" | ")} |`,
    ...rows,
  ];
  writeFileSync(summary, `${lines.join("\n")}\n`);
  console.log(`changed=${changed}`);
}

const [command, ...args] = process.argv.slice(2);
switch (command) {
  case "build":
    assert.equal(args.length, 2, "usage: build <minify.rs> <bin>");
    build(...args);
    break;
  case "measure":
    assert(args.length === 3 || args.length === 4, "usage: measure <bin> <input-dir> <results.json> [<output-dir>]");
    measure(...args);
    break;
  case "compare":
    assert.equal(args.length, 3, "usage: compare <base.json> <head.json> <summary.md>");
    compare(...args);
    break;
  default:
    console.error("usage: node minifier-size.mjs build|measure|compare ...");
    process.exit(1);
}
