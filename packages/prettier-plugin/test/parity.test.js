import assert from "node:assert/strict";
import test from "node:test";
import * as prettier from "prettier";
import * as plugin from "../index.js";
import { fixtureCases, formatNativeSync, normalizeInput } from "./native.js";

test("SWCFMT matches every native fixture option on the same normalized input", async (t) => {
  const cases = await fixtureCases();
  const differences = [];
  for (const { relative, filepath, source, options } of cases) {
    const normalized = normalizeInput(source, options);
    const expected = formatNativeSync(
      normalized.source,
      filepath,
      normalized.options,
    );
    const actual = await prettier.format(source, {
      plugins: [plugin],
      filepath,
      ...options,
    });
    // The host restores a leading BOM outside the plugin's parser/printer pair.
    const output = (source.startsWith("\uFEFF") ? "\uFEFF" : "") + expected;
    if (actual !== output) {
      differences.push({ relative, options, expected: output, actual });
    }
  }
  const fixtures = new Set(cases.map(({ filepath }) => filepath)).size;
  const binding =
    process.env.NAPI_RS_FORCE_WASI === "error" ? "Wasm" : "Native";
  t.diagnostic(
    `${binding}: ${fixtures} fixtures, ${cases.length} option sets, ${differences.length} byte differences`,
  );
  assert.deepEqual(differences, []);
});
