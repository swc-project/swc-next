import { expect, test } from "@rstest/core";
import { NodeKind, parseSync, walk } from "@swc-next/core/ffi2";

function identifier(ast, name) {
  const node = ast.new_node(NodeKind.IdentifierReference);
  node.name = name;
  return node;
}

function statement(ast, name) {
  const node = ast.new_node(NodeKind.ExpressionStatement);
  node.expression = identifier(ast, name);
  return node;
}

function statementNames(program) {
  return program.body.slice().map((node) => node.expression.name);
}

test("visits schema children with state and parent context", () => {
  const ast = parseSync("first; second;");
  const program = ast.program;
  const state = { names: [] };

  expect(walk(program, {
    IdentifierReference(ctx) {
      ctx.state.names.push(ctx.node.name);
      expect(ctx.key).toBe("expression");
      expect(ctx.index).toBeNull();
      expect(ctx.parent.isSame(ast.node(ctx.parentId))).toBe(true);
      expect(ctx.ancestors().map((ancestor) => ancestor.type))
        .toEqual(["Program", "ExpressionStatement"]);
      expect(ctx.nodeId).toBe(ctx.node.nodeId);
      expect(ctx.kind).toBe(NodeKind.IdentifierReference);
      expect(ctx.type).toBe("IdentifierReference");
      expect(ctx.ast).toBe(ast);
    },
  }, state)).toBe(program);
  expect(state.names).toEqual(["first", "second"]);
});

test("orders global and native kind hooks", () => {
  const ast = parseSync("(function f() {})");
  const seen = [];
  walk(ast.program, {
    enter(ctx) { if (ctx.type === "Function") seen.push("global enter"); },
    Function: {
      enter() { seen.push("typed enter"); },
      leave() { seen.push("typed leave"); },
    },
    leave(ctx) { if (ctx.type === "Function") seen.push("global leave"); },
  });
  expect(seen).toEqual([
    "global enter", "typed enter", "typed leave", "global leave",
  ]);
});

test("replacement visits new children and selects leave hooks for the new type", () => {
  const ast = parseSync("before;");
  const program = ast.program;
  const seen = [];
  walk(program, {
    IdentifierReference(ctx) {
      seen.push(ctx.node.name);
      if (ctx.node.name === "before") {
        const replacement = ast.new_node(NodeKind.UnaryExpression);
        replacement.operator = "LogicalNot";
        replacement.argument = identifier(ast, "after");
        ctx.replace(replacement);
      }
    },
    UnaryExpression: {
      enter() { seen.push("unexpected re-entry"); },
      leave() { seen.push("unary leave"); },
    },
  });
  expect(seen).toEqual(["before", "after", "unary leave"]);

  seen.length = 0;
  walk(program, {
    IdentifierReference(ctx) { seen.push(ctx.node.name); },
    UnaryExpression: { leave(ctx) { ctx.replace(identifier(ast, "final")); } },
  });
  expect(seen).toEqual(["after"]);
  expect(program.body.get(0).expression.type).toBe("IdentifierReference");
  expect(program.body.get(0).expression.name).toBe("final");
});

test("skip retains leave while remove skips children and leave", () => {
  const ast = parseSync("skip; remove; keep;");
  const program = ast.program;
  const seen = [];
  walk(program, {
    ExpressionStatement: {
      enter(ctx) {
        if (ctx.node.expression.name === "skip") ctx.skip();
        if (ctx.node.expression.name === "remove") ctx.remove();
      },
      leave(ctx) { seen.push(`leave ${ctx.node.expression.name}`); },
    },
    IdentifierReference(ctx) { seen.push(ctx.node.name); },
  });
  expect(seen).toEqual(["leave skip", "keep", "leave keep"]);
  expect(statementNames(program)).toEqual(["skip", "keep"]);
});

test("insertions preserve order and visit only nodes inserted after the current node", () => {
  const ast = parseSync("current; tail;");
  const program = ast.program;
  const seen = [];
  walk(program, {
    ExpressionStatement(ctx) {
      if (ctx.node.expression.name !== "current") return;
      ctx.insertBefore([statement(ast, "before1"), statement(ast, "before2")]);
      expect(ctx.index).toBe(2);
      ctx.insertAfter(statement(ast, "after1"));
      ctx.insertAfter(statement(ast, "after2"));
    },
    IdentifierReference(ctx) { seen.push(ctx.node.name); },
  });
  expect(statementNames(program))
    .toEqual(["before1", "before2", "current", "after1", "after2", "tail"]);
  expect(seen).toEqual(["current", "after1", "after2", "tail"]);
});

test("context replace accepts kind descriptors", () => {
  const ast = parseSync("original;");
  const program = ast.program;
  walk(program, {
    ExpressionStatement(ctx) {
      if (ctx.node.expression.name === "original") {
        ctx.replace({
          kind: NodeKind.ExpressionStatement,
          expression: { kind: NodeKind.IdentifierReference, name: "nested" },
        });
      }
    },
  });
  expect(statementNames(program)).toEqual(["nested"]);
});

test("context insertions accept kind descriptors", () => {
  const ast = parseSync("current;");
  const program = ast.program;
  walk(program, {
    ExpressionStatement(ctx) {
      if (ctx.node.expression.name !== "current") return;
      ctx.insertBefore([{ kind: NodeKind.EmptyStatement }, { kind: NodeKind.EmptyStatement }]);
      ctx.insertAfter({ kind: NodeKind.DebuggerStatement });
    },
  });
  expect(program.body.toArray().map((node) => node.type))
    .toEqual(["EmptyStatement", "EmptyStatement", "ExpressionStatement", "DebuggerStatement"]);
});

test("context edits reject plain ESTree objects", () => {
  const ast = parseSync("original;");
  expect(() => walk(ast.program, {
    IdentifierReference(ctx) { ctx.replace({ type: "IdentifierReference", name: "bad" }); },
  })).toThrow(/Expected a node view or a node descriptor/);
});

test("stop retains edits and suppresses siblings and pending leave hooks", () => {
  const ast = parseSync("first; second;");
  const program = ast.program;
  const leaves = [];
  walk(program, {
    IdentifierReference(ctx) {
      ctx.node.name = "changed";
      ctx.stop();
    },
    leave(ctx) { leaves.push(ctx.type); },
  });
  expect(statementNames(program)).toEqual(["changed", "second"]);
  expect(leaves).toEqual([]);
});

test("contexts are reused, nested walks are isolated and saved contexts expire", () => {
  const ast = parseSync("one; two;");
  const program = ast.program;
  const contexts = [];
  const ancestors = [];
  walk(program, {
    IdentifierReference(ctx) {
      const node = ctx.node;
      contexts.push(ctx);
      ancestors.push(ctx.ancestors().map((ancestor) => ancestor.type));
      walk(parseSync("nested;").program, {
        IdentifierReference(nested) {
          expect(nested).not.toBe(ctx);
          expect(ctx.node).toBe(node);
        },
      });
      expect(ctx.node).toBe(node);
    },
  });
  expect(contexts[0]).toBe(contexts[1]);
  expect(ancestors).toEqual([
    ["Program", "ExpressionStatement"],
    ["Program", "ExpressionStatement"],
  ]);
  expect(() => contexts[0].parent).toThrow(/only valid/);
  expect(() => contexts[0].replace(identifier(ast, "bad"))).toThrow(/only valid/);

  let failedContext;
  expect(() => walk(program, {
    Program(ctx) {
      failedContext = ctx;
      throw new Error("hook failure");
    },
  })).toThrow("hook failure");
  expect(() => failedContext.node).toThrow(/only valid/);
});

test("rejects invalid callbacks, roots and structural edits", () => {
  expect(() => walk(parseSync("").program, { Typo() {} })).toThrow(/Unknown AST hook/);
  expect(() => walk({ type: "Program" }, {})).toThrow(/NodeView/);
  expect(() => walk(parseSync("").program, { Program: async () => {} })).toThrow(/synchronous/);
  expect(() => walk(parseSync("").program, { Program: () => false })).toThrow(/return undefined/);
  expect(() => walk(parseSync("").program, { Program(ctx) { ctx.remove(); } })).toThrow(/root/);
  expect(() => walk(parseSync("a;").program, { IdentifierReference(ctx) { ctx.remove(); } }))
    .toThrow(/array element/);
  expect(() => walk(parseSync("").program, { leave(ctx) { ctx.skip(); } })).toThrow(/enter/);

  const ast = parseSync("a;");
  expect(() => walk(ast.program, {
    IdentifierReference(ctx) {
      ctx.parent.expression = identifier(ast, "changed");
      ctx.replace(identifier(ast, "bad"));
    },
  })).toThrow(/container/);
});
test("parameter patterns keep their native owner as parent", () => {
  const ast = parseSync("function f([a, b]) {}", { lang: "ts" });
  const pattern = ast.program.body.get(0).params.items.get(0).pattern;
  const seen = [];
  walk(ast.program, {
    BindingIdentifier(ctx) {
      if (ctx.node.name === "a") seen.push([ctx.parentId, ctx.parent.type, ctx.key, ctx.index]);
    },
  });
  expect(seen).toEqual([[pattern.nodeId, "ArrayPattern", "elements", 0]]);
});
