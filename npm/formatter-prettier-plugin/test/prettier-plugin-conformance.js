import { readFile, readdir, writeFile } from "node:fs/promises";
import { dirname, join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";
import * as prettier from "prettier";
import { formatDocBufferSync } from "@swc-next/formatter-js";
import { printDocBufferSync } from "@swc-next/formatter";
import { encodeOptions } from "../src-js/options.js";
import * as plugin from "../dist/index.js";

const workspace = fileURLToPath(new URL("../../../", import.meta.url));
const fixtureRoot = join(workspace, "crates/swc_formatter_js/tests/fixtures");

const { options: declarations } = await prettier.getSupportInfo();
const defaults = Object.fromEntries(
  declarations.map(({ name, default: value }) => [name, value]),
);
const reportFile = new URL(
  "./prettier-plugin-conformance.json",
  import.meta.url,
);

// Mirror the pinned Prettier host's input boundary, before the plugin parser.
// Resolve auto from the original text: normalization would discard that choice.
function normalizeInput(source, options = {}) {
  let endOfLine = options.endOfLine ?? "lf";
  if (endOfLine === "auto") {
    const cr = source.indexOf("\r");
    endOfLine = cr < 0 ? "lf" : source[cr + 1] === "\n" ? "crlf" : "cr";
  }
  return {
    source: source.replace(/^\uFEFF/u, "").replace(/\r\n?/gu, "\n"),
    options: { ...options, endOfLine },
  };
}

async function fixtureCases() {
  const entries = await readdir(fixtureRoot, {
    recursive: true,
    withFileTypes: true,
  });
  const paths = entries
    .filter(
      (entry) =>
        entry.isFile() &&
        /\.(?:js|jsx|ts|tsx|mjs|cjs|mts|cts)$/u.test(entry.name),
    )
    .map((entry) => join(entry.parentPath, entry.name))
    .sort();
  const cache = new Map();
  async function optionSets(directory) {
    if (!cache.has(directory)) {
      let sets;
      try {
        sets = JSON.parse(
          await readFile(join(directory, "options.json"), "utf8"),
        );
      } catch (error) {
        if (error.code !== "ENOENT") throw error;
        sets =
          directory === fixtureRoot
            ? [{}]
            : await optionSets(dirname(directory));
      }
      if (!Array.isArray(sets) || sets.length === 0) {
        throw new Error(`Expected formatter option sets in ${directory}`);
      }
      cache.set(directory, sets);
    }
    return cache.get(directory);
  }
  const cases = [];
  for (const filepath of paths) {
    const source = await readFile(filepath, "utf8");
    for (const options of await optionSets(dirname(filepath))) {
      cases.push({
        relative: relative(fixtureRoot, filepath).split(sep).join("/"),
        filepath,
        source,
        options,
      });
    }
  }
  if (cases.length === 0) {
    throw new Error(`No formatter fixtures found in ${fixtureRoot}`);
  }
  return cases;
}

function parserOptions(filepath) {
  const language = /\.d\.(?:ts|mts|cts)$/iu.test(filepath)
    ? 5
    : /\.tsx$/iu.test(filepath)
      ? 4
      : /\.(?:ts|mts|cts)$/iu.test(filepath)
        ? 2
        : 3;
  const sourceType = /\.(?:mjs|mts)$/iu.test(filepath)
    ? 1
    : /\.(?:cjs|cts)$/iu.test(filepath)
      ? 3
      : 4;
  return 1 | (sourceType << 1) | (language << 4);
}

function formatSwcWithSwcFormatter(source, filepath, options) {
  const normalized = normalizeInput(source, { ...defaults, ...options });
  const buffer = formatDocBufferSync(
    normalized.source,
    parserOptions(filepath),
    encodeOptions(normalized.options),
  );
  const bom = source.startsWith("\uFEFF") ? "\uFEFF" : "";
  return bom + printDocBufferSync(buffer, normalized.options);
}

async function format(formatter) {
  try {
    return { output: await formatter() };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { error: message.split("\n")[0] };
  }
}

function recordDifference(comparison, fixture, options, outputs) {
  const [left, right] = Object.values(outputs);
  if (
    left.error === undefined &&
    right.error === undefined &&
    left.output === right.output
  )
    return;

  const difference = Object.keys(options).length > 0 ? { options } : {};
  for (const [name, { output, error }] of Object.entries(outputs)) {
    if (error !== undefined) difference[`${name}Error`] = error;
    else difference[name] = output;
  }
  (comparison[fixture] ??= []).push(difference);
}

const printerComparison = {};
const pluginComparison = {};
for (const { relative, filepath, source, options } of await fixtureCases()) {
  const swcWithSwcFormatter = await format(() =>
    formatSwcWithSwcFormatter(source, filepath, options),
  );
  const swcWithPrettierFormatter = await format(() =>
    prettier.format(source, {
      ...options,
      filepath,
      plugins: [plugin],
    }),
  );
  const parser = /\.(?:ts|tsx|mts|cts)$/iu.test(filepath)
    ? "typescript"
    : "babel";
  const prettierWithPrettierFormatter = await format(() =>
    prettier.format(source, { ...options, filepath, parser }),
  );

  // Keep SWC document construction fixed and compare the two printers.
  recordDifference(printerComparison, relative, options, {
    swcFormatter: swcWithSwcFormatter,
    prettierFormatter: swcWithPrettierFormatter,
  });
  // Keep the Prettier printer fixed and compare the two language formatters.
  recordDifference(pluginComparison, relative, options, {
    swcPlugin: swcWithPrettierFormatter,
    prettierNative: prettierWithPrettierFormatter,
  });
}
const report = {
  "SWC plugin + SWC formatter vs SWC plugin + Prettier formatter":
    printerComparison,
  "SWC plugin + Prettier formatter vs native Prettier + Prettier formatter":
    pluginComparison,
};
await writeFile(reportFile, JSON.stringify(report, null, 2) + "\n");
console.log("Updated test/prettier-plugin-conformance.json");
