import assert from "node:assert/strict";
import test from "node:test";
import { parse } from "acorn";
import { printSync as printCodegen } from "@swc-next/codegen";
import { decode } from "@swc-next/decoder";
import { encode } from "@swc-next/encoder";
import { parseSync, printSync } from "../src-js/index.js";

test("codegen accepts unmodified Acorn ASTs", () => {
  for (const source of [
    '"use strict"; function f({ x = 1 }, ...rest) { return [x, ...rest]; }',
    'import { x as y } from "pkg"; export const f = () => y?.value ?? 1;',
    'class C { #x = 1; method() { return this.#x; } }',
    'const values = [1n, /a+/gi, `hello ${name}`, null];',
  ]) {
    const program = parse(source, { ecmaVersion: "latest", sourceType: "module" });
    const expected = printSync(parseSync(source).program).code;
    assert.equal(printCodegen(program).code, expected);
    assert.equal(printSync(decode(encode(program), source).program).code, expected);
  }
});

test("body-less TypeScript functions keep their public ESTree shape", () => {
  const declaration = parseSync("declare function f(): void;", { lang: "ts" }).program.body[0];
  assert.equal(declaration.type, "TSDeclareFunction");
  assert.equal(declaration.body, null);
  assert.equal(declaration.expression, false);

  const method = parseSync("abstract class C { abstract f(): void; }", { lang: "ts" }).program.body[0].body.body[0];
  assert.equal(method.value.type, "TSEmptyBodyFunctionExpression");
  assert.equal(method.value.body, null);
});
