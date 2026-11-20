import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { performance } from "node:perf_hooks";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";
import * as api from "@swc-next/core";

const filesDirectory = join(dirname(fileURLToPath(import.meta.url)), "../files");
const pluginName = process.argv[2];
assert(["formatjs", "import-meta-env", "react-remove-properties", "count-functions"].includes(pluginName));
const analysis = pluginName === "count-functions";
// The trusted printer is an internal native binding, not a public core export.
const uncheckedPrint = analysis ? undefined
  : (await import(new URL("./binding.js", import.meta.resolve("@swc-next/core")))).generateSyncUnchecked;
if (!analysis) {
  assert.equal(typeof uncheckedPrint, "function");
}

const files = readdirSync(filesDirectory, { withFileTypes: true })
  .filter(entry => entry.isFile() && /\.(?:js|jsx|ts|tsx)$/.test(entry.name))
  .map(entry => entry.name)
  .sort()
  .map(name => ({
    name,
    source: readFileSync(join(filesDirectory, name), "utf8"),
    parser: { lang: api.langFromPath(name) },
  }));
assert(files.length > 0);

async function createPlugin() {
  if (analysis) {
    const module = await import("../files/js-plugin/count-functions.js");
    return module.countFunctions();
  }
  const module = await import(`../files/js-plugin/${pluginName === "formatjs" ? "formatjs/index" : pluginName}.js`);
  if (pluginName === "formatjs") {
    return module.formatjs({ ast: true });
  }
  return pluginName === "import-meta-env" ? module.importMetaEnv() : module.reactRemoveProperties();
}

const plugin = await createPlugin();
const warmupIterations = Number(process.env.JS_PLUGIN_WARMUP ?? 1);
assert(Number.isSafeInteger(warmupIterations) && warmupIterations > 0,
  "JS_PLUGIN_WARMUP must be a positive integer");

// Resolve the root and comments during parse, including lazy JS decoding.
// Context construction and result consumption are outside the phase clocks.
// Analysis returns before encode or native printing; each route is monomorphic.
const measureFile = analysis ? (file, phases) => {
  const parseStart = performance.now();
  const ast = api.parseSync(file.source, file.parser);
  const program = ast.program, comments = ast.comments;
  const parseEnd = performance.now();
  const context = { source: file.source, filename: file.name, comments, metadata: {} };
  const pluginStart = performance.now();
  plugin(program, context);
  const pluginEnd = performance.now();
  phases.parse += parseEnd - parseStart;
  phases.plugin += pluginEnd - pluginStart;
  return context.metadata.functionCount;
} : (file, phases) => {
  const parseStart = performance.now();
  const ast = api.parseSync(file.source, file.parser);
  const program = ast.program, comments = ast.comments;
  const parseEnd = performance.now();
  const context = { source: file.source, filename: file.name, comments, metadata: {} };
  const pluginStart = performance.now();
  plugin(program, context);
  const pluginEnd = performance.now();
  const code = uncheckedPrint(...ast.encode(), 0);
  const printEnd = performance.now();
  phases.parse += parseEnd - parseStart;
  phases.plugin += pluginEnd - pluginStart;
  phases.print += printEnd - pluginEnd;
  return code.length;
};

// First execution before preflight or warmup. Keep this auxiliary measurement
// separate from the warmed Criterion samples and from parse/print/startup time.
let sink = 0;
const coldPhases = { parse: 0, plugin: 0, print: 0 };
const coldPluginByFile = [];
for (const file of files) {
  const before = coldPhases.plugin;
  sink += measureFile(file, coldPhases);
  coldPluginByFile.push(Math.round((coldPhases.plugin - before) * 1e6));
}
const coldPluginNs = Math.round(coldPhases.plugin * 1e6);

function transform(file) {
  const ast = api.parseSync(file.source, file.parser);
  assert.equal(ast.diagnostics.filter(diagnostic => diagnostic.severity === "error").length, 0,
    `${pluginName}/${file.name}: parser diagnostics`);
  const originalCode = analysis ? undefined : uncheckedPrint(...ast.encode(), 0);
  if (analysis) {
    ast.encode = () => { throw new Error("Analysis must not encode AST storage"); };
  }
  const context = { source: file.source, filename: file.name, comments: ast.comments, metadata: {} };
  plugin(ast.program, context);
  if (analysis) {
    assert.equal(ast.materializedNodeCount, 1, "Numeric analysis should only create the root view");
    let expected = 0;
    api.walk(api.parseSync(file.source, file.parser).program, {
      FunctionDeclaration() { expected++; },
      FunctionExpression() { expected++; },
      ArrowFunctionExpression() { expected++; },
    });
    assert.equal(context.metadata.functionCount, expected, `${file.name}: function count`);
    return { code: "", metadata: context.metadata, changed: false };
  }
  const buffers = ast.encode();
  const code = uncheckedPrint(...buffers, 0);
  return { code, metadata: context.metadata, changed: originalCode !== code };
}

const corpusHash = createHash("sha256");
const outputHash = createHash("sha256");
const rawOutputHash = createHash("sha256");
const metadataHash = createHash("sha256");
const outcomes = [];

// Native printing can reorder RegExp flags. Normalize only those lexical ranges
// for validation; the measured pipelines still print their original output.
function canonicalRegExpFlags(code, parser) {
  const parsed = api.parseSync(code, parser);
  const edits = [];
  api.walk(parsed.program, {
    Literal(node) {
      if (node.regex !== undefined && node.regex !== null) {
        const start = node.end - node.regex.flags.length;
        const flags = code.slice(start, node.end);
        const canonical = [...flags].sort().join("");
        if (flags !== canonical) {
          edits.push({ start, end: node.end, flags: canonical });
        }
      }
    },
  });
  let normalized = code;
  for (const edit of edits.sort((left, right) => right.start - left.start)) {
    normalized = normalized.slice(0, edit.start) + edit.flags + normalized.slice(edit.end);
  }
  return normalized;
}

for (const file of files) {
  corpusHash.update(file.name).update("\0").update(file.source).update("\0");
  const actual = transform(file);
  const normalizedActual = analysis ? "" : canonicalRegExpFlags(actual.code, file.parser);
  outputHash.update(file.name).update("\0").update(normalizedActual).update("\0");
  rawOutputHash.update(file.name).update("\0").update(actual.code).update("\0");
  metadataHash.update(file.name).update("\0").update(JSON.stringify(actual.metadata)).update("\0");
  outcomes.push({ name: file.name, changed: actual.changed, messages: actual.metadata.formatjs?.messages.length ?? 0,
    functionCount: actual.metadata.functionCount, coldPluginNs: String(coldPluginByFile[outcomes.length]) });
}

const bytes = files.reduce((sum, file) => sum + Buffer.byteLength(file.source), 0);
const report = {
  plugin: pluginName,
  mode: "ast",
  node: process.version,
  nodeArgs: process.execArgv,
  v8: process.versions.v8,
  worker: fileURLToPath(import.meta.url),
  core: import.meta.resolve("@swc-next/core"),
  files: files.map(file => file.name),
  bytes,
  corpusHash: corpusHash.digest("hex"),
  outputHash: outputHash.digest("hex"),
  rawOutputHash: rawOutputHash.digest("hex"),
  outputNormalization: "RegExp flag order only, outside timing",
  metadataHash: metadataHash.digest("hex"),
  outcomes,
  printInterface: analysis ? "none (parse and analysis only)" : "native.generateSyncUnchecked (unchecked)",
  coldPluginNs: String(coldPluginNs),
  coldPluginDefinition: "First full dataset execution before preflight/warmup; JS plugin only, excluding parse, print, module loading and process startup",
  warmupIterations,
  phaseDefinitions: {
    parse: "Rust parse + Rust AST encode + JS decode (root and comments)",
    plugin: "JS walker/transform after warmup; lazy field reads during the plugin are included",
    print: analysis ? "omitted" : "JS AST encode + unchecked Rust decode + Rust print",
    total: "Complete loop including context construction, clock reads and result consumption",
  },
  samples: [],
};
const reportDirectory = process.env.JS_PLUGIN_REPORT_DIR;
function writeReport() {
  if (reportDirectory !== undefined) {
    mkdirSync(reportDirectory, { recursive: true });
    writeFileSync(join(reportDirectory, `${pluginName}.json`), `${JSON.stringify(report, null, 2)}\n`);
  }
}
const warmupPhases = { parse: 0, plugin: 0, print: 0 };
for (let iteration = 0; iteration < warmupIterations; iteration++) {
  for (const file of files) {
    sink += measureFile(file, warmupPhases);
  }
}
writeReport();
process.stdout.write(`ready ast ${files.length} ${bytes} ${coldPluginNs}\n`);

const input = createInterface({ input: process.stdin, crlfDelay: Infinity });
for await (const line of input) {
  const [count, phase = "total"] = line.trim().split(/\s+/);
  const iterations = Number(count);
  assert(Number.isSafeInteger(iterations) && iterations > 0);
  assert(["parse", "plugin", "print", "total"].includes(phase));
  assert(!analysis || phase !== "print", "Analysis has no print phase");
  if (process.env.JS_PLUGIN_TRACE === "1") {
    process.stderr.write(`JS_PLUGIN_MEASURE_BEGIN ${report.samples.length}\n`);
  }
  const phases = { parse: 0, plugin: 0, print: 0 };
  const start = process.hrtime.bigint();
  for (let iteration = 0; iteration < iterations; iteration++) {
    for (const file of files) {
      sink += measureFile(file, phases);
    }
  }
  const elapsed = process.hrtime.bigint() - start;
  if (process.env.JS_PLUGIN_TRACE === "1") {
    process.stderr.write(`JS_PLUGIN_MEASURE_END ${report.samples.length}\n`);
  }
  const parseNs = Math.round(phases.parse * 1e6);
  const pluginNs = Math.round(phases.plugin * 1e6);
  const printNs = Math.round(phases.print * 1e6);
  report.samples.push({ iterations, phase, elapsedNs: elapsed.toString(),
    parseNs: String(parseNs), pluginNs: String(pluginNs), printNs: String(printNs) });
  writeReport();
  const selected = phase === "parse" ? parseNs : phase === "plugin" ? pluginNs
    : phase === "print" ? printNs : elapsed;
  process.stdout.write(`${selected}\n`);
}
assert(sink >= 0);
