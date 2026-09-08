import assert from "node:assert/strict";
import test from "node:test";

import { decode } from "@swc-next/decoder";
import { CommentMode, Lang, parseSync, SourceType } from "../src-js/index.js";

test("decoder reads little-endian words from unaligned buffers", () => {
  const backing = new ArrayBuffer(73);
  const buffer = new Uint8Array(backing, 1, 72);
  const view = new DataView(backing, 1, 72);
  view.setUint32(0, 1, true); // node count
  view.setUint32(4, 1, true); // extra-data count
  view.setUint32(48, 0xffffffff, true); // no hashbang

  const result = decode(buffer, "");
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
  const buffer = new Uint8Array(65);
  const view = new DataView(buffer.buffer);
  view.setUint32(0, 1, true); // node count
  view.setUint32(24, 1, true); // diagnostic count
  buffer[64] = 0xff; // diagnostic severity

  const result = decode(buffer, "");
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

test("parseSync resolves unambiguous CommonJS wrapper syntax", () => {
  const options = { sourceType: SourceType.Unambiguous };

  const topLevelReturn = parseSync("return;", options);
  assert.equal(topLevelReturn.program.sourceType, SourceType.CommonJs);

  const newTarget = parseSync("() => new.target;", options);
  assert.equal(newTarget.program.sourceType, SourceType.CommonJs);

  const module = parseSync("export {}; new.target;", options);
  assert.equal(module.program.sourceType, SourceType.Module);
});

test("parseSync exposes flat comments lazily", () => {
  const source = '#!/usr/bin/env node\nconst emoji = "😀";\n// one 😀\n/* two */';
  const result = parseSync(source, { comments: CommentMode.Flat });
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
    comments: CommentMode.Flat,
    sourceType: SourceType.Script,
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
  const result = parseSync("// omitted", { comments: CommentMode.None });

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

test("parseSync reports diagnostics with UTF-16 offsets", () => {
  const source = '/* 文😀 */ const = 1';
  const result = parseSync(source);

  assert.equal(result.diagnostics.length, 1);
  assert.equal(result.diagnostics[0].start, source.indexOf("="));
  assert.equal(result.diagnostics[0].end, source.indexOf("=") + 1);
  assert.equal(result.comments[0].value, " 文😀 ");

  const noComments = parseSync(source, { comments: CommentMode.None });
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
  const result = parseSync(source, { lang: Lang.Jsx });

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

test("parseSync preserves multiple recoverable diagnostics", () => {
  const result = parseSync("const a; const b;");

  assert.equal(result.diagnostics.length, 2);
  assert.equal(result.diagnostics[0].severity, "error");
  assert.equal(result.diagnostics[1].severity, "error");
  assert.ok(result.diagnostics[0].start < result.diagnostics[1].start);
  assert.equal(result.program.body.length, 2);

  const ambient = parseSync("declare function f() {}", { lang: Lang.Dts });
  assert.equal(ambient.diagnostics.length, 1);
  assert.equal(ambient.diagnostics[0].severity, "error");
  assert.equal(ambient.diagnostics[0].help, undefined);
});


test("ambient implementation recovery preserves enclosing and following declarations", () => {
  for (const container of ["namespace N", 'module "pkg"']) {
    const source = `declare ${container} { function bad() {} function good(): void; } declare function after(): void;`;
    const result = parseSync(source, { lang: Lang.Ts, sourceType: SourceType.Module });
    assert.equal(result.diagnostics.length, 1);
    assert.equal(result.program.body.length, 2);
    const members = result.program.body[0].body.body;
    assert.deepEqual(members.map((node) => node.id.name), ["bad", "good"]);
    assert.equal(members[0].type, "TSDeclareFunction");
    assert.equal(members[0].body, null);
    assert.equal(result.program.body[1].id.name, "after");
  }
  const result = parseSync("declare function bad() {} declare function good(): void;", {
    lang: Lang.Dts,
    sourceType: SourceType.Module,
  });
  assert.equal(result.diagnostics.length, 1);
  assert.deepEqual(result.program.body.map((node) => node.id.name), ["bad", "good"]);
  assert.equal(result.program.body[0].declare, true);
  assert.equal(result.program.body[0].body, null);
});
