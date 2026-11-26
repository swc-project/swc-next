import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { performance } from "node:perf_hooks";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";

const pluginSpecs = {
  formatjs: {
    load: loadPlugin("formatjs/index.js", "formatjs", { ast: true }),
  },
  "import-meta-env": {
    load: loadPlugin("import-meta-env.js", "importMetaEnv"),
  },
  "react-remove-properties": {
    load: loadPlugin("react-remove-properties.js", "reactRemoveProperties"),
  },
  "ident-length": {
    analysis: true,
    load: loadPlugin("ident-length.js", "identLength"),
    result: context => context.metadata.identifierLength,
  },
  "ident-rename": {
    // Full renames can make parameter properties unparseable, so the raw output
    // cannot be re-parsed for RegExp flag normalization.
    normalize: false,
    normalization: "raw (destructive rename may not re-parse)",
    load: loadPlugin("ident-rename.js", "identRename"),
  },
};

function loadPlugin(path, factory, ...args) {
  return async suite => (await import(`../files/${suite}/${path}`))[factory](...args);
}

function loadFixtures(directory, api) {
  const selected = process.env.JS_PLUGIN_FILES?.split(",").map(name => name.trim()).filter(Boolean);
  const names = readdirSync(directory, { withFileTypes: true })
    .filter(entry => entry.isFile() && /\.(?:js|jsx|ts|tsx)$/.test(entry.name))
    .map(entry => entry.name)
    .filter(name => selected === undefined || selected.includes(name))
    .sort();
  if (selected !== undefined) {
    for (const name of selected) {
      assert(names.includes(name), `Unknown benchmark fixture: ${name}`);
    }
  }
  return names.map(name => ({
    name,
    source: readFileSync(join(directory, name), "utf8"),
    parser: { lang: api.langFromPath(name) },
  }));
}

function createPrint(api, uncheckedPrint) {
  return uncheckedPrint === undefined
    ? ast => api.generateSync(ast.program).code
    : ast => uncheckedPrint(...ast.encode(), 0);
}

// Resolve the root and comments during parse, including lazy JS decoding.
// Context construction and result consumption are outside the phase clocks.
// Analysis returns before encode or native printing; each route is monomorphic.
function createMeasureFile({ api, plugin, analysis, print, spec }) {
  return function measureFile(file, phases) {
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
    if (analysis) {
      return spec.result(context);
    }
    const code = print(ast);
    const printEnd = performance.now();
    phases.print += printEnd - pluginEnd;
    return code.length;
  };
}

// First execution before preflight or warmup. Keep this auxiliary measurement
// separate from the warmed Criterion samples and from parse/print/startup time.
function runColdPass(files, measureFile) {
  const phases = { parse: 0, plugin: 0, print: 0 };
  const coldPluginByFile = [];
  let sink = 0;
  for (const file of files) {
    const before = phases.plugin;
    sink += measureFile(file, phases);
    coldPluginByFile.push(Math.round((phases.plugin - before) * 1e6));
  }
  return { sink, coldPluginNs: Math.round(phases.plugin * 1e6), coldPluginByFile };
}

function createTransform({ api, pluginName, plugin, analysis, print }) {
  return function transform(file) {
    const ast = api.parseSync(file.source, file.parser);
    assert.equal(ast.diagnostics.filter(diagnostic => diagnostic.severity === "error").length, 0,
      `${pluginName}/${file.name}: parser diagnostics`);
    const originalCode = analysis ? undefined : print(ast);
    if (analysis && ast.encode !== undefined) {
      ast.encode = () => { throw new Error("Analysis must not encode AST storage"); };
    }
    const context = { source: file.source, filename: file.name, comments: ast.comments, metadata: {} };
    plugin(ast.program, context);
    if (analysis) {
      return { code: "", metadata: context.metadata, changed: false };
    }
    const code = print(ast);
    return { code, metadata: context.metadata, changed: originalCode !== code };
  };
}

// Native printing can reorder RegExp flags. Normalize only those lexical ranges
// for validation; the measured pipelines still print their original output.
function canonicalRegExpFlags(api, isFfi2, code, parser) {
  const parsed = api.parseSync(code, parser);
  const edits = [];
  function collect(node, flagCount) {
    const start = node.end - flagCount;
    const flags = code.slice(start, node.end);
    const canonical = [...flags].sort().join("");
    if (flags !== canonical) { edits.push({ start, end: node.end, flags: canonical }); }
  }
  if (isFfi2) {
    api.walk(parsed.program, { RegExpLiteral(ctx) { const node = ctx.node; collect(node, node.flags.length); } });
  } else {
    api.walk(parsed.program, { Literal(node) { if (node.regex) { collect(node, node.regex.flags.length); } } });
  }
  let normalized = code;
  for (const edit of edits.sort((left, right) => right.start - left.start)) {
    normalized = normalized.slice(0, edit.start) + edit.flags + normalized.slice(edit.end);
  }
  return normalized;
}

function validateOutputs({ api, isFfi2, files, transform, analysis, normalizeOutput, coldPluginByFile }) {
  const corpusHash = createHash("sha256");
  const outputHash = createHash("sha256");
  const rawOutputHash = createHash("sha256");
  const metadataHash = createHash("sha256");
  const outcomes = [];
  for (const file of files) {
    corpusHash.update(file.name).update("\0").update(file.source).update("\0");
    const actual = transform(file);
    const normalizedActual = analysis ? "" : normalizeOutput ? canonicalRegExpFlags(api, isFfi2, actual.code, file.parser) : actual.code;
    outputHash.update(file.name).update("\0").update(normalizedActual).update("\0");
    rawOutputHash.update(file.name).update("\0").update(actual.code).update("\0");
    metadataHash.update(file.name).update("\0").update(JSON.stringify(actual.metadata)).update("\0");
    outcomes.push({ name: file.name, changed: actual.changed, messages: actual.metadata.formatjs?.messages.length ?? 0,
      identifierLength: actual.metadata.identifierLength,
      coldPluginNs: String(coldPluginByFile[outcomes.length]) });
  }
  return { corpusHash, outputHash, rawOutputHash, metadataHash, outcomes };
}

function createReport({ suite, core, pluginName, files, bytes, hashes, analysis, normalizeOutput, spec, coldPluginNs, warmupIterations, uncheckedPrint }) {
  return {
    plugin: pluginName,
    mode: suite,
    node: process.version,
    nodeArgs: process.execArgv,
    v8: process.versions.v8,
    worker: fileURLToPath(import.meta.url),
    core,
    files: files.map(file => file.name),
    bytes,
    corpusHash: hashes.corpusHash.digest("hex"),
    outputHash: hashes.outputHash.digest("hex"),
    rawOutputHash: hashes.rawOutputHash.digest("hex"),
    outputNormalization: analysis ? "omitted (analysis only)" : normalizeOutput
      ? "RegExp flag order only, outside timing" : spec.normalization,
    metadataHash: hashes.metadataHash.digest("hex"),
    outcomes: hashes.outcomes,
    printInterface: analysis ? "none (parse and analysis only)"
      : uncheckedPrint === undefined ? "api.generateSync" : "native.generateSync2Unchecked (unchecked)",
    coldPluginNs: String(coldPluginNs),
    coldPluginDefinition: "First full dataset execution before preflight/warmup; JS plugin only, excluding parse, print, module loading and process startup",
    warmupIterations,
    phaseDefinitions: {
      parse: "Rust parse + Rust AST encode + JS decode (root and comments)",
      plugin: "JS walker/transform after warmup; lazy field reads during the plugin are included",
      print: analysis ? "omitted" : "JS AST encode + Rust decode + Rust print",
      total: "Complete loop including context construction, clock reads and result consumption",
    },
    samples: [],
  };
}

function writeReport(report, directory) {
  if (directory === undefined) return;
  mkdirSync(directory, { recursive: true });
  writeFileSync(join(directory, `${report.mode}-${report.plugin}.json`), `${JSON.stringify(report, null, 2)}\n`);
}

function runWarmup(files, measureFile, iterations, state) {
  const phases = { parse: 0, plugin: 0, print: 0 };
  for (let iteration = 0; iteration < iterations; iteration++) {
    for (const file of files) {
      state.sink += measureFile(file, phases);
    }
  }
}

async function serveMeasurements({ files, measureFile, report, analysis, state, write }) {
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
        state.sink += measureFile(file, phases);
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
    write();
    const selected = phase === "parse" ? parseNs : phase === "plugin" ? pluginNs
      : phase === "print" ? printNs : elapsed;
    process.stdout.write(`${selected}\n`);
  }
}

async function main() {
  const filesDirectory = join(dirname(fileURLToPath(import.meta.url)), "../files");
  const pluginName = process.argv[2];
  const suite = process.argv[3] ?? "js-plugin";
  assert(suite === "js-plugin" || suite === "js-plugin2", `Unknown plugin suite: ${suite}`);
  const isFfi2 = suite === "js-plugin2";
  const core = import.meta.resolve(isFfi2 ? "@swc-next/core/ffi2" : "@swc-next/core");
  const api = await import(core);
  const spec = pluginSpecs[pluginName];
  assert(spec !== undefined, `Unknown plugin: ${pluginName}`);
  const analysis = spec.analysis === true;
  const normalizeOutput = !analysis && spec.normalize !== false;
  // The trusted printer is an internal native binding, not a public core export.
  const binding = analysis || !isFfi2 ? undefined : await import(new URL("./binding.js", core));
  const uncheckedPrint = binding?.generateSync2Unchecked;
  const print = analysis ? undefined : createPrint(api, uncheckedPrint);

  const files = loadFixtures(filesDirectory, api);
  assert(files.length > 0);
  const plugin = await spec.load(suite);
  // Three full dataset passes reach the hot path with default V8 tier-up thresholds.
  const warmupIterations = Number(process.env.JS_PLUGIN_WARMUP ?? 3);
  assert(Number.isSafeInteger(warmupIterations) && warmupIterations > 0,
    "JS_PLUGIN_WARMUP must be a positive integer");

  const measureFile = createMeasureFile({ api, plugin, analysis, print, spec });
  const cold = runColdPass(files, measureFile);
  const transform = createTransform({ api, pluginName, plugin, analysis, print });
  const hashes = validateOutputs({ api, isFfi2, files, transform, analysis, normalizeOutput, coldPluginByFile: cold.coldPluginByFile });
  const bytes = files.reduce((sum, file) => sum + Buffer.byteLength(file.source), 0);
  const report = createReport({
    suite, core, pluginName, files, bytes, hashes, analysis, normalizeOutput, spec,
    coldPluginNs: cold.coldPluginNs, warmupIterations, uncheckedPrint,
  });
  const reportDirectory = process.env.JS_PLUGIN_REPORT_DIR;
  const write = () => writeReport(report, reportDirectory);

  const state = { sink: cold.sink };
  runWarmup(files, measureFile, warmupIterations, state);
  write();
  process.stdout.write(`ready ${suite} ${files.length} ${bytes} ${cold.coldPluginNs}\n`);

  await serveMeasurements({ files, measureFile, report, analysis, state, write });
  assert(state.sink >= 0);
}

await main();
