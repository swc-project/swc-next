import assert from "node:assert/strict";
import test from "node:test";
import { parseSync, printSync, transformSync, walk } from "../src-js/index.js";

const ast = (source) => parseSync(source).program;

function removeConsole() {
  return {
    name: "remove-console",
    visitor: {
      ExpressionStatement(node, ctx) {
        const call = node.expression;
        if (
          call.type === "CallExpression" &&
          call.callee.type === "MemberExpression" &&
          call.callee.object.type === "Identifier" &&
          call.callee.object.name === "console"
        )
          ctx.remove();
      },
    },
  };
}

test("transformSync parses, applies plugin factories' results, and prints", () => {
  const result = transformSync('console.log("hello"); doSomething();', {
    plugins: [removeConsole()],
  });
  assert.equal(result.code, "doSomething();\n");
  assert.equal(result.program.body.length, 1);
  assert.deepEqual(result.diagnostics, []);
  assert.equal(result.meta.lang, "js");
});

test("each plugin completes a full pass before the next sees the AST", () => {
  const first = { count: 0 };
  const seen = [];
  const result = transformSync("first(); second();", {
    plugins: [
      {
        state: first,
        visitor: {
          Identifier(node, ctx) {
            ctx.state.count++;
            node.name += "Changed";
          },
        },
      },
      {
        visitor: {
          Program(node) {
            assert.equal(node.body[1].expression.callee.name, "secondChanged");
          },
          Identifier(node) {
            seen.push(node.name);
          },
        },
      },
    ],
  });
  assert.equal(first.count, 2);
  assert.deepEqual(seen, ["firstChanged", "secondChanged"]);
  assert.equal(result.code, "firstChanged();\nsecondChanged();\n");
});

test("structural edits round-trip through the encoder and parser", () => {
  const replacement = ast("replacement(nested());").body[0].expression;
  const before = ast("before();").body[0];
  const after = ast("after();").body[0];
  const seen = [];
  const result = transformSync("debugger; original(); tail();", {
    plugins: [
      {
        visitor: {
          DebuggerStatement(node, ctx) {
            ctx.remove();
          },
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
        },
      },
    ],
  });
  assert.deepEqual(seen, ["original", "nested", "after", "tail"]);
  assert.equal(
    result.code,
    "before();\nreplacement(nested());\nafter();\ntail();\n",
  );
  assert.deepEqual(parseSync(result.code).diagnostics, []);
});

test("transform preserves parser metadata and passes codegen options", () => {
  const source = "const x: number = 1;";
  const parsed = parseSync(source, { lang: "ts" });
  const result = transformSync(source, {
    parser: { lang: "ts" },
    codegen: { minify: true },
  });
  assert.equal(result.meta.lang, "ts");
  assert.equal(
    result.code,
    printSync(parsed.program, { minify: true }, parsed.meta).code,
  );
  assert.deepEqual(parseSync(result.code, { lang: "ts" }).diagnostics, []);
});

test("parse errors prevent plugin execution and expose diagnostics", () => {
  let called = false;
  assert.throws(
    () =>
      transformSync("const = ;", {
        plugins: [
          {
            visitor: {
              Program() {
                called = true;
              },
            },
          },
        ],
      }),
    (error) =>
      error instanceof SyntaxError &&
      error.diagnostics.some((d) => d.severity === "error"),
  );
  assert.equal(called, false);
  assert.throws(
    () => transformSync("ok();", { plugins: [removeConsole] }),
    /plugin factories/,
  );
});

// On parser-produced trees, compare the generated traversal with structural
// discovery to catch projection fields omitted from the visitor-key generator.
function assertProjectionCoverage(source, options) {
  const parsed = parseSync(source, options);
  assert.deepEqual(parsed.diagnostics, []);
  const expected = new Map();
  function collect(node) {
    expected.set(node, (expected.get(node) ?? 0) + 1);
    for (const value of Object.values(node)) {
      if (Array.isArray(value)) {
        for (const child of value)
          if (typeof child?.type === "string") collect(child);
      } else if (typeof value?.type === "string") collect(value);
    }
  }
  collect(parsed.program);
  const actual = new Map();
  walk(parsed.program, {
    enter(node) {
      actual.set(node, (actual.get(node) ?? 0) + 1);
    },
  });
  assert.deepEqual(actual, expected);
}

test("visitor keys cover projected JS, JSX, and TypeScript children", () => {
  assertProjectionCoverage(
    '"use strict"; function f({ a = 1, ...rest }, ...args) { return `${a}:${args[0]}`; }',
  );
  assertProjectionCoverage(
    'import data from "x" with { type: "json" }; export { data }; export * as ns from "x";',
  );
  assertProjectionCoverage(
    "class C extends Base { #x = 1; get value() { return this.#x; } static { run(); } }",
  );
  assertProjectionCoverage(
    "const x = <A<T> value={f()} {...props}><B />{g()}</A>;",
    { lang: "tsx" },
  );
  assertProjectionCoverage(
    "function f<T>(this: C, x: T = value, ...rest: T[]): T { return x; }",
    { lang: "ts" },
  );
  assertProjectionCoverage(
    "class C { constructor(public value: number) {} method(x: number): void {} }",
    { lang: "ts" },
  );
  assertProjectionCoverage(
    "abstract class C<T> extends Base<T> implements I<T> { abstract method(x: T): T; abstract value: T; accessor other: T; }",
    { lang: "ts" },
  );
  assertProjectionCoverage(
    "type F = <T>(this: C, x: T) => T; type Ctor = new <T>(x: T) => C<T>;",
    { lang: "ts" },
  );
  assertProjectionCoverage(
    "interface I<T> extends Base<T> { (x: T): T; new(x: T): I<T>; method(x: T): T; [key: string]: T; }",
    { lang: "ts" },
  );
  assertProjectionCoverage(
    "type M<T> = { [K in keyof T as `get${K}`]?: T[K] }; type Tup = [name: string, count?: number, ...rest: boolean[]];",
    { lang: "ts" },
  );
  assertProjectionCoverage(
    "declare function f<T>(x: T): T; declare namespace N { const x: number; } declare global { interface G {} } export {};",
    { lang: "ts" },
  );
  assertProjectionCoverage(
    'enum E { A = f(), B }; import X = require("x"); type T = typeof import("x").foo;',
    { lang: "ts" },
  );
  assertProjectionCoverage(
    "const { x }: T = input; try {} catch (e: unknown) {} const f = (x: number = 1): number => x;",
    { lang: "ts" },
  );
  assertProjectionCoverage(
    "@dec class C { @dec value = 1; @dec method(@dec x: T) {} }",
    { lang: "ts" },
  );
});
