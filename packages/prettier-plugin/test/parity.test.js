import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import test from "node:test";
import * as prettier from "prettier";
import * as core from "../../../npm/core/src-js/index.js";
import * as plugin from "../index.js";
import { fixtureCases, formatNativeSync, normalizeInput } from "./native.js";

const sha256 = (value) => createHash("sha256").update(value).digest("hex");
const caseKey = (relative, options) => {
  const stableOptions = Object.fromEntries(
    Object.entries(options).sort(([left], [right]) => left.localeCompare(right)),
  );
  return `${relative}\u0000${JSON.stringify(stableOptions)}`;
};
const manifest = JSON.parse(
  await readFile(new URL("./width-only-parity.json", import.meta.url), "utf8"),
);
const widthOnlyCases = new Map();
for (const entry of manifest.cases) {
  const key = caseKey(entry.relative, entry.options);
  assert.equal(sha256(key), entry.caseKeySha256);
  assert.equal(entry.classification, "width-only");
  assert.ok(!widthOnlyCases.has(key), `duplicate width-only case: ${entry.relative}`);
  widthOnlyCases.set(key, entry);
}

test("SWCFMT matches native fixtures except exact width-only layout cases", async (t) => {
  const cases = await fixtureCases();
  const differences = [];
  const staleExceptions = [];
  const matchedExceptions = new Set();
  let verifiedWidthOnly = 0;
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
    const key = caseKey(relative, options);
    const exception = widthOnlyCases.get(key);
    if (actual === output) {
      if (exception) staleExceptions.push({ relative, options });
      continue;
    }
    if (!exception) {
      differences.push({ relative, options, expected: output, actual });
      continue;
    }

    matchedExceptions.add(key);
    assert.equal(sha256(source), exception.sourceSha256, `${relative}: source changed`);
    assert.equal(
      sha256(output),
      exception.nativeExpectedSha256,
      `${relative}: native output changed`,
    );
    assert.equal(
      sha256(actual),
      exception.pluginActualSha256,
      `${relative}: Prettier output changed`,
    );

    const parserOptions = {
      lang: core.langFromPath(filepath),
      sourceType: core.sourceTypeFromPath(filepath),
    };
    const expectedAst = core.parseSync(output.replace(/^\uFEFF/u, ""), parserOptions);
    const actualAst = core.parseSync(actual.replace(/^\uFEFF/u, ""), parserOptions);
    const expectedCanonical = core.printSync(
      expectedAst.program,
      {},
      expectedAst.meta,
    ).code;
    const actualCanonical = core.printSync(
      actualAst.program,
      {},
      actualAst.meta,
    ).code;
    assert.equal(
      expectedCanonical,
      actualCanonical,
      `${relative}: width-only outputs must preserve the same parsed AST`,
    );
    assert.equal(
      sha256(expectedCanonical),
      exception.canonicalAstSha256,
      `${relative}: canonical AST output changed`,
    );

    const fixedPoint = await prettier.format(actual, {
      plugins: [plugin],
      filepath,
      ...options,
    });
    assert.equal(fixedPoint, actual, `${relative}: plugin output must be a fixed point`);
    assert.equal(
      sha256(fixedPoint),
      exception.fixedPointSha256,
      `${relative}: fixed-point output changed`,
    );
    verifiedWidthOnly++;
  }
  const fixtures = new Set(cases.map(({ filepath }) => filepath)).size;
  const binding =
    process.env.NAPI_RS_FORCE_WASI === "error" ? "Wasm" : "Native";
  t.diagnostic(
    `${binding}: ${fixtures} fixtures, ${cases.length} option sets, ${verifiedWidthOnly} verified width-only differences, ${differences.length} unexpected byte differences`,
  );
  assert.deepEqual(staleExceptions, []);
  assert.deepEqual([...matchedExceptions].sort(), [...widthOnlyCases.keys()].sort());
  assert.deepEqual(differences, []);
});
