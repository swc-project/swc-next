import assert from "node:assert/strict";
import test from "node:test";

import { decode } from "@swc-next/decoder";
import * as core from "../src-js/index.js";
import { langFromPath, parseSync, sourceTypeFromPath } from "../src-js/index.js";
import { parseSync as parseStandalone } from "../../parser/src-js/index.js";

test("parser options use strings without enum value exports", () => {
  assert.equal("Lang" in core, false);
  assert.equal("SourceType" in core, false);
  assert.equal("CommentMode" in core, false);
  assert.equal(langFromPath("input.ts"), "ts");
  assert.equal(sourceTypeFromPath("input.cjs"), "commonjs");

  const result = parseSync("// comment\nconst value: number = 1;", {
    lang: "ts",
    sourceType: "script",
    comments: "none",
  });
  assert.equal(result.meta.lang, "ts");
  assert.equal(result.meta.sourceType, "script");
  assert.deepEqual(result.comments, []);
  assert.equal(result.program.body[0].declarations[0].id.typeAnnotation.type, "TSTypeAnnotation");
});

test("decoder reads little-endian words from unaligned buffers", () => {
  const backing = new ArrayBuffer(77);
  const buffer = new Uint8Array(backing, 1, 76);
  const view = new DataView(backing, 1, 76);
  view.setUint32(0, 1, true); // node count
  view.setUint32(4, 1, true); // extra-data count
  view.setUint32(60, 0xffffffff, true); // no hashbang

  const result = decode(buffer, "", new Uint8Array());
  assert.deepEqual(result.program, {
    type: "Program",
    start: 0,
    end: 0,
    sourceType: "module",
    hashbang: null,
    body: [],
  });
  assert.deepEqual(result.comments, []);
});

test("decoder rejects unknown diagnostic severities", () => {
  const buffer = new Uint8Array(69);
  const view = new DataView(buffer.buffer);
  view.setUint32(0, 1, true); // node count
  view.setUint32(24, 1, true); // diagnostic count
  buffer[68] = 0xff; // diagnostic severity

  const result = decode(buffer, "", new Uint8Array());
  assert.throws(
    () => result.diagnostics,
    new RangeError("Unknown diagnostic severity 255"),
  );
});

test("parseSync exposes the program lazily", () => {
  const result = parseSync("const answer = 42;");
  const descriptor = Object.getOwnPropertyDescriptor(result, "program");

  assert.equal(typeof descriptor.get, "function");
  assert.equal(descriptor.set, undefined);
  assert.equal(descriptor.enumerable, true);
  assert.equal(descriptor.configurable, true);

  const program = result.program;
  assert.equal(program.type, "Program");
  assert.equal(program.body[0].type, "VariableDeclaration");
  assert.equal(result.program, program);
});

test("parseSync decodes ASCII strings after non-ASCII source text", () => {
  const longName = "longAsciiIdentifierAfterUnicodePrefix".repeat(3);
  const source = `/* 😀 */ const asciiName = "plain"; const escaped = "\\u0061"; const 文 = "é"; const ${longName} = 1;`;

  for (const lang of ["js", "ts"]) {
    const declarations = parseSync(source, { lang }).program.body;
    assert.equal(declarations[0].declarations[0].id.name, "asciiName");
    assert.equal(declarations[0].declarations[0].init.value, "plain");
    assert.equal(declarations[1].declarations[0].id.name, "escaped");
    assert.equal(declarations[1].declarations[0].init.value, "a");
    assert.equal(declarations[2].declarations[0].id.name, "文");
    assert.equal(declarations[2].declarations[0].init.value, "é");
    assert.equal(declarations[3].declarations[0].id.name, longName);
  }
});

test("parseSync decodes UTF-8 source and pool strings", () => {
  const longValue = "é".repeat(40);
  const source = `const short = "é"; const long = "${longValue}"; const escaped = "\\uFEFF";`;
  const declarations = parseSync(source).program.body;

  assert.equal(declarations[0].declarations[0].init.value, "é");
  assert.equal(declarations[1].declarations[0].init.value, longValue);
  assert.equal(declarations[2].declarations[0].init.value, "\uFEFF");
});

test("parseSync exposes flat comments lazily", () => {
  const source =
    '#!/usr/bin/env node\nconst emoji = "😀";\n// one 😀\n/* two */';
  const result = parseSync(source, { comments: "flat" });
  const descriptor = Object.getOwnPropertyDescriptor(result, "comments");

  assert.equal(typeof descriptor.get, "function");
  assert.equal(descriptor.set, undefined);
  assert.equal(descriptor.enumerable, true);
  assert.equal(descriptor.configurable, true);

  const lineStart = source.indexOf("// one 😀");
  const blockStart = source.indexOf("/* two */");
  assert.deepEqual(result.comments, [
    {
      type: "Line",
      value: " one 😀",
      start: lineStart,
      end: lineStart + "// one 😀".length,
    },
    {
      type: "Block",
      value: " two ",
      start: blockStart,
      end: blockStart + "/* two */".length,
    },
  ]);
  assert.equal(result.comments, result.comments);
});

test("parseSync exposes HTML comments without delimiters", () => {
  const source = "<!-- open\n--> close\n";
  const result = parseSync(source, {
    comments: "flat",
    sourceType: "script",
  });

  assert.deepEqual(result.comments, [
    { type: "Line", value: " open", start: 0, end: 9 },
    { type: "Line", value: " close", start: 10, end: 19 },
  ]);
});

test("parseSync defaults to flat comments and returns an empty array", () => {
  const result = parseSync("const answer = 42;");

  assert.deepEqual(result.comments, []);
  assert.equal(result.comments, result.comments);
});

test("parseSync can disable comment collection", () => {
  const result = parseSync("// omitted", { comments: "none" });

  assert.deepEqual(result.comments, []);
});

test("parseSync exposes parser diagnostics lazily", () => {
  const result = parseSync("const = 1");
  const descriptor = Object.getOwnPropertyDescriptor(result, "diagnostics");

  assert.equal(typeof descriptor.get, "function");
  assert.equal(descriptor.set, undefined);
  assert.equal(descriptor.enumerable, true);
  assert.equal(result.diagnostics.length, 1);
  const [error] = result.diagnostics;
  assert.equal(error.severity, "error");
  assert.ok(error.message.length > 0);
  assert.equal(error.start, 6);
  assert.equal(error.end, 7);
  assert.deepEqual(error.labels, []);
  assert.equal(result.diagnostics, result.diagnostics);
  assert.equal(result.program.type, "Program");
});

test("parseSync returns an empty diagnostics array for valid input", () => {
  const result = parseSync("const answer = 42;");

  assert.deepEqual(result.diagnostics, []);
  assert.equal(result.diagnostics, result.diagnostics);

  const empty = parseSync("");
  assert.deepEqual(empty.diagnostics, []);
});

test("parseSync makes semantic early-error checking optional", () => {
  const source = "let value; let value;";
  assert.deepEqual(parseSync(source).diagnostics, []);
  assert.deepEqual(parseSync(source, { semanticErrors: false }).diagnostics, []);

  const checked = parseSync(source, { semanticErrors: true });
  assert.equal(checked.diagnostics.length, 1);
  assert.equal(checked.diagnostics[0].severity, "error");
  assert.equal(checked.program.body.length, 2);
  assert.deepEqual(checked.program, parseSync(source).program);

  const valid = parseSync("let value; { let value; }", { semanticErrors: true });
  assert.deepEqual(valid.diagnostics, []);
  const missingExport = "export { missing };";
  assert.deepEqual(parseSync(missingExport).diagnostics, []);
  assert.equal(parseSync(missingExport, { semanticErrors: true }).diagnostics.length, 1);
  assert.throws(() => parseSync(source, { semanticErrors: 1 }), TypeError);
});

test("semantic checking supports TypeScript through both parser entry points", () => {
  const source = "type Value = string; type Value = number;";
  const options = { lang: "ts", semanticErrors: true };
  const checked = parseSync(source, options);
  assert.deepEqual(parseSync(source, { lang: "ts" }).diagnostics, []);
  assert.deepEqual(parseSync(source, { ...options, semanticErrors: false }).diagnostics, []);
  assert.equal(checked.diagnostics.length, 1);
  assert.equal(checked.diagnostics[0].severity, "error");
  assert.deepEqual(checked.program, parseSync(source, { lang: "ts" }).program);
  assert.deepEqual(parseStandalone(source, options).diagnostics, checked.diagnostics);
  assert.deepEqual(parseStandalone(source, options).program, checked.program);
  assert.deepEqual(parseStandalone(source, { ...options, semanticErrors: false }).diagnostics, []);

  const valid = parseSync("interface Value {} interface Value {}", options);
  assert.deepEqual(valid.diagnostics, []);

  const invalid = "type Value = ;";
  assert.deepEqual(
    parseSync(invalid, options).diagnostics,
    parseSync(invalid, { lang: "ts" }).diagnostics,
  );
});

test("semantic diagnostics preserve Unicode offsets and related labels", () => {
  const source = "/* 😀 */ let 文; let 文;";
  const result = parseSync(source, { semanticErrors: true, comments: "none" });
  const [error] = result.diagnostics;
  assert.equal(error.start, source.lastIndexOf("文"));
  assert.equal(error.end, source.lastIndexOf("文") + 1);
  assert.ok(error.labels.some((label) => label.start === source.indexOf("文")));
  assert.ok(error.message.includes("文"));
  assert.deepEqual(result.comments, []);
});

test("semantic checking appends diagnostics after parser recovery", () => {
  function checkRecovery(source, lang) {
    const unchecked = parseSync(source, { lang });
    const checked = parseSync(source, { lang, semanticErrors: true });
    assert.equal(unchecked.diagnostics.length, 1);
    assert.equal(checked.diagnostics.length, 2);
    assert.deepEqual(checked.diagnostics.slice(0, 1), unchecked.diagnostics);
    assert.equal(checked.diagnostics[1].severity, "error");
    assert.deepEqual(checked.program, unchecked.program);
    assert.deepEqual(checked.comments, unchecked.comments);
    assert.deepEqual(
      parseStandalone(source, { lang, semanticErrors: true }).diagnostics,
      checked.diagnostics,
    );
  }

  checkRecovery("/* 😀 */ const a; let 文; let 文;", "js");
  checkRecovery("/* 😀 */ const a; type Value = string; type Value = number;", "ts");
});

test("parseSync reports diagnostics with UTF-16 offsets", () => {
  const source = "/* 文😀 */ const = 1";
  const result = parseSync(source);

  assert.equal(result.diagnostics.length, 1);
  assert.equal(result.diagnostics[0].start, source.indexOf("="));
  assert.equal(result.diagnostics[0].end, source.indexOf("=") + 1);
  assert.equal(result.comments[0].value, " 文😀 ");

  const noComments = parseSync(source, { comments: "none" });
  assert.deepEqual(noComments.diagnostics, result.diagnostics);
  assert.deepEqual(noComments.comments, []);

  const eofSource = '"文😀"; const value =';
  const eof = parseSync(eofSource);
  assert.equal(eof.diagnostics.length, 1);
  assert.equal(eof.diagnostics[0].start, eofSource.length);
  assert.equal(eof.diagnostics[0].end, eofSource.length);
});

test("parseSync preserves diagnostic help, labels, and Unicode messages", () => {
  const source = "/* 😀 */ <开始></结束>";
  const result = parseSync(source, { lang: "jsx" });

  assert.equal(result.diagnostics.length, 1);
  const [error] = result.diagnostics;
  assert.equal(error.severity, "error");
  assert.ok(error.message.includes("开始"));
  assert.ok(error.message.includes("结束"));
  assert.ok(error.help.length > 0);
  assert.equal(error.start, source.indexOf("结束"));
  assert.equal(error.end, source.indexOf("结束") + 2);
  assert.equal(error.labels.length, 1);
  assert.ok(error.labels[0].message.length > 0);
  assert.equal(error.labels[0].start, source.indexOf("开始"));
  assert.equal(error.labels[0].end, source.indexOf("开始") + 2);
});

test("parseSync decodes diagnostic arrays and optional metadata", () => {
  const result = parseSync("const a; const b;");

  assert.equal(result.diagnostics.length, 2);
  assert.equal(result.diagnostics[0].severity, "error");
  assert.equal(result.diagnostics[1].severity, "error");
  assert.ok(result.diagnostics[0].start < result.diagnostics[1].start);

  const ambient = parseSync("declare function f() {}", { lang: "dts" });
  assert.equal(ambient.diagnostics.length, 1);
  assert.equal(ambient.diagnostics[0].severity, "error");
  assert.equal(ambient.diagnostics[0].help, undefined);
});

test("parseSync preserves source encoding at the API boundary", () => {
  const empty = parseSync("");
  assert.deepEqual(empty.program.body, []);
  assert.equal(empty.program.end, 0);

  const source = 'const 값 = "한글😀";';
  const result = parseSync(source);
  const standalone = parseStandalone(source);
  assert.equal(typeof Object.getOwnPropertyDescriptor(result, "program").get, "function");
  assert.equal(result.program.end, source.length);
  assert.equal(result.program.body[0].declarations[0].init.value, "한글😀");
  assert.deepEqual(standalone.program, result.program);

  const high = parseSync('"\ud800";');
  assert.equal(high.program.body[0].expression.value, "�");
  assert.equal(high.program.end, 4);

  const low = parseSync('"\udc00";');
  assert.equal(low.program.body[0].expression.value, "�");
  assert.equal(low.program.end, 4);

  assert.throws(() => parseSync(undefined), TypeError);
  assert.throws(() => parseSync(123), TypeError);
  assert.throws(() => parseStandalone(null), TypeError);
});

test("path helpers return string enum values through N-API", () => {
  assert.equal(langFromPath("input.js"), "js");
  assert.equal(langFromPath("input.jsx"), "jsx");
  assert.equal(langFromPath("input.ts"), "ts");
  assert.equal(langFromPath("input.tsx"), "tsx");
  assert.equal(langFromPath("input.d.ts"), "dts");
  assert.equal(sourceTypeFromPath("input.cjs"), "commonjs");
  assert.equal(sourceTypeFromPath("input.mjs"), "module");
});

test("path helpers preserve strings across the native boundary", () => {
  assert.equal(langFromPath(""), "js");
  assert.equal(langFromPath("한글😀/input.tsx"), "tsx");
  assert.equal(langFromPath("x\0.ts"), "ts");
  assert.equal(langFromPath("\ud800/input.ts"), "ts");
  assert.equal(sourceTypeFromPath("한글😀/input.cjs"), "commonjs");
  assert.equal(langFromPath("한".repeat(1362) + ".ts"), "ts");
  assert.equal(langFromPath("한글😀/".repeat(1000) + "input.tsx"), "tsx");
  assert.throws(() => langFromPath(123));
  assert.throws(() => sourceTypeFromPath(null));
});
