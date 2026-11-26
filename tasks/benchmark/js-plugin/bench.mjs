import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { performance } from "node:perf_hooks";
import { setImmediate } from "node:timers/promises";

const corePackages = { ffi1: "@swc-next/core", ffi2: "@swc-next/core-ffi2" };

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
  return async suite => (await import(`./${suite}/${path}`))[factory](...args);
}

function loadFixtures(api) {
  const selected = process.env.JS_PLUGIN_FILES?.split(",").map(name => name.trim()).filter(Boolean);
  const paths = new Map();
  for (const directory of ["../files", "fixtures"]) {
    const root = join(import.meta.dirname, directory);
    for (const entry of readdirSync(root, { withFileTypes: true })) {
      if (entry.isFile() && /\.(?:js|jsx|ts|tsx)$/.test(entry.name)) {
        paths.set(entry.name, join(root, entry.name));
      }
    }
  }
  if (selected !== undefined) {
    for (const name of selected) {
      assert(paths.has(name), `Unknown benchmark fixture: ${name}`);
    }
  }
  const names = (selected === undefined ? [...paths.keys()] : [...new Set(selected)]).sort();
  return names.map(name => ({
    name,
    source: readFileSync(paths.get(name), "utf8"),
    parser: { lang: api.langFromPath(name) },
  }));
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
// separate from the warmed samples and from parse/print/startup time.
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

function createReport({ suite, core, pluginName, files, bytes, hashes, analysis, normalizeOutput, spec, coldPluginNs, warmupIterations }) {
  return {
    plugin: pluginName,
    mode: suite,
    node: process.version,
    nodeArgs: process.execArgv,
    v8: process.versions.v8,
    worker: import.meta.filename,
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
    printInterface: analysis ? "none (parse and analysis only)" : "api.generateSync",
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

function runPass(files, measureFile, state) {
  const phases = { parse: 0, plugin: 0, print: 0 };
  const start = process.hrtime.bigint();
  for (const file of files) {
    state.sink += measureFile(file, phases);
  }
  return {
    elapsedNs: (process.hrtime.bigint() - start).toString(),
    parseNs: String(Math.round(phases.parse * 1e6)),
    pluginNs: String(Math.round(phases.plugin * 1e6)),
    printNs: String(Math.round(phases.print * 1e6)),
  };
}

function medianMs(samples, key) {
  const sorted = samples.map(sample => Number(sample[key])).sort((left, right) => left - right);
  const lower = sorted[Math.floor((sorted.length - 1) / 2)];
  const upper = sorted[Math.floor(sorted.length / 2)];
  return ((lower + upper) / 2e6).toFixed(3);
}

function positiveInteger(name, fallback) {
  const value = Number(process.env[name] ?? fallback);
  assert(Number.isSafeInteger(value) && value > 0, `${name} must be a positive integer`);
  return value;
}

async function runBenchmark(pluginName, suite) {
  const core = import.meta.resolve(corePackages[suite]);
  const api = await import(core);
  const spec = pluginSpecs[pluginName];
  const analysis = spec.analysis === true;
  const normalizeOutput = !analysis && spec.normalize !== false;
  const print = ast => api.generateSync(ast.program).code;

  const files = loadFixtures(api);
  assert(files.length > 0, "Select at least one benchmark fixture");
  const plugin = await spec.load(suite);
  const warmupIterations = positiveInteger("JS_PLUGIN_WARMUP", 3);
  const sampleCount = positiveInteger("JS_PLUGIN_SAMPLES", 20);

  const measureFile = createMeasureFile({ api, plugin, analysis, print, spec });
  const cold = runColdPass(files, measureFile);
  const transform = createTransform({ api, pluginName, plugin, analysis, print });
  const hashes = validateOutputs({ api, isFfi2: suite === "ffi2", files, transform, analysis,
    normalizeOutput, coldPluginByFile: cold.coldPluginByFile });
  const bytes = files.reduce((sum, file) => sum + Buffer.byteLength(file.source), 0);
  const report = createReport({
    suite, core, pluginName, files, bytes, hashes, analysis, normalizeOutput, spec,
    coldPluginNs: cold.coldPluginNs, warmupIterations,
  });

  const state = { sink: cold.sink };
  // Give Node a turn to finalize native AST buffers between corpus passes.
  for (let iteration = 0; iteration < warmupIterations; iteration++) {
    await setImmediate();
    runPass(files, measureFile, state);
  }

  const trace = process.env.JS_PLUGIN_TRACE === "1";
  for (let sample = 0; sample < sampleCount; sample++) {
    await setImmediate();
    if (trace) process.stderr.write(`JS_PLUGIN_MEASURE_BEGIN ${sample}\n`);
    report.samples.push(runPass(files, measureFile, state));
    if (trace) process.stderr.write(`JS_PLUGIN_MEASURE_END ${sample}\n`);
  }
  assert(state.sink >= 0);
  writeReport(report, process.env.JS_PLUGIN_REPORT_DIR);

  const { samples } = report;
  const timings = [
    `parse ${medianMs(samples, "parseNs")} ms`,
    `plugin ${medianMs(samples, "pluginNs")} ms`,
    ...(analysis ? [] : [`print ${medianMs(samples, "printNs")} ms`]),
    `total ${medianMs(samples, "elapsedNs")} ms`,
  ];
  const coldMs = (cold.coldPluginNs / 1e6).toFixed(3);
  console.log(
    `${suite}/${pluginName} (${files.length} files, median of ${sampleCount}): ` +
    `${timings.join(", ")} [cold plugin ${coldMs} ms]`,
  );
}

async function main() {
  const [plugin = "all", suite = "all"] = process.argv.slice(2);
  assert(plugin === "all" || Object.hasOwn(pluginSpecs, plugin), `Unknown plugin: ${plugin}`);
  assert(suite === "all" || Object.hasOwn(corePackages, suite), `Unknown suite: ${suite}`);
  if (plugin !== "all" && suite !== "all") {
    await runBenchmark(plugin, suite);
    return;
  }

  const plugins = plugin === "all" ? Object.keys(pluginSpecs) : [plugin];
  const suites = suite === "all" ? Object.keys(corePackages) : [suite];
  // Each case gets a fresh process for its cold measurement and V8 warmup.
  for (const mode of suites) {
    for (const name of plugins) {
      const result = spawnSync(process.execPath, [...process.execArgv, import.meta.filename, name, mode], {
        stdio: "inherit",
      });
      if (result.error) throw result.error;
      if (result.status !== 0) {
        process.exitCode = result.status ?? 1;
        return;
      }
    }
  }
}

await main();
