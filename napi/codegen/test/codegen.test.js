import assert from "node:assert/strict";
import test from "node:test";

import { parseSync } from "../../parser/src-js/index.js";
import { encode } from "../src-js/encode.js";
import { printSync } from "../src-js/index.js";

test("encode writes spans in canonical little-endian order", () => {
  const buffer = encode({
    type: "Program",
    start: 0x01020304,
    end: 0x05060708,
    sourceType: "module",
    body: [],
  });

  assert.deepEqual(
    [...buffer.subarray(36, 44)],
    [0x04, 0x03, 0x02, 0x01, 0x08, 0x07, 0x06, 0x05],
  );
});

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

test("printSync prints modified JSX attribute strings", () => {
  const { program } = parseSync(
    'const element = <div title="old" />;',
    { lang: "tsx" },
  );
  const value = program.body[0].declarations[0].init.openingElement.attributes[0].value;

  value.value = "new";
  assert.equal(printSync(program).code, 'const element = <div title="new" />;\n');
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
