import { expect, test } from "@rstest/core";
import { parseSync, printSync, walk } from "@swc-next/core";

const ast = (source) => parseSync(source).program;

test("edits round-trip through the core encoder and parser", () => {
  const replacement = ast("replacement(nested());").body[0].expression;
  const before = ast("before();").body[0];
  const after = ast("after();").body[0];
  const seen = [];
  const { program, meta } = parseSync("#!/usr/bin/env node\ndebugger; original(); tail();");
  walk(program, {
    Hashbang(node) {
      seen.push("hashbang");
      node.value = "/usr/bin/env bun";
    },
    DebuggerStatement(_, ctx) { ctx.remove(); },
    ExpressionStatement(node, ctx) {
      if (node.expression.callee.name === "original") {
        ctx.insertBefore(before);
        ctx.insertAfter(after);
      }
    },
    CallExpression(node, ctx) {
      seen.push(node.callee.name);
      if (node.callee.name === "original") ctx.replace(replacement);
    },
  });
  const { code } = printSync(program, {}, meta);
  expect(seen).toEqual(["hashbang", "original", "nested", "after", "tail"]);
  expect(code).toBe("#!/usr/bin/env bun\nbefore();\nreplacement(nested());\nafter();\ntail();\n");
  expect(parseSync(code).diagnostics).toEqual([]);
});

function expectProjectionCoverage(source, options) {
  const { program, diagnostics } = parseSync(source, options);
  expect(diagnostics).toEqual([]);
  const expected = new Map();
  function collect(node) {
    expected.set(node, (expected.get(node) ?? 0) + 1);
    for (const value of Object.values(node)) {
      if (Array.isArray(value)) {
        for (const child of value) if (typeof child?.type === "string") collect(child);
      } else if (typeof value?.type === "string") {
        collect(value);
      }
    }
  }
  collect(program);
  const actual = new Map();
  walk(program, { enter(node) { actual.set(node, (actual.get(node) ?? 0) + 1); } });
  expect(actual).toEqual(expected);
  const identifiers = [];
  walk(program, { Identifier(node) { identifiers.push(node); } });
  expect(new Set(identifiers)).toEqual(
    new Set([...expected.keys()].filter((node) => node.type === "Identifier")),
  );
}

test("traverses parser-projected JavaScript, JSX and TypeScript children", () => {
  expectProjectionCoverage(
    '#!/usr/bin/env node\nfunction f({ a = 1, ...rest }, ...args) { return `${a}:${args[0]}`; }',
  );
  expectProjectionCoverage(
    "const x = <A<T> value={f()} {...props}><B />{g()}</A>;", { lang: "tsx" },
  );
  expectProjectionCoverage(
    "function f<T>(this: C, x: T = value, ...rest: T[]): T { return x; }", { lang: "ts" },
  );
  expectProjectionCoverage(
    "class C { constructor(public value: number) {} method(x: number): void {} }", { lang: "ts" },
  );
});
