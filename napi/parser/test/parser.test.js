import assert from "node:assert/strict";
import test from "node:test";

import { CommentMode, parseSync } from "../src-js/index.js";

test("parseSync parses JavaScript", () => {
  const { program } = parseSync("const answer = 42;");

  assert.equal(program.type, "Program");
  assert.equal(program.body[0].type, "VariableDeclaration");
});

test("parseSync exposes flat comments lazily", () => {
  const source = `#!/usr/bin/env node
const emoji = "😀";
// one 😀
/* two */`;
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

test("parseSync defaults to flat comments and returns an empty array", () => {
  const result = parseSync("const answer = 42;");

  assert.deepEqual(result.comments, []);
  assert.equal(result.comments, result.comments);
});

test("parseSync rejects unsupported comment modes", () => {
  assert.throws(() => parseSync("", { comments: "none" }));
  assert.throws(() => parseSync("", { comments: "attached" }));
});
