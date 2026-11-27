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
    assert.deepEqual(output, { code: 'const value="\\u96EA\\u{1F30D}";', diagnostics: [] });

    const preserved = transform('/* comment */ const value: string = "雪🌍";', {
      ...options,
      transformer: { typescript: false },
    });
    assert.deepEqual(preserved, {
      code: 'const value:string="\\u96EA\\u{1F30D}";',
      diagnostics: [],
    });

    const defaults = transform("const value = 1;");
    assert.deepEqual(defaults, { code: "const value = 1;\n", diagnostics: [] });
  }
});

test("transform forwards transformer v1 options", () => {
  for (const transform of [transformSync, transformStandalone, transformProtocol]) {
    const tsx = transform("export const App = (props: { x: number }) => <div {...props} />;", {
      filename: "App.tsx",
      transformer: {
        jsx: { runtime: "classic", pragma: "h" },
        module: { type: "commonjs" },
      },
    });
    assert.equal(tsx.diagnostics.length, 0);
    assert.match(tsx.code, /Object\.defineProperty\(exports, "App"/);
    assert.match(tsx.code, /h\("div", props\)/);

    const lowered = transform("class A { static { console.log(1); } }", {
      transformer: {
        target: "es2021",
        assumptions: { setPublicClassFields: true },
      },
    });
    assert.equal(lowered.diagnostics.length, 0);
    assert.doesNotMatch(lowered.code, /static \{/);

    const feature = transform("try {} catch {}", {
      transformer: { features: { "optional-catch-binding": true } },
    });
    assert.equal(feature.diagnostics.length, 0);
    assert.match(feature.code, /catch \([^)]+\)/);

    const lowAndHighFeature = transform("a ** b; /a/v;", {
      transformer: {
        features: {
          "exponentiation-operator": true,
          "unicode-sets-regex": true,
        },
      },
    });
    assert.equal(lowAndHighFeature.diagnostics.length, 0);
    assert.match(lowAndHighFeature.code, /Math\.pow\(a, b\)/);
    assert.match(lowAndHighFeature.code, /RegExp\("a", "v"\)/);

    const featureDisable = transform("a ?? b; a?.b;", {
      transformer: {
        target: "es2019",
        features: { "nullish-coalescing-operator": false },
      },
    });
    assert.equal(featureDisable.diagnostics.length, 0);
    assert.match(featureDisable.code, /\?\?/);
    assert.doesNotMatch(featureDisable.code, /\?\./);

    const engineTarget = transform("a ?? b;", {
      transformer: { target: { node: "12" } },
    });
    assert.equal(engineTarget.diagnostics.length, 0);
    assert.doesNotMatch(engineTarget.code, /\?\?/);

    const queryList = transform("a?.b;", {
      transformer: { target: ["chrome 70", "chrome 120"] },
    });
    assert.equal(queryList.diagnostics.length, 0);
    assert.doesNotMatch(queryList.code, /\?\./);

    const decorators = transform("@dec class A {}", {
      transformer: { decorators: { version: "legacy" } },
    });
    assert.equal(decorators.diagnostics.length, 0);
    assert.doesNotMatch(decorators.code, /@dec/);

    const invalidTarget = transform("const x = 1;", {
      transformer: { target: "not-a-browser 1" },
    });
    assert.equal(invalidTarget.code, "");
    assert.equal(invalidTarget.diagnostics[0].severity, "error");
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

    assert.throws(() => transform("", { transformer: true }), TypeError);

    assert.throws(() => transform("", { unknownOption: true }), TypeError);

    assert.throws(() => transform("", { parser: { unknownOption: true } }), TypeError);

    assert.throws(() => transform("", { generator: { unknownOption: true } }), TypeError);

    assert.throws(() => transform("", { transformer: { unknownOption: true } }), TypeError);

    assert.throws(
      () => transform("", { transformer: { typescript: { preserveConstEnums: true } } }),
      TypeError,
    );
  }
});

test("transform borrows overlapping source and configuration views", () => {
  const source = "const value = 1;";
  const config = encodeTransformApiOptions({ filename: source });
  const bytes = new Uint8Array(config);
  const sourceBytes = bytes.subarray(20);
  const output = decodeJsOutput(transformNative(sourceBytes, source.length, bytes));
  assert.deepEqual(output, { code: "const value = 1;\n", diagnostics: [] });
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
