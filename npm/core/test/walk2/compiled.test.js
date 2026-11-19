import { expect, test } from "@rstest/core";
import { NodeKind, parseSync } from "@swc-next/core/ffi2";
import { createWalker, walk } from "@swc-next/walker2";

function identifier(ast, name) {
  const node = ast.new_node(NodeKind.IdentifierReference);
  node.name = name;
  return node;
}

test("compiled walkers snapshot hooks and isolate invocation state", () => {
  const pair = {
    enter(ctx) {
      ctx.state.push("typed");
    },
  };
  const hooks = {
    Function(ctx) {
      ctx.state.push("alias");
    },
    FunctionDeclaration: pair,
    DebuggerStatement(ctx) {
      ctx.state.push(ctx.type);
      ctx.remove();
    },
  };
  const walker = createWalker(hooks);
  hooks.Function = pair.enter = () => {
    throw new Error("hooks were read again");
  };
  const first = parseSync("function f() { debugger; }").program;
  const second = parseSync("function f() { debugger; }").program;
  const firstState = ["first"];
  const secondState = ["second"];
  expect(walker.walk(first, firstState)).toBe(first);
  walker.walk(second, secondState);
  expect(first.body.get(0).body.body.length).toBe(0);
  expect(second.body.get(0).body.body.length).toBe(0);
  expect(firstState).toEqual(["first", "alias", "typed", "DebuggerStatement"]);
  expect(secondState).toEqual(["second", "alias", "typed", "DebuggerStatement"]);
});

test("compiled walkers support reentrancy and recover after hook failures", () => {
  const outerState = [];
  const innerState = [];
  const walker = createWalker({
    Identifier(ctx) {
      const node = ctx.node;
      if (node.name === "throwError") throw new Error("hook failure");
      ctx.state.push(node.name);
      if (node.name === "outer") {
        walker.walk(parseSync("inner;").program, innerState);
        expect(ctx.node).toBe(node);
        expect(ctx.state).toBe(outerState);
        ctx.state.push("resumed");
      }
    },
  });
  walker.walk(parseSync("outer;").program, outerState);
  expect(outerState).toEqual(["outer", "resumed"]);
  expect(innerState).toEqual(["inner"]);
  expect(() => walker.walk(parseSync("throwError;").program, [])).toThrow("hook failure");
  const afterState = [];
  walker.walk(parseSync("after;").program, afterState);
  expect(afterState).toEqual(["after"]);
});

test("list edits dispatch inserted nodes and stop subsequent traversal", () => {
  const ast = parseSync("var first, last;");
  const program = ast.program;
  const declarations = program.body.get(0).declarations;
  function declarator(name) {
    const node = ast.new_node(NodeKind.VariableDeclarator);
    node.id = identifier(ast, name);
    return node;
  }
  const seen = [];
  walk(program, {
    VariableDeclarator(ctx) {
      seen.push(ctx.node.id.name);
      if (ctx.node.id.name === "first") {
        ctx.insertBefore(declarator("before"));
        ctx.insertAfter(declarator("inserted"));
        ctx.remove();
      }
    },
  });
  expect(seen).toEqual(["first", "inserted", "last"]);
  expect(declarations.slice().map((node) => node.id.name))
    .toEqual(["before", "inserted", "last"]);

  const stopped = [];
  walk(program, {
    enter(ctx) {
      stopped.push(ctx.type);
    },
    VariableDeclarator(ctx) {
      if (ctx.node.id.name === "before") ctx.stop();
    },
    VariableDeclaration: {
      leave() {
        stopped.push("leave");
      },
    },
  });
  expect(stopped).toEqual(["Program", "VariableDeclaration", "VariableDeclarator"]);
});
