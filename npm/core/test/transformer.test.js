import assert from "node:assert/strict";
import nodeTest from "node:test";

import { transformSync as transformCore } from "../src-js/index.js";
import { transformSync as transformStandalone } from "../../transformer/src-js/index.js";
import { parseSync as parseBinding, transformSync as transformBinding } from "../src-js/binding.js";
import { decode } from "@swc-next/decoder";
import { encodeParserConfig } from "@swc-next/decoder/options";
import { encodeJsTransformApiOptions, decodeJsOutput } from "@swc-next/transformer-bridge";

nodeTest("transform bridge uses packed options and binary output", () => {
  const config = encodeJsTransformApiOptions({ filename: "组件😀.ts", sourceMap: true, generator: { minify: true } });
  const output = transformBinding(Buffer.from('const text: string = "雪😀";'), 'const text: string = "雪😀";'.length, config);
  assert.ok(output instanceof Uint8Array);
  const result = decodeJsOutput(output);
  assert.equal(result.code, 'const text="雪😀";');
  assert.deepEqual(JSON.parse(result.map).sources, ["组件😀.ts"]);

  const empty = encodeJsTransformApiOptions({});
  assert.equal(empty.byteLength, 16);
  assert.equal(new DataView(empty.buffer, empty.byteOffset).getUint32(0, true), 1);
});

nodeTest("transform option scratch buffers recover after growth and validation errors", () => {
  encodeJsTransformApiOptions({ filename: "路径😀".repeat(4096) + ".tsx", sourceMap: true });
  assert.throws(() => encodeJsTransformApiOptions({ sourceMap: 1 }), TypeError);
  const source = "const x = 1;";
  const output = transformBinding(Buffer.from(source), source.length, encodeJsTransformApiOptions({}));
  const result = decodeJsOutput(output);
  assert.equal(result.code, "const x = 1;\n");
  assert.deepEqual(result.diagnostics, []);
});

nodeTest("parser and transform diagnostics preserve metadata in offset buffers", () => {
  const offsetBuffer = (buffer) => {
    const padded = new Uint8Array(buffer.length + 1);
    padded.set(buffer, 1);
    return padded.subarray(1);
  };
  const compareDiagnostics = (source, lang) => {
    const sourceBytes = Buffer.from(source);
    const parser = { lang };
    const parsedBuffer = parseBinding(sourceBytes, source.length, encodeParserConfig(parser));
    const transformedBuffer = transformBinding(sourceBytes, source.length, encodeJsTransformApiOptions({ parser }));
    const parsed = decode(offsetBuffer(parsedBuffer), source, sourceBytes);
    const transformed = decodeJsOutput(offsetBuffer(transformedBuffer));
    assert.ok(transformed.diagnostics.length > 0);
    assert.deepEqual(transformed.diagnostics, parsed.diagnostics.map(({ help, ...diagnostic }) =>
      help === undefined ? diagnostic : { ...diagnostic, help }));
    assert.equal(parsed.diagnostics, parsed.diagnostics);
    return { parsed: parsed.diagnostics, transformed: transformed.diagnostics };
  };

  for (const lang of ["js", "ts"]) {
    const { transformed } = compareDiagnostics('"雪😀"; const a; const b;', lang);
    assert.equal(transformed.length, 2);
  }

  const ambient = compareDiagnostics("declare function f() {}", "dts");
  assert.equal(Object.hasOwn(ambient.parsed[0], "help"), true);
  assert.equal(Object.hasOwn(ambient.transformed[0], "help"), false);

  for (const lang of ["jsx", "tsx"]) {
    const { transformed } = compareDiagnostics("/* 😀 */ <开始></结束>", lang);
    assert.ok(transformed[0].help.length > 0);
    assert.equal(transformed[0].labels.length, 1);
    assert.ok(transformed[0].message.includes("结束"));
  }
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
    assert.throws(() => transformSync("", { filename: 1 }), TypeError);
    assert.throws(() => transformSync("", { sourceMap: 1 }), TypeError);
    assert.throws(() => transformSync("", { generator: { minify: 1 } }), TypeError);
    assert.throws(() => transformSync("", { parser: { errorRecovery: true } }), TypeError);
    assert.throws(() => transformSync("", { generator: { unknown: true } }), TypeError);
  });

  test("transformSync rejects Rust transformation options", () => {
    assert.throws(() => transformSync("", { transformer: {} }), TypeError);
    assert.throws(() => transformSync("", { transformer: { typescript: false } }), TypeError);
    assert.throws(() => transformSync("", { jsx: { runtime: "classic" } }), TypeError);
  });

  test("transformSync forwards filename and parser language overrides", () => {
    const source = "const answer: number = 42;";
    assert.equal(transformSync(source, { filename: "answer.ts" }).code, "const answer = 42;\n");
    assert.equal(transformSync(source, { filename: "answer.js", parser: { lang: "ts" } }).code, "const answer = 42;\n");

    const commonjs = "return 42;";
    assert.equal(transformSync(commonjs, { filename: "answer.cjs" }).diagnostics.length, 0);
    assert.ok(transformSync(commonjs, { filename: "answer.cjs", parser: { sourceType: "module" } }).diagnostics.length > 0);
  });

  test("transformSync enables JSX defaults from the language", () => {
    const source = "export const element = <div />;";
    const automatic = transformSync(source, { filename: "view.jsx" });
    assert.equal(automatic.diagnostics.length, 0);
    assert.ok(automatic.code.includes("react/jsx-runtime"));
    assert.deepEqual(transformSync(source, { parser: { lang: "jsx" } }), automatic);
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
