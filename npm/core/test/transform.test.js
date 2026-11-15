import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import test from "node:test";

import { encodeSource } from "@swc-next/protocol/source";
import { encodeTransformApiOptions } from "@swc-next/protocol/transform";
import { decodeJsOutput } from "@swc-next/protocol/output";
import { transformSync } from "@swc-next/core";
import { transformSync as transformNative } from "../dist/binding.js";
import { transformSync as transformStandalone } from "@swc-next/transformer";

function transformProtocol(source, options) {
  const config = encodeTransformApiOptions(options);
  return decodeJsOutput(transformNative(encodeSource(source), source.length, config));
}

test("transform transports nested configurations", () => {
  const options = {
    parser: { lang: "ts", sourceType: "script" },
    generator: { asciiOnly: true, minify: true },
    comments: "none",
  };
  for (const transform of [transformSync, transformStandalone, transformProtocol]) {
    const output = transform('/* comment */ const value: string = "雪🌍";', options);
    assert.deepEqual(output, { code: 'const value:string="\\u96EA\\u{1F30D}";', diagnostics: [] });

    const defaults = transform("const value = 1;");
    assert.deepEqual(defaults, { code: "const value = 1;\n", diagnostics: [] });
  }
});

test("transform transports source maps and diagnostics", () => {
  const source = "const value = 1;";
  const filename = "雪🌍.js";
  const options = { sourceMap: true, filename };
  for (const transform of [transformSync, transformStandalone, transformProtocol]) {
    const output = transform(source, options);
    assert.equal(output.code, "const value = 1;\n");
    assert.deepEqual(JSON.parse(output.map).sources, [filename]);
    assert.deepEqual(output.diagnostics, []);

    const invalid = transform("const = 1;");
    assert.equal(invalid.code, "");
    assert.equal(invalid.diagnostics[0].severity, "error");
    assert.equal(invalid.diagnostics[0].start, 6);
    assert.equal(invalid.diagnostics[0].end, 7);
    assert.equal(typeof invalid.diagnostics[0].message, "string");
  }
});

test("transform validates configuration values", () => {
  for (const transform of [transformSync, transformStandalone, transformProtocol]) {
    assert.throws(() => transform("", { parser: { lang: "invalid" } }), TypeError);

    assert.throws(() => transform("", { generator: { minify: 1 } }), TypeError);
  }
});

test("standalone transform loads without runtime protocol dependencies", () => {
  const entry = import.meta.resolve("@swc-next/transformer");
  execFileSync(process.execPath, ["--input-type=module", "--eval", `
    import assert from "node:assert/strict";
    import { registerHooks } from "node:module";
    registerHooks({
      load(url, context, nextLoad) {
        assert(!url.includes("/packages/protocol/"));
        return nextLoad(url, context);
      },
    });
    const { transformSync } = await import(${JSON.stringify(entry)});
    assert.deepEqual(transformSync("const value = 1;", { generator: { minify: true } }), {
      code: "const value=1;",
      diagnostics: [],
    });
  `], { stdio: "pipe" });
});
