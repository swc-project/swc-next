import assert from "node:assert/strict";
import test from "node:test";
import { NodeKind, NodeView, parseSync, generateBuffersSync } from "../dist/index.js";
import { createWalker, walk, NONE } from "@swc-next/walker/ast";

test("numeric walker exposes metadata without creating node views", () => {
  const ast = parseSync("value;");
  const created = ast.materializedNodeCount;
  const ids = [];
  const types = [];
  assert.equal(walk(ast, ast.rootId, { enter(id, ctx) {
    assert.equal(ctx.nodeId, id);
    assert.equal(ctx.ast, ast);
    assert.equal(typeof ctx.kind, "number");
    if (id === ast.rootId) {
      assert.equal(ctx.parentId, NONE);
      assert.equal(ctx.key, null);
      assert.equal(ctx.index, null);
      assert.deepEqual(ctx.ancestorIds(), []);
    }
    ids.push(id);
    types.push(ctx.type);
  } }), ast.rootId);
  assert.deepEqual(types, ["Program", "ExpressionStatement", "Identifier"]);
  assert.equal(ast.materializedNodeCount, created);

  walk(ast, ast.rootId, { Identifier(id, ctx) {
    const first = ctx.node, second = ctx.node;
    assert.notEqual(first, second);
    assert.ok(NodeView.isSame(first, second));
    assert.equal(first.nodeId, id);
    assert.equal(first.name, "value");
    assert.equal(ctx.parent.nodeId, ctx.parentId);
    assert.deepEqual(ctx.ancestors().map(node => node.nodeId), ctx.ancestorIds());
  } });
  assert.equal(ids.length, 3);
});

test("numeric walker replacement writes through the FFI buffers", () => {
  const ast = parseSync("1;");
  const original = ast.program.body.get(0).expression.nodeId;
  const replacement = ast.new_node(NodeKind.IdentifierReference);
  replacement.name = "changed";
  const entered = [], left = [];
  createWalker({
    Literal(id, ctx) {
      entered.push(id);
      ctx.replace(replacement);
      assert.equal(ctx.nodeId, replacement.nodeId);
      assert.ok(NodeView.isSame(ctx.node, replacement));
    },
    Identifier: { leave(id) { left.push(id); } },
  }).walk(ast, ast.rootId);
  assert.deepEqual(entered, [original]);
  assert.deepEqual(left, [replacement.nodeId]);
  assert.equal(ast.program.body.get(0).expression.nodeId, replacement.nodeId);
  assert.equal(generateBuffersSync(...ast.encode()).code.trim(), "changed;");
});

test("numeric walker keeps callback state and context lifetime", () => {
  const ast = parseSync("value;");
  const state = [];
  let escaped;
  const walker = createWalker({ Identifier(id, ctx) {
    assert.equal(ctx.state, state);
    ctx.state.push(id);
    escaped = ctx;
  } });
  walker.walk(ast, ast.rootId, state);
  assert.equal(state.length, 1);
  assert.throws(() => escaped.nodeId);
  assert.throws(() => escaped.node);
  assert.throws(() => walk(ast, -1, {}), RangeError);
  assert.throws(() => walk(ast, NONE, {}), RangeError);
  assert.throws(() => walk(ast, ast.rootId, { enter() { return 1; } }), TypeError);
});


test("numeric traversal resolves remaining fields after a kind change", () => {
  const ast = parseSync("run();");
  const types = [];
  let changed = false;
  walk(ast, ast.rootId, {
    enter(_id, ctx) { types.push(ctx.type); },
    Identifier(_id, ctx) {
      if (changed || ctx.key !== "callee") { return; }
      changed = true;
      const parent = ctx.parent;
      parent.type = "TSInstantiationExpression";
      parent.set("expression", ctx.node);
      const argumentsNode = ast.new_node(NodeKind.TsTypeParameterInstantiation);
      argumentsNode.params.push(ast.new_node(NodeKind.TsStringKeyword));
      parent.set("typeArguments", argumentsNode);
    },
  });
  assert.deepEqual(types, [
    "Program", "ExpressionStatement", "CallExpression", "Identifier",
    "TSTypeParameterInstantiation", "TSStringKeyword",
  ]);
  assert.equal(generateBuffersSync(...ast.encode()).code, "run<string>;\n");
});
