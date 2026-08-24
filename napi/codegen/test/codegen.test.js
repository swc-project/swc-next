import assert from "node:assert/strict";
import test from "node:test";

import { parseSync } from "../../parser/src-js/index.js";
import { printSync } from "../src-js/index.js";

test("printSync prints modified parser-produced ESTree", () => {
  const { program } = parseSync("const answer = 1;");
  program.body[0].declarations[0].id.name = "changed";
  program.body[0].declarations[0].init.value = 42;

  assert.equal(printSync(program).code, "const changed = 42;\n");
});

test("printSync forwards codegen options", () => {
  const { program } = parseSync("const emoji = '😀';");

  assert.equal(
    printSync(program, { asciiOnly: true, minify: true }).code,
    'const emoji="\\u{1F600}";',
  );
});

test("printSync rejects unsupported ESTree nodes", () => {
  assert.throws(
    () =>
      printSync({
        type: "Program",
        sourceType: "script",
        body: [{ type: "Unsupported" }],
      }),
    /Unsupported ESTree node type Unsupported/,
  );
});
