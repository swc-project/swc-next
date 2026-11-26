import assert from "node:assert/strict";
import test from "node:test";
import { NodeKind, parseSync, generateBuffersSync } from "../dist/ffi2.js";
import { createWalker, walk, NONE } from "@swc-next/walker2";

test("walker exposes numeric metadata alongside node views", () => {
  const ast = parseSync("value;");
  const program = ast.program;
  const ids = [];
  const types = [];
  assert.equal(walk(program, { enter(ctx) {
    assert.equal(ctx.nodeId, ctx.node.nodeId);
    assert.equal(ctx.ast, ast);
    assert.equal(typeof ctx.kind, "number");
    if (ctx.nodeId === ast.rootId) {
      assert.equal(ctx.parentId, NONE);
      assert.equal(ctx.key, null);
      assert.equal(ctx.index, null);
      assert.deepEqual(ctx.ancestorIds(), []);
    }
    ids.push(ctx.nodeId);
    types.push(ctx.type);
  } }), program);
  assert.deepEqual(types, ["Program", "ExpressionStatement", "IdentifierReference"]);

  walk(program, { IdentifierReference(ctx) {
    assert.equal(ctx.node.nodeId, ctx.nodeId);
    assert.equal(ctx.node.name, "value");
    assert.equal(ctx.parent.nodeId, ctx.parentId);
    assert.deepEqual(ctx.ancestors().map(view => view.nodeId), ctx.ancestorIds());
  } });
  assert.equal(ids.length, 3);
});

test("walker replacement writes through the FFI buffers", () => {
  const ast = parseSync("1;");
  const program = ast.program;
  const original = program.body.get(0).expression.nodeId;
  const replacement = ast.new_node(NodeKind.IdentifierReference);
  replacement.name = "changed";
  const entered = [], left = [];
  createWalker({
    NumericLiteral(ctx) {
      entered.push(ctx.nodeId);
      ctx.replace(replacement);
      assert.equal(ctx.nodeId, replacement.nodeId);
      assert.ok(ctx.node.isSame(replacement));
    },
    IdentifierReference: { leave(ctx) { left.push(ctx.nodeId); } },
  }).walk(program);
  assert.deepEqual(entered, [original]);
  assert.deepEqual(left, [replacement.nodeId]);
  assert.equal(program.body.get(0).expression.nodeId, replacement.nodeId);
  assert.equal(generateBuffersSync(...ast.encode()).code.trim(), "changed;");
});

test("walker keeps callback state and context lifetime", () => {
  const ast = parseSync("value;");
  const program = ast.program;
  const state = [];
  let escaped;
  const walker = createWalker({ IdentifierReference(ctx) {
    assert.equal(ctx.state, state);
    ctx.state.push(ctx.nodeId);
    escaped = ctx;
  } });
  walker.walk(program, state);
  assert.equal(state.length, 1);
  assert.throws(() => escaped.nodeId);
  assert.throws(() => escaped.node);
  assert.throws(() => walk({ type: "Program" }, {}), TypeError);
  assert.throws(() => walk(program, { enter() { return 1; } }), TypeError);
});

test("traversal resolves remaining fields after a kind change", () => {
  const ast = parseSync("run();");
  const program = ast.program;
  const types = [];
  let changed = false;
  walk(program, {
    enter(ctx) { types.push(ctx.type); },
    IdentifierReference(ctx) {
      if (changed || ctx.key !== "callee") { return; }
      changed = true;
      const parent = ctx.parent;
      parent.type = "TsInstantiationExpression";
      parent.set("expression", ctx.node);
      const argumentsNode = ast.new_node(NodeKind.TsTypeParameterInstantiation);
      argumentsNode.params.push(ast.new_node(NodeKind.TsStringKeyword));
      parent.set("typeArguments", argumentsNode);
    },
  });
  assert.deepEqual(types, [
    "Program", "ExpressionStatement", "CallExpression", "IdentifierReference",
    "TsTypeParameterInstantiation", "TsStringKeyword",
  ]);
  assert.equal(generateBuffersSync(...ast.encode()).code, "run<string>;\n");
});
