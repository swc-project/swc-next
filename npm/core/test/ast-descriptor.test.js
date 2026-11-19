import assert from "node:assert/strict";
import test from "node:test";
import { parseSync, generateSync, NodeKind } from "../dist/ffi2.js";

function parse(source) {
  const ast = parseSync(source);
  return { ast, declaration: ast.program.body.get(0).declarations.get(0) };
}

function code(ast) {
  return generateSync(ast.program).code;
}

test("node setters accept kind descriptors", () => {
  const { ast, declaration } = parse("let value;");
  declaration.init = { kind: NodeKind.IdentifierReference, name: "foo" };
  assert.equal(declaration.init.name, "foo");
  assert.equal(code(ast), "let value = foo;\n");
});

test("descriptors nest through node and list fields", () => {
  const { ast, declaration } = parse("let value;");
  declaration.init = {
    kind: NodeKind.ObjectExpression,
    properties: [
      { kind: NodeKind.ObjectProperty, key: { kind: NodeKind.IdentifierName, name: "a" }, value: { kind: NodeKind.NumericLiteral, value: 1 } },
    ],
  };
  assert.equal(code(ast), "let value = { a: 1 };\n");
});

test("list setters accept kind descriptors", () => {
  const { ast } = parse("let value;");
  ast.program.body.push({ kind: NodeKind.EmptyStatement });
  assert.equal(code(ast), "let value;\n;\n");
});

test("descriptors accept UTF-16 spans", () => {
  const { ast, declaration } = parse("let value;");
  declaration.init = { kind: NodeKind.IdentifierReference, name: "foo", start: 4, end: 9 };
  assert.equal(declaration.init.start, 4);
  assert.equal(declaration.init.end, 9);
  assert.equal(code(ast), "let value = foo;\n");
});

test("set accepts kind descriptors", () => {
  const { ast, declaration } = parse("let value;");
  declaration.set("init", { kind: NodeKind.IdentifierReference, name: "bar" });
  assert.equal(code(ast), "let value = bar;\n");
});

test("descriptors require a numeric kind", () => {
  const { declaration } = parse("let value;");
  assert.throws(() => { declaration.init = { name: "foo" }; }, /Expected a node view/);
});

test("scalar setters reject node descriptors", () => {
  const { declaration } = parse("let value;");
  assert.throws(() => { declaration.id.name = { kind: NodeKind.IdentifierReference, name: "foo" }; }, /Field cannot be assigned directly/);
});
