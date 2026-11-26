import assert from "node:assert/strict";
import test from "node:test";
import { parseSync, generateSync } from "../dist/ffi2.js";

const typescript = { lang: "ts" };

function cloneInto(target, source, index = 0) {
  const cloned = target.clone(source.program.body.get(index));
  target.program.body.push(cloned);
  return cloned;
}

function code(ast) {
  return generateSync(ast.program).code;
}

test("clone copies a subtree across AST views", () => {
  const source = parseSync("const value = { nested: [1, 2] };");
  const target = parseSync("let placeholder = 0;");
  const cloned = cloneInto(target, source);
  assert.equal(cloned.ast, target);
  assert.equal(code(target), "let placeholder = 0;\nconst value = { nested: [1, 2] };\n");
  assert.equal(code(source), "const value = { nested: [1, 2] };\n");
});

test("clone copies native TypeScript fields", () => {
  const source = parseSync("function f(a: number): string { return `${a}`; }", typescript);
  const target = parseSync("", typescript);
  cloneInto(target, source);
  assert.equal(code(target), "function f(a: number): string {\n  return `${a}`;\n}\n");
});

test("clone duplicates a subtree within one AST", () => {
  const ast = parseSync("const x = 1;");
  cloneInto(ast, ast);
  assert.equal(code(ast), "const x = 1;\nconst x = 1;\n");
});

test("clone edits do not affect the source AST", () => {
  const source = parseSync("const value = 1;");
  const target = parseSync("");
  const cloned = cloneInto(target, source);
  cloned.declarators.get(0).id.name = "renamed";
  assert.equal(code(target), "const renamed = 1;\n");
  assert.equal(code(source), "const value = 1;\n");
});

test("clone preserves spans when the source text matches", () => {
  const source = parseSync("const value = 1;");
  const target = parseSync("const value = 1;");
  const cloned = target.clone(source.program.body.get(0));
  assert.equal(cloned.start, 0);
  assert.equal(cloned.end, 16);
  assert.equal(cloned.declarators.get(0).id.start, 6);
});

test("clone resets spans for a different source", () => {
  const source = parseSync("const value = 1;");
  const target = parseSync("let other = 2;");
  const cloned = target.clone(source.program.body.get(0));
  assert.equal(cloned.start, 0);
  assert.equal(cloned.end, 0);
  assert.equal(cloned.declarators.get(0).id.start, 0);
});

test("clone resets TypeScript template element spans for a different source", () => {
  const source = parseSync("`x${y}z`;", typescript);
  const target = parseSync("", typescript);
  const cloned = cloneInto(target, source);
  const quasis = cloned.expression.quasis;
  assert.equal(quasis.get(0).start, 0);
  assert.equal(quasis.get(0).end, 0);
  assert.equal(quasis.get(1).start, 0);
  assert.equal(quasis.get(1).end, 0);
  assert.equal(code(target), "`x${y}z`;\n");
});

test("cloned TypeScript template element spans accept writes", () => {
  const source = parseSync("`x`;", typescript);
  const target = parseSync("let abc = 0;", typescript);
  const cloned = cloneInto(target, source);
  const quasi = cloned.expression.quasis.get(0);
  assert.equal(quasi.start, 0);
  assert.equal(quasi.end, 0);
  quasi.end = 5;
  assert.equal(quasi.start, 0);
  assert.equal(quasi.end, 5);
  quasi.start = 2;
  assert.equal(quasi.start, 2);
  assert.equal(quasi.end, 5);
});

test("assigning a foreign node clones it into the target AST", () => {
  const source = parseSync("const value = 1;");
  const target = parseSync("let x = 0;");
  const foreign = source.program.body.get(0);
  target.program.body.push(foreign);
  assert.notEqual(target.program.body.get(1).nodeId, foreign.nodeId);
  assert.equal(code(target), "let x = 0;\nconst value = 1;\n");
  target.program.body.get(1).declarators.get(0).id.name = "renamed";
  assert.equal(code(source), "const value = 1;\n");
  assert.equal(code(target), "let x = 0;\nconst renamed = 1;\n");
});

test("foreign node fields clone into the target AST", () => {
  const source = parseSync("foo;");
  const target = parseSync("bar;");
  const foreign = source.program.body.get(0).expression;
  target.program.body.get(0).expression = foreign;
  assert.notEqual(target.program.body.get(0).expression.nodeId, foreign.nodeId);
  assert.equal(code(target), "foo;\n");
  assert.equal(code(source), "foo;\n");
});

test("clone rejects non-view inputs", () => {
  const ast = parseSync("");
  assert.throws(() => ast.clone({ type: "IdentifierReference" }), /Expected a NodeView/);
});
