import assert from "node:assert/strict";
import test from "node:test";
import { walk, visitorKeys } from "../src-js/ast.js";

const id = (name) => ({ type: "Identifier", name });
const statement = (name) => ({
  type: "ExpressionStatement",
  expression: id(name),
});
const program = (...names) => ({ type: "Program", body: names.map(statement) });
const bodyNames = (ast) => ast.body.map((node) => node.expression.name);

test("walk works without a parser or binding and visits only schema child keys", () => {
  const ast = program("first", "second");
  ast.metadata = id("not-a-child");
  const seen = [];
  const state = { visits: 0 };
  assert.equal(
    walk(
      ast,
      {
        Identifier(node, ctx) {
          seen.push(node.name);
          ctx.state.visits++;
          assert.equal(ctx.parent.type, "ExpressionStatement");
          assert.equal(ctx.key, "expression");
          assert.equal(ctx.index, null);
          assert.deepEqual(
            ctx.ancestors().map((node) => node.type),
            ["Program", "ExpressionStatement"],
          );
        },
      },
      state,
    ),
    ast,
  );
  assert.deepEqual(seen, ["first", "second"]);
  assert.equal(state.visits, 2);
  assert.ok(Object.isFrozen(visitorKeys));
  assert.ok(Object.isFrozen(visitorKeys.Program));
});

test("aliases surround typed hooks and global hooks surround aliases", () => {
  const calls = [];
  const ast = {
    type: "FunctionExpression",
    params: [],
    body: { type: "BlockStatement", body: [] },
  };
  walk(ast, {
    FunctionExpression: {
      enter() {
        calls.push("typed enter");
      },
      leave() {
        calls.push("typed leave");
      },
    },
    Function: {
      enter() {
        calls.push("alias enter");
      },
      leave() {
        calls.push("alias leave");
      },
    },
    enter(node) {
      if (node === ast) calls.push("global enter");
    },
    leave(node) {
      if (node === ast) calls.push("global leave");
    },
  });
  assert.deepEqual(calls, [
    "global enter",
    "alias enter",
    "typed enter",
    "typed leave",
    "alias leave",
    "global leave",
  ]);
});

test("replacement visits new children once and leaves with the replacement type", () => {
  const ast = program("before");
  ast.body[0].expression.start = 10;
  ast.body[0].expression.end = 16;
  const seen = [];
  walk(ast, {
    Identifier(node, ctx) {
      seen.push(`enter ${node.name}`);
      if (node.name === "before")
        ctx.replace({
          type: "UnaryExpression",
          operator: "!",
          prefix: true,
          argument: id("after"),
        });
    },
    UnaryExpression: {
      enter() {
        assert.fail("replacement root must not re-enter");
      },
      leave(node) {
        seen.push(`leave ${node.type}`);
      },
    },
  });
  assert.deepEqual(seen, [
    "enter before",
    "enter after",
    "leave UnaryExpression",
  ]);
  assert.equal(ast.body[0].expression.start, 10);
  assert.equal(ast.body[0].expression.end, 16);
});

test("global replacement stops remaining enter hooks; leave replacement is not traversed", () => {
  const ast = program("before");
  const seen = [];
  walk(ast, {
    enter(node, ctx) {
      if (node.type === "Identifier")
        ctx.replace({ type: "Literal", value: 1 });
    },
    Identifier() {
      assert.fail("old typed enter must not run");
    },
    Literal: {
      enter() {
        assert.fail("new typed enter must not run");
      },
      leave(node, ctx) {
        seen.push(node.value);
        ctx.replace(id("after"));
      },
    },
    leave(node) {
      assert.notEqual(node.type, "Identifier");
    },
  });
  assert.deepEqual(seen, [1]);
  assert.equal(ast.body[0].expression.name, "after");
});

test("skip retains leave; removal skips children and leave without losing siblings", () => {
  const ast = program("skip", "remove", "keep");
  const seen = [];
  walk(ast, {
    ExpressionStatement: {
      enter(node, ctx) {
        if (node.expression.name === "skip") ctx.skip();
        if (node.expression.name === "remove") ctx.remove();
      },
      leave(node) {
        seen.push(`leave ${node.expression.name}`);
      },
    },
    Identifier(node) {
      seen.push(node.name);
    },
  });
  assert.deepEqual(seen, ["leave skip", "keep", "leave keep"]);
  assert.deepEqual(bodyNames(ast), ["skip", "keep"]);
});

test("insertions preserve call order; before is unvisited and after is visited", () => {
  const ast = program("current", "tail");
  const seen = [];
  walk(ast, {
    ExpressionStatement(node, ctx) {
      if (node.expression.name !== "current") return;
      assert.equal(ctx.index, 0);
      ctx.insertBefore([statement("before1"), statement("before2")]);
      assert.equal(ctx.index, 2);
      ctx.insertAfter(statement("after1"));
      ctx.insertAfter(statement("after2"));
    },
    Identifier(node) {
      seen.push(node.name);
    },
  });
  assert.deepEqual(bodyNames(ast), [
    "before1",
    "before2",
    "current",
    "after1",
    "after2",
    "tail",
  ]);
  assert.deepEqual(seen, ["current", "after1", "after2", "tail"]);
});

test("insertions followed by removal and removal during leave retain subsequent visits", () => {
  const ast = program("current", "tail");
  const seen = [];
  walk(ast, {
    ExpressionStatement: {
      enter(node, ctx) {
        if (node.expression.name !== "current") return;
        ctx.insertBefore(statement("before"));
        ctx.insertAfter(statement("after"));
        ctx.remove();
      },
      leave(node, ctx) {
        if (node.expression.name === "after") ctx.remove();
      },
    },
    Identifier(node) {
      seen.push(node.name);
    },
  });
  assert.deepEqual(bodyNames(ast), ["before", "tail"]);
  assert.deepEqual(seen, ["after", "tail"]);
});

test("stop aborts siblings and pending leave hooks while retaining edits", () => {
  const ast = program("first", "second");
  walk(ast, {
    Identifier(node, ctx) {
      assert.equal(node.name, "first");
      node.name = "changed";
      ctx.stop();
    },
    leave() {
      assert.fail("no pending leave after stop");
    },
  });
  assert.deepEqual(bodyNames(ast), ["changed", "second"]);
});

test("contexts are reused within a walk and isolated across nested walks", () => {
  let saved;
  walk(program("one", "two"), {
    Identifier(node, ctx) {
      if (saved) {
        assert.equal(saved, ctx);
        assert.equal(saved.node, node);
      }
      saved = ctx;
      walk(program("nested"), {
        Identifier(nested, nestedCtx) {
          assert.notEqual(nestedCtx, ctx);
          assert.equal(ctx.node, node);
        },
      });
      assert.equal(ctx.node, node);
    },
  });
  assert.throws(() => saved.parent, /only valid/);
  assert.throws(() => saved.replace(id("bad")), /only valid/);
  assert.throws(
    () =>
      walk(program("error"), {
        Identifier(node, ctx) {
          saved = ctx;
          throw Error("sentinel");
        },
      }),
    /sentinel/,
  );
  assert.throws(() => saved.state, /only valid/);
});

test("structural edits reject roots and scalar deletion/insertion", () => {
  assert.throws(
    () =>
      walk(program(), {
        Program(node, ctx) {
          ctx.remove();
        },
      }),
    /root/,
  );
  assert.throws(
    () =>
      walk(program(), {
        Program(node, ctx) {
          ctx.replace(program());
        },
      }),
    /root/,
  );
  assert.throws(
    () =>
      walk(program("a"), {
        Identifier(node, ctx) {
          ctx.remove();
        },
      }),
    /array element/,
  );
  assert.throws(
    () =>
      walk(program("a"), {
        Identifier(node, ctx) {
          ctx.insertBefore(id("b"));
        },
      }),
    /array element/,
  );
  assert.throws(
    () =>
      walk(program(), {
        leave(node, ctx) {
          ctx.skip();
        },
      }),
    /enter/,
  );
});

test("invalid hooks, nodes, async callbacks, and stale containers fail explicitly", async () => {
  assert.throws(() => walk(program(), { Typo() {} }), /Unknown AST hook/);
  assert.throws(
    () => walk(program(), { Program: { exit() {} } }),
    /Invalid hook/,
  );
  assert.throws(() => walk({ type: "Unknown" }, {}), /Unsupported AST/);
  assert.throws(
    () =>
      walk(program(), {
        Program() {
          return false;
        },
      }),
    /return undefined/,
  );
  let saved;
  assert.throws(
    () =>
      walk(program(), {
        async Program(node, ctx) {
          saved = ctx;
          await 0;
        },
      }),
    /synchronous/,
  );
  await Promise.resolve();
  assert.throws(() => saved.node, /only valid/);
  assert.throws(
    () =>
      walk(program("a"), {
        Identifier(node, ctx) {
          ctx.parent.expression = id("changed");
          ctx.replace(id("bad"));
        },
      }),
    /container/,
  );
});

test("optional hooks and array holes are accepted, detached lists are rejected", () => {
  const ast = { type: "ArrayExpression", elements: [null, id("a")] };
  const seen = [];
  walk(ast, {
    enter: undefined,
    Identifier: {
      enter: undefined,
      leave(node) {
        seen.push(node.name);
      },
    },
  });
  assert.deepEqual(seen, ["a"]);
  assert.throws(
    () =>
      walk(program("a"), {
        ExpressionStatement(node, ctx) {
          ctx.parent.body = [];
          ctx.remove();
        },
      }),
    /container/,
  );
});

test("replace plus skip preserves leave, and removed contexts reject further edits", () => {
  const ast = program("a");
  const seen = [];
  walk(ast, {
    Identifier(node, ctx) {
      ctx.replace({
        type: "CallExpression",
        callee: id("unvisited"),
        arguments: [],
      });
      ctx.skip();
    },
    CallExpression: {
      leave(node) {
        seen.push(node.callee.name);
      },
    },
  });
  assert.deepEqual(seen, ["unvisited"]);
  walk(ast, {
    ExpressionStatement(node, ctx) {
      ctx.remove();
      assert.equal(ctx.index, null);
      assert.throws(() => ctx.remove(), /removed/);
      assert.throws(() => ctx.replace(statement("b")), /removed/);
    },
  });
  assert.equal(ast.body.length, 0);
});

test("nested edits restore parent positions for leave and later siblings", () => {
  const first = { type: "ArrayExpression", elements: [id("edit"), id("tail")] };
  const second = { type: "ArrayExpression", elements: [id("keep")] };
  const ast = { type: "ArrayExpression", elements: [first, second] };
  const seen = [];
  walk(ast, {
    Identifier(node, ctx) {
      if (node.name === "edit") {
        ctx.insertBefore(id("before"));
        ctx.insertAfter(id("after"));
        ctx.remove();
      } else {
        assert.deepEqual(ctx.ancestors(), [ast, ctx.parent]);
        seen.push(node.name);
      }
    },
    ArrayExpression: {
      leave(node, ctx) {
        assert.equal(ctx.node, node);
        if (node === ast) {
          assert.equal(ctx.parent, null);
          assert.equal(ctx.index, null);
        } else {
          assert.equal(ctx.parent, ast);
          assert.equal(ctx.key, "elements");
          assert.equal(ctx.parent.elements[ctx.index], node);
        }
      },
    },
  });
  assert.deepEqual(seen, ["after", "tail", "keep"]);
  assert.deepEqual(first.elements.map((node) => node.name), [
    "before", "after", "tail",
  ]);
  assert.deepEqual(second.elements.map((node) => node.name), ["keep"]);
});


test("generated child accessors stop before later scalar fields", () => {
  const ast = { type: "BinaryExpression", operator: "+", left: id("left"), right: id("right") };
  const seen = [];
  walk(ast, {
    Identifier(node, ctx) {
      seen.push(node.name);
      assert.equal(ctx.parent, ast);
      assert.equal(ctx.key, "left");
      ctx.stop();
    },
    leave() { assert.fail("stop must suppress pending leave hooks"); },
  });
  assert.deepEqual(seen, ["left"]);
  ast.left = { type: "Unknown" };
  assert.throws(() => walk(ast, {}), /Unsupported AST/);
  ast.left = Object.assign(() => {}, { type: "Identifier" });
  assert.throws(() => walk(ast, {}), /Unsupported AST/);
});
