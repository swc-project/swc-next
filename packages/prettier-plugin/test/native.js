import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import { createRequire } from "node:module";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const workspace = fileURLToPath(new URL("../../../", import.meta.url));
const fixtureRoot = join(workspace, "crates/swc_ecma_formatter/tests/fixtures");

// Load the native addon directly, including when the plugin is forced to Wasm.
// It returns strings from format(), independently of the option and Doc bridge.
export const { formatNativeSync } = createRequire(import.meta.url)(
  join(workspace, "npm/formatter/test/binding/swc-next-formatter.node"),
);

// Mirror the pinned Prettier host's input boundary, before the plugin parser.
// Resolve auto from the original text: normalization would discard that choice.
export function normalizeInput(source, options = {}) {
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

export async function fixtureCases() {
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
      assert.ok(Array.isArray(sets) && sets.length > 0);
      cache.set(directory, sets);
    }
    return cache.get(directory);
  }
  const cases = [];
  for (const filepath of paths) {
    const source = await readFile(filepath, "utf8");
    for (const options of await optionSets(dirname(filepath))) {
      cases.push({
        relative: relative(fixtureRoot, filepath),
        filepath,
        source,
        options,
      });
    }
  }
  assert.ok(cases.length > 0);
  return cases;
}
