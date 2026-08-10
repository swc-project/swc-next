import assert from "node:assert/strict";
import test from "node:test";

import { parseSync } from "../src-js/index.js";

test("parseSync parses JavaScript", () => {
  const { program } = parseSync("const answer = 42;");

  assert.equal(program.type, "Program");
  assert.equal(program.body[0].type, "VariableDeclaration");
});
