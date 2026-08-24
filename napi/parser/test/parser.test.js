import assert from "node:assert/strict";
import test from "node:test";

import {
  CommentMode,
  parseExpressionFragmentSync,
  parseStatementFragmentSync,
  parseSync,
  SourceType,
} from "../src-js/index.js";

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

test("parseExpressionFragmentSync applies its function context", () => {
  const source = "/* leading */ yield await value";
  const result = parseExpressionFragmentSync(
    source,
    { function: { async: true, generator: true } },
    { sourceType: SourceType.Script },
  );
  const descriptor = Object.getOwnPropertyDescriptor(result, "expression");

  assert.equal(typeof descriptor.get, "function");
  assert.equal(result.expression.type, "YieldExpression");
  assert.equal(result.expression.argument.type, "AwaitExpression");
  assert.equal(result.expression.argument.argument.name, "value");
  assert.equal(result.expression, result.expression);
  assert.equal(result.comments[0].value, " leading ");

  const topLevel = parseExpressionFragmentSync(
    "await value",
    {},
    { sourceType: SourceType.Module },
  );
  assert.equal(topLevel.expression.type, "AwaitExpression");
});

test("parseStatementFragmentSync applies its function context", () => {
  const result = parseStatementFragmentSync(
    "return value;",
    { function: true },
    { sourceType: SourceType.Script },
  );

  assert.equal(result.statement.type, "ReturnStatement");
  assert.equal(result.statement.argument.name, "value");
  assert.throws(
    () =>
      parseStatementFragmentSync(
        "return value;",
        { function: false },
        { sourceType: SourceType.Script },
      ),
    /only valid inside a function/,
  );
});
