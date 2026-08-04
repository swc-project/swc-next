import assert from "node:assert/strict";
import test from "node:test";

import { parseSync as rawParseSync } from "../src-js/binding.js";
import { parseSync } from "../src-js/index.js";

const TS_OPTIONS = {
  comments: "none",
  lang: "ts",
  preserveParens: false,
  sourceType: "module",
};

function collectNodes(root) {
  const nodes = [];
  const pending = [root];
  while (pending.length > 0) {
    const value = pending.pop();
    if (value === null || typeof value !== "object") continue;
    if (value.type) nodes.push(value);
    pending.push(...Object.values(value));
  }
  return nodes;
}

test("decodes representative TypeScript ESTree structures", () => {
  const source = [
    '/** @import { Box } from "./box" */',
    "interface Factory {",
    "  readonly [key: string]: Box<string>;",
    "  new <T>(value: T): Box<T>;",
    "  <T>(value: T): Box<T>;",
    "}",
    "type EventName<T extends string> = `on${Capitalize<T>}`;",
    "declare function invoke<T>(value: T): T;",
    'invoke<{ value: string }>({ value: "ok" });',
    "export = invoke;",
  ].join("\n");
  const { program } = parseSync(source, TS_OPTIONS);
  const nodes = collectNodes(program);
  const types = new Set(nodes.map((node) => node.type));

  for (const type of [
    "TSIndexSignature",
    "TSConstructSignatureDeclaration",
    "TSCallSignatureDeclaration",
    "TSTemplateLiteralType",
    "TSTypeReference",
    "TSExportAssignment",
  ]) {
    assert.ok(types.has(type), `missing ${type}`);
  }

  const genericCall = nodes.find(
    (node) =>
      node.type === "CallExpression" &&
      node.typeArguments?.params[0]?.type === "TSTypeLiteral",
  );
  assert.ok(genericCall, "missing generic call with an object type argument");
  assert.equal(program.start, source.indexOf("interface"));
});

test("orders TypeScript parameter-property fields like the ESTree oracle", () => {
  const source =
    "export class Box { constructor(public readonly value: string) {} }";
  const { program } = parseSync(source, TS_OPTIONS);
  const parameterProperty = collectNodes(program).find(
    (node) => node.type === "TSParameterProperty",
  );

  assert.ok(parameterProperty);
  assert.deepEqual(Object.keys(parameterProperty), [
    "type",
    "accessibility",
    "decorators",
    "override",
    "parameter",
    "readonly",
    "static",
    "start",
    "end",
  ]);
});

test("serializes representative TypeScript ASTs to stable bytes", () => {
  const source = [
    "interface Callable { [key: string]: unknown; new (): Callable; (): void; }",
    "type Ref = Promise<`value-${string}`>;",
    "export = Ref;",
  ].join("\n");

  const first = rawParseSync(source, TS_OPTIONS);
  const second = rawParseSync(source, TS_OPTIONS);
  assert.deepEqual(first, second);
});
