import assert from "node:assert/strict";
import nodeTest from "node:test";

import { transformSync as transformCore } from "../src-js/index.js";
import { transformSync as transformStandalone } from "../../transformer/src-js/index.js";
import { transformSync as transformBinding } from "../src-js/binding.js";
import { createTransformOptionsEncoder, decodeTransform } from "@swc-next/decoder/transform";

nodeTest("transform bridge uses packed options and binary output", () => {
  const encodeOptions = createTransformOptionsEncoder();
  const config = encodeOptions({ filename: "组件😀.ts", sourceMap: true, generator: { minify: true } });
  const output = transformBinding(Buffer.from('const text: string = "雪😀";'), 'const text: string = "雪😀";'.length, config);
  assert.ok(output instanceof Uint8Array);
  const result = decodeTransform(output);
  assert.equal(result.code, 'const text="雪😀";');
  assert.deepEqual(JSON.parse(result.map).sources, ["组件😀.ts"]);

  const empty = encodeOptions({});
  assert.equal(empty.byteLength, 48);
  assert.equal(new DataView(empty.buffer, empty.byteOffset).getUint32(0, true), 1);
});

nodeTest("transform option scratch buffers recover after growth and validation errors", () => {
  const encodeOptions = createTransformOptionsEncoder();
  encodeOptions({ filename: "路径😀".repeat(4096) + ".tsx", transformer: { jsx: { importSource: "preact" } } });
  assert.throws(() => encodeOptions({ transformer: { jsx: { development: 1 } } }), TypeError);
  const result = transformCore("const x = 1;", {});
  assert.equal(result.code, "const x = 1;\n");
  assert.deepEqual(result.diagnostics, []);
});

for (const [name, transformSync] of [["core", transformCore], ["transformer", transformStandalone]]) {
  const test = (title, run) => nodeTest(`${name}: ${title}`, run);

  test("transformSync accepts omitted and null options", () => {
    const expected = { code: "const answer = 42;\n", diagnostics: [] };
    assert.deepEqual(transformSync("const answer = 42;"), expected);
    assert.deepEqual(transformSync("const answer = 42;", null), expected);
  });

  test("transformSync validates source and options", () => {
    assert.throws(() => transformSync(42), TypeError);
    assert.throws(() => transformSync("", "{}"), TypeError);
    assert.throws(() => transformSync("", []), TypeError);
    assert.throws(() => transformSync("", { unknown: true }), Error);
    assert.throws(() => transformSync("", { parser: { lang: "unknown" } }), Error);
    assert.throws(() => transformSync("", { transformer: { jsx: 1 } }), Error);
    assert.throws(() => transformSync("", { transformer: { features: { unknown: true } } }), Error);
  });

  test("transformSync forwards filename and parser language overrides", () => {
    const source = "const answer: number = 42;";
    assert.equal(transformSync(source, { filename: "answer.ts" }).code, "const answer = 42;\n");
    assert.equal(transformSync(source, { filename: "answer.js", parser: { lang: "ts" } }).code, "const answer = 42;\n");
    assert.equal(transformSync(source, { filename: "answer.ts", transformer: { typescript: false } }).code, "const answer: number = 42;\n");

    const commonjs = "return 42;";
    assert.equal(transformSync(commonjs, { filename: "answer.cjs" }).diagnostics.length, 0);
    assert.ok(transformSync(commonjs, { filename: "answer.cjs", parser: { sourceType: "module" } }).diagnostics.length > 0);
  });

  test("transformSync forwards JSX runtime and boolean options", () => {
    const source = "const element = <div />;";
    const automatic = transformSync(source, { filename: "view.jsx", transformer: { jsx: true } });
    assert.equal(automatic.diagnostics.length, 0);
    assert.ok(automatic.code.includes("react/jsx-runtime"));

    const classic = transformSync(source, {
      filename: "view.jsx",
      transformer: { jsx: { runtime: "classic", pragma: "h", pure: false } },
    });
    assert.ok(classic.code.includes('h("div", null)'));
    assert.ok(!classic.code.includes("react/jsx-runtime"));

    assert.equal(transformSync(source, { filename: "view.jsx", transformer: { jsx: false } }).code, "const element = <div />;\n");
  });

  test("transformSync forwards TypeScript options", () => {
    const source = 'import { value } from "module";';
    assert.equal(transformSync(source, { filename: "input.ts", transformer: { typescript: true } }).code, "export {};\n");
    assert.equal(transformSync(source, { filename: "input.ts", transformer: { typescript: { verbatimModuleSyntax: true } } }).code, source + "\n");
  });

  test("transformSync forwards feature overrides", () => {
    const source = "try {} catch {}";
    const lowered = transformSync(source, { transformer: { features: { "optional-catch-binding": true } } });
    assert.match(lowered.code, /catch\s*\(/);

    const preserved = transformSync(source, { transformer: { features: { "optional-catch-binding": false } } });
    assert.ok(!preserved.code.includes("catch ("));

    const staticBlock = transformSync("class C { static { Object.freeze(this); } }", {
      transformer: {
        features: { "class-static-block": true },
        proposal: { preventExtensions: true },
        assumptions: { noDocumentAll: true },
      },
    });
    assert.equal(staticBlock.diagnostics.length, 0);
    assert.ok(staticBlock.code.includes("_staticBlock()"));
    assert.ok(!staticBlock.code.includes("static {"));
  });

  test("transformSync forwards JSX import source and React Refresh options", () => {
    const result = transformSync("export function App() { return <div />; }", {
      filename: "view.jsx",
      transformer: {
        jsx: {
          importSource: "preact",
          development: true,
          refresh: { refreshReg: "register", refreshSig: "signature", emitFullSignatures: true },
        },
      },
    });
    assert.equal(result.diagnostics.length, 0);
    assert.ok(result.code.includes("preact/jsx-dev-runtime"));
    assert.ok(result.code.includes('register(_c, "App")'));
  });

  test("transformSync forwards comment and generator options", () => {
    const source = '// 中文😀\nconst text = "雪😀";';
    assert.ok(transformSync(source).code.includes("// 中文😀"));

    const compact = transformSync(source, { comments: "none", generator: { minify: true, asciiOnly: true } });
    assert.equal(compact.diagnostics.length, 0);
    assert.ok(!compact.code.includes("//"));
    assert.ok(!compact.code.includes("雪"));
    assert.ok(!compact.code.includes("😀"));
    assert.ok(!compact.code.includes("\n"));
  });

  test("transformSync returns source maps with Unicode filenames and content", () => {
    const source = 'const text: string = "雪😀";';
    const result = transformSync(source, { filename: "组件😀.ts", sourceMap: true });
    const map = JSON.parse(result.map);
    assert.equal(map.version, 3);
    assert.deepEqual(map.sources, ["组件😀.ts"]);
    assert.deepEqual(map.sourcesContent, [source]);
    assert.ok(map.mappings.length > 0);
    assert.equal(result.code, 'const text = "雪😀";\n');
  });

  test("transformSync returns diagnostics in UTF-16 source offsets", () => {
    const source = 'const text = "雪😀"; const text = 1;';
    const result = transformSync(source, { parser: { semanticErrors: true } });
    assert.equal(result.code, "");
    assert.equal(result.diagnostics.length, 1);
    const diagnostic = result.diagnostics[0];
    assert.equal(diagnostic.severity, "error");
    assert.equal(diagnostic.start, source.lastIndexOf("text"));
    assert.equal(diagnostic.end, diagnostic.start + 4);
    const original = diagnostic.labels.find((label) => label.start === source.indexOf("text"));
    assert.ok(original);
    assert.equal(original.end, original.start + 4);
    const duplicate = diagnostic.labels.find((label) => label.start === source.lastIndexOf("text"));
    assert.ok(duplicate);
    assert.equal(duplicate.end, duplicate.start + 4);

    const invalid = '"雪😀"; const = 1;';
    const syntax = transformSync(invalid);
    assert.equal(syntax.code, "");
    assert.ok(syntax.diagnostics.some((diagnostic) => diagnostic.severity === "error"));
    assert.equal(syntax.diagnostics[0].start, invalid.indexOf("="));
  });

  test("transformSync preserves strings across repeated native calls", () => {
    const first = transformSync('const text = "中文😀\\0";');
    transformSync('const other = "é";');
    assert.equal(first.code, 'const text = "中文😀\\0";\n');
    assert.deepEqual(first.diagnostics, []);

    assert.equal(transformSync('const text = "\uD800";').code, 'const text = "�";\n');

    const longText = "a".repeat(4096) + "中文😀";
    const longSource = `const text = "${longText}";`;
    assert.equal(transformSync(longSource).code, longSource + "\n");
  });
}
