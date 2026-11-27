import { readFile, readdir, writeFile } from "node:fs/promises";
import { dirname, join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";
import * as prettier from "prettier";
import { printDocBufferSync } from "@swc-next/formatter";
import * as plugin from "@swc-next/formatter-prettier-plugin/javascript";
import { formatJsDocBufferSync } from "../dist/binding.js";
import { formatJsDoc } from "../src-js/javascript/format.js";
import { toFormatterConfig } from "../src-js/javascript/options.js";

const workspace = fileURLToPath(new URL("../../../", import.meta.url));
const fixtureRoot = join(workspace, "crates/swc_formatter/tests/fixtures");

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
    ? "dts"
    : /\.tsx$/iu.test(filepath)
      ? "tsx"
      : /\.(?:ts|mts|cts)$/iu.test(filepath)
        ? "ts"
        : "jsx";
  const sourceType = /\.(?:mjs|mts)$/iu.test(filepath)
    ? "module"
    : /\.(?:cjs|cts)$/iu.test(filepath)
      ? "commonjs"
      : "unambiguous";
  return { lang: language, sourceType };
}

// Convert SWC fixture configuration only at the Prettier conformance boundary.
function prettierOptions(options) {
  const config = {
    lineWidth: 120,
    indentStyle: "tab",
    indentWidth: 2,
    lineEnding: "lf",
    semicolons: "always",
    quoteStyle: "double",
    jsxQuoteStyle: "double",
    quoteProperties: "as-needed",
    trailingCommas: "all",
    arrowParentheses: "always",
    bracketSpacing: true,
    bracketSameLine: false,
    singleAttributePerLine: false,
    ...options,
  };
  return {
    printWidth: config.lineWidth,
    useTabs: config.indentStyle === "tab",
    tabWidth: config.indentWidth,
    endOfLine: config.lineEnding,
    semi: config.semicolons === "always",
    singleQuote: config.quoteStyle === "single",
    jsxSingleQuote: config.jsxQuoteStyle === "single",
    quoteProps: config.quoteProperties,
    trailingComma: config.trailingCommas,
    arrowParens: config.arrowParentheses === "always" ? "always" : "avoid",
    bracketSpacing: config.bracketSpacing,
    bracketSameLine: config.bracketSameLine,
    singleAttributePerLine: config.singleAttributePerLine,
  };
}

function formatSwcWithSwcFormatter(source, filepath, options) {
  const normalized = normalizeInput(source, options);
  const config = toFormatterConfig(normalized.options);
  const buffer = formatJsDoc(
    formatJsDocBufferSync,
    normalized.source,
    parserOptions(filepath),
    config,
  );
  const bom = source.startsWith("\uFEFF") ? "\uFEFF" : "";
  return bom + printDocBufferSync(buffer, config);
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
  const prettierConfig = prettierOptions(options);
  const swcWithSwcFormatter = await format(() =>
    formatSwcWithSwcFormatter(source, filepath, prettierConfig),
  );
  const swcWithPrettierFormatter = await format(() =>
    prettier.format(source, {
      ...prettierConfig,
      filepath,
      plugins: [plugin],
    }),
  );
  const parser = /\.(?:ts|tsx|mts|cts)$/iu.test(filepath)
    ? "typescript"
    : "babel";
  const prettierWithPrettierFormatter = await format(() =>
    prettier.format(source, { ...prettierConfig, filepath, parser }),
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
