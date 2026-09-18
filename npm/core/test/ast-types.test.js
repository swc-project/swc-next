import assert from "node:assert/strict";
import test from "node:test";
import { parseSync } from "../src-js/index.js";

test("body-less TypeScript functions keep their public ESTree shape", () => {
  const declaration = parseSync("declare function f(): void;", { lang: "ts" }).program.body[0];
  assert.equal(declaration.type, "TSDeclareFunction");
  assert.equal(declaration.body, null);
  assert.equal(declaration.expression, false);

  const method = parseSync("abstract class C { abstract f(): void; }", { lang: "ts" }).program.body[0].body.body[0];
  assert.equal(method.value.type, "TSEmptyBodyFunctionExpression");
  assert.equal(method.value.body, null);
});
