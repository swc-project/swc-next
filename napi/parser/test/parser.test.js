import assert from "node:assert/strict";
import test from "node:test";

import { decode } from "@swc-next/decoder";
import { CommentMode, parseSync, SourceType } from "../src-js/index.js";

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
