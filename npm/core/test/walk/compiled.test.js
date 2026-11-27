import { expect, test } from "@rstest/core";
import { createWalker, merge, walk } from "@swc-next/walker";

const id = (name) => ({ type: "Identifier", name });
const program = () => ({
  type: "FunctionDeclaration",
  id: id("f"),
  params: [],
  body: { type: "BlockStatement", body: [{ type: "DebuggerStatement" }] },
});

test("compiled walkers snapshot hooks and isolate invocation state", () => {
  const pair = {
    enter(node, ctx) {
      ctx.state.push("typed");
    },
  };
  const hooks = {
    Function(node, ctx) {
      ctx.state.push("alias");
    },
    FunctionDeclaration: pair,
    DebuggerStatement(node, ctx) {
      ctx.state.push(node.type);
      ctx.remove();
    },
  };
  const walker = createWalker(hooks);
  hooks.Function = pair.enter = () => {
    throw new Error("hooks were read again");
  };
  const first = program();
  const second = program();
  const firstState = ["first"];
  const secondState = ["second"];
  expect(walker.walk(first, firstState)).toBe(first);
  walker.walk(second, secondState);
  expect(first.body.body).toEqual([]);
  expect(second.body.body).toEqual([]);
  expect(firstState).toEqual(["first", "alias", "typed", "DebuggerStatement"]);
  expect(secondState).toEqual(["second", "alias", "typed", "DebuggerStatement"]);
});

test("compiled walkers support reentrancy and recover after hook failures", () => {
  const outerState = [];
  const innerState = [];
  const walker = createWalker({
    Identifier(node, ctx) {
      if (node.name === "throw") throw new Error("hook failure");
      ctx.state.push(node.name);
      if (node.name === "outer") {
        walker.walk(id("inner"), innerState);
        expect(ctx.node).toBe(node);
        expect(ctx.state).toBe(outerState);
        ctx.state.push("resumed");
      }
    },
  });
  walker.walk(id("outer"), outerState);
  expect(outerState).toEqual(["outer", "resumed"]);
  expect(innerState).toEqual(["inner"]);
  expect(() => walker.walk(id("throw"), [])).toThrow("hook failure");
  const afterState = [];
  walker.walk(id("after"), afterState);
  expect(afterState).toEqual(["after"]);
});

test("merged visitors preserve per-visitor hook order in one traversal", () => {
  const ast = program();
  const events = [];
  const visitor = (name) => ({
    enter(node) { if (node === ast) events.push(`${name}:global enter`); },
    Function: {
      enter() { events.push(`${name}:alias enter`); },
      leave() { events.push(`${name}:alias leave`); },
    },
    FunctionDeclaration: {
      enter() { events.push(`${name}:typed enter`); },
      leave() { events.push(`${name}:typed leave`); },
    },
    Identifier() { events.push(`${name}:Identifier`); },
    leave(node) { if (node === ast) events.push(`${name}:global leave`); },
  });
  const visitors = [visitor("A"), visitor("B")];
  const merged = merge(visitors);
  const walker = createWalker(merged);
  visitors[0].Function = () => { throw new Error("hook snapshot changed"); };
  visitors.push({ enter() { throw new Error("visitor snapshot changed"); } });
  walker.walk(ast);
  expect(events).toEqual([
    "A:global enter", "A:alias enter", "A:typed enter",
    "B:global enter", "B:alias enter", "B:typed enter",
    "A:Identifier", "B:Identifier",
    "A:typed leave", "A:alias leave", "A:global leave",
    "B:typed leave", "B:alias leave", "B:global leave",
  ]);
  const directEvents = [];
  walk(id("direct"), merge([
    { Identifier(node) { directEvents.push(`A:${node.name}`); } },
    { Identifier(node) { directEvents.push(`B:${node.name}`); } },
  ]));
  expect(directEvents).toEqual(["A:direct", "B:direct"]);
  expect(Object.isFrozen(merged)).toBe(true);
  expect(Object.isFrozen(merged.FunctionDeclaration.enter)).toBe(true);
  expect(createWalker(merge([])).walk(ast)).toBe(ast);
  expect(() => merge([{}, { UnknownNode() {} }])).toThrow(/Unknown/);
  expect(() => merge({})).toThrow(/array/);
});

test("merge accepts merged visitors and snapshots callback arrays", () => {
  const events = [];
  const callbacks = [() => { events.push("A"); }, () => { events.push("B"); }];
  const merged = merge([merge([{ enter: callbacks }]), { Identifier() { events.push("C"); } }]);
  callbacks[0] = () => { throw new Error("callback snapshot changed"); };
  walk(id("value"), merged);
  expect(events).toEqual(["A", "B", "C"]);
  expect(() => merge([{ Identifier: { enter: [1] } }])).toThrow(/Invalid hook/);
});

test("compiled and merged walkers reject sparse callbacks before traversal", () => {
  const callbacks = [() => {}, ,];
  expect(() => createWalker({ enter: callbacks })).toThrow(TypeError);
  expect(() => createWalker({ Identifier: { enter: callbacks } })).toThrow(TypeError);
  expect(() => merge([{ Identifier: { leave: callbacks } }])).toThrow(TypeError);
});

test("merged visitors share skip, replacement, removal, and stop controls", () => {
  const statement = (name) => ({ type: "ExpressionStatement", expression: id(name) });
  const ast = { type: "Program", body: [statement("skip"), statement("remove"), statement("replace"), statement("stop"), statement("tail")] };
  const events = [];
  createWalker(merge([
    {
      ExpressionStatement: {
        enter(node, ctx) {
          events.push(`A:${node.expression.name}`);
          if (node.expression.name === "skip") ctx.skip();
          if (node.expression.name === "remove") ctx.remove();
          if (node.expression.name === "replace") ctx.replace(statement("new"));
          if (node.expression.name === "stop") ctx.stop();
        },
        leave(node) { events.push(`A:leave ${node.expression.name}`); },
      },
      Identifier(node) { events.push(`A:child ${node.name}`); },
    },
    {
      ExpressionStatement: {
        enter(node) { events.push(`B:${node.expression.name}`); },
        leave(node) { events.push(`B:leave ${node.expression.name}`); },
      },
      Identifier(node) { events.push(`B:child ${node.name}`); },
    },
  ])).walk(ast);
  expect(events).toEqual([
    "A:skip", "A:leave skip", "B:leave skip",
    "A:remove", "A:replace", "A:child new", "B:child new",
    "A:leave new", "B:leave new", "A:stop",
  ]);
  expect(ast.body.map(node => node.expression.name)).toEqual(["skip", "new", "stop", "tail"]);
});

test("list edits dispatch inserted nodes and stop subsequent traversal", () => {
  const first = { type: "VariableDeclarator", id: id("first"), init: null };
  const last = { type: "VariableDeclarator", id: id("last"), init: null };
  const inserted = {
    type: "ExpressionStatement",
    expression: { type: "Literal", value: "nested" },
  };
  const ast = { type: "VariableDeclaration", declarations: [first, null, last] };
  const seen = [];
  walk(ast, {
    VariableDeclarator(node, ctx) {
      seen.push(node.id.name);
      if (node === first) {
        ctx.insertBefore(id("before"));
        ctx.insertAfter(inserted);
        ctx.remove();
      }
    },
    ExpressionStatement() {
      seen.push("inserted");
    },
    Literal(node) {
      seen.push(node.value);
    },
  });
  expect(seen).toEqual(["first", "inserted", "nested", "last"]);
  expect(ast.declarations).toEqual([id("before"), inserted, null, last]);

  const stopped = [];
  walk(ast, {
    enter(node) {
      stopped.push(node.type);
    },
    ExpressionStatement(node, ctx) {
      ctx.stop();
    },
    VariableDeclaration: {
      leave() {
        stopped.push("leave");
      },
    },
  });
  expect(stopped).toEqual(["VariableDeclaration", "Identifier", "ExpressionStatement"]);
});
