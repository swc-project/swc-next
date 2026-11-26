import { expect, test } from "@rstest/core";
import { parseSync, generateSync, walk, NodeKind } from "@swc-next/core/ffi2";

test("edits round-trip through the core encoder and parser", () => {
  const ast = parseSync("#!/usr/bin/env node\ndebugger; original(); tail();");
  const program = ast.program;
  function call(name, argument) {
    const callee = ast.new_node(NodeKind.IdentifierReference);
    callee.name = name;
    const node = ast.new_node(NodeKind.CallExpression);
    node.callee = callee;
    if (argument) { node.arguments.push(argument); }
    return node;
  }
  function statement(name) {
    const node = ast.new_node(NodeKind.ExpressionStatement);
    node.expression = call(name);
    return node;
  }
  const replacement = call("replacement", call("nested"));
  const before = statement("before");
  const after = statement("after");
  const seen = [];
  walk(program, {
    Hashbang(ctx) {
      seen.push("hashbang");
      ctx.node.value = "/usr/bin/env bun";
    },
    DebuggerStatement(ctx) { ctx.remove(); },
    ExpressionStatement(ctx) {
      if (ctx.node.expression.callee.name === "original") {
        ctx.insertBefore(before);
        ctx.insertAfter(after);
      }
    },
    CallExpression(ctx) {
      seen.push(ctx.node.callee.name);
      if (ctx.node.callee.name === "original") { ctx.replace(replacement); }
    },
  });
  const { code } = generateSync(program);
  expect(seen).toEqual(["original", "nested", "after", "tail", "hashbang"]);
  expect(code).toBe("#!/usr/bin/env bun\nbefore();\nreplacement(nested());\nafter();\ntail();\n");
  expect(parseSync(code).diagnostics).toEqual([]);
});

function expectNativeCoverage(source, options) {
  const { program, diagnostics } = parseSync(source, options);
  expect(diagnostics).toEqual([]);
  const expected = [];
  function collect(node) {
    expected.push([node.type, node.start, node.end]);
    for (const value of Object.values(node)) {
      if (Array.isArray(value)) {
        for (const child of value) {
          if (typeof child?.type === "string") { collect(child); }
        }
      } else if (typeof value?.type === "string") {
        collect(value);
      }
    }
  }
  collect(program.ast.materialize(program));
  const actual = [];
  walk(program, { enter(ctx) { actual.push([ctx.type, ctx.node.start, ctx.node.end]); } });
  const sortVisits = (left, right) => JSON.stringify(left).localeCompare(JSON.stringify(right));
  expect(actual.sort(sortVisits)).toEqual(expected.sort(sortVisits));
  const identifiers = [];
  walk(program, { IdentifierReference(ctx) { identifiers.push([ctx.type, ctx.node.start, ctx.node.end]); } });
  expect(identifiers.sort(sortVisits)).toEqual(expected.filter(([type]) => type === "IdentifierReference"));
}

test("traverses native JavaScript, JSX and TypeScript children", () => {
  expectNativeCoverage(
    '#!/usr/bin/env node\nfunction f({ a = 1, ...rest }, ...args) { return `${a}:${args[0]}`; }',
  );
  expectNativeCoverage(
    "const x = <A<T> value={f()} {...props}><B />{g()}</A>;", { lang: "tsx" },
  );
  expectNativeCoverage(
    "function f<T>(this: C, x: T = value, ...rest: T[]): T { return x; }", { lang: "ts" },
  );
  expectNativeCoverage(
    "class C { constructor(public value: number) {} method(x: number): void {} }", { lang: "ts" },
  );
});
