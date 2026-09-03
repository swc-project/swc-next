import assert from "node:assert/strict";
import test from "node:test";

import { parseSync, printSync } from "../src-js/index.js";

test("core exposes parseSync and printSync", () => {
  const { program } = parseSync("const value = 1;");
  program.body[0].declarations[0].init.value = 2;

  assert.equal(printSync(program).code, "const value = 2;\n");
});
