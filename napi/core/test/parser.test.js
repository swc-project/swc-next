import assert from "node:assert/strict";
import test from "node:test";

import { parseSync } from "../src-js/index.js";

function parseTsx(source, preserveParens) {
  return parseSync(source, {
    comments: "none",
    lang: "tsx",
    preserveParens,
    sourceType: "script",
  }).program;
}

test("TypeScript ESTree programs start at their first statement", () => {
  const source = '/// <reference path="react.d.ts" />\n\nconst view = <div />;';
  const program = parseTsx(source, false);

  assert.equal(program.start, source.indexOf("const"));
  assert.equal(program.body[0].start, program.start);
});

test("empty TypeScript ESTree programs start at the end of trivia", () => {
  const source = "// no statements";
  const program = parseTsx(source, false);

  assert.equal(program.start, source.length);
  assert.equal(program.end, source.length);
});

test("preserveParens false unwraps TSX expressions and TypeScript types", () => {
  const source = [
    "type Children = (string | JSX.Element)[];",
    "const Component = () => (<div />);",
  ].join("\n");
  const program = parseTsx(source, false);

  assert.equal(program.body[0].typeAnnotation.elementType.type, "TSUnionType");
  assert.equal(program.body[1].declarations[0].init.body.type, "JSXElement");
});

test("preserveParens true retains TSX expression and TypeScript type wrappers", () => {
  const source = [
    "type Children = (string | JSX.Element)[];",
    "const Component = () => (<div />);",
  ].join("\n");
  const program = parseTsx(source, true);

  assert.equal(program.body[0].typeAnnotation.elementType.type, "TSParenthesizedType");
  assert.equal(
    program.body[1].declarations[0].init.body.type,
    "ParenthesizedExpression",
  );
});
