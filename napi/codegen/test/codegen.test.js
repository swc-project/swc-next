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

test("printSync rejects lone surrogates before they reach native AstUtf8", () => {
  const { program } = parseSync("const value = 1;");
  program.body[0].declarations[0].id.name = "\uD800";

  // Letting WTF-8 bytes reach native AstUtf8 would violate the safety
  // requirement of Rust's `from_utf8_unchecked` and may cause undefined behavior.
  assert.throws(
    () => printSync(program),
    /AST UTF-8 field contains an unpaired surrogate/,
  );
});

test("printSync preserves lone surrogates in WTF-8 fields", () => {
  const { program } = parseSync('const value = "ok";');
  program.body[0].declarations[0].init.value = "\uD800";
  program.body[0].declarations[0].init.raw = null;

  assert.equal(printSync(program).code, 'const value = "\\uD800";\n');
});

test("printSync accepts surrogate pairs in UTF-8 fields", () => {
  const { program } = parseSync("const value = 1;");
  program.body[0].declarations[0].id.name = "\u{10400}";

  assert.equal(printSync(program).code, "const 𐐀 = 1;\n");
});

test("printSync preserves modified negative zero", () => {
  const { program } = parseSync("const value = 1;");
  program.body[0].declarations[0].init.value = -0;

  assert.equal(printSync(program).code, "const value = -0;\n");
});

test("printSync handles modified negative numbers contextually", () => {
  const exponent = parseSync("1 ** 2;").program;
  exponent.body[0].expression.left.value = -1;
  assert.equal(printSync(exponent).code, "(-1) ** 2;\n");

  const property = parseSync("({ 1: 0 });").program;
  property.body[0].expression.expression.properties[0].key.value = -1;
  assert.equal(printSync(property).code, '({ "-1": 0 });\n');
});

test("printSync preserves reordered directive statements", () => {
  const { program } = parseSync('"use strict"; work();');
  program.body.reverse();

  assert.equal(printSync(program).code, 'work();\n"use strict";\n');
});

test("printSync invalidates stale directive raw text", () => {
  const { program } = parseSync('"use client";');
  program.body[0].expression.value = "use server";

  assert.equal(printSync(program).code, '"use server";\n');

  const updated = parseSync('"use client";').program;
  updated.body[0].expression.value = "use server";
  updated.body[0].directive = "use server";

  assert.equal(printSync(updated).code, '"use server";\n');

  const escaped = parseSync('"use\\x20client";').program;
  assert.equal(printSync(escaped).code, '"use\\x20client";\n');

  escaped.body[0].expression.value = "use server";
  assert.equal(printSync(escaped).code, '"use server";\n');
});

test("printSync preserves code point escapes with leading zeros", () => {
  const { program } = parseSync('"use\\u{0000020}strict";', {
    sourceType: "script",
  });

  assert.equal(printSync(program).code, '"use\\u{0000020}strict";\n');
});

test("printSync preserves an unchanged BigInt radix", () => {
  const { program } = parseSync("0x1_2_3n;");

  assert.equal(printSync(program).code, "0x123n;\n");
});

test("printSync preserves unchanged legacy octal raw text", () => {
  const { program } = parseSync("077;", { sourceType: "script" });

  assert.equal(printSync(program).code, "077;\n");

  program.body[0].expression.value = 64;
  assert.equal(printSync(program).code, "64;\n");
});

test("printSync handles modified negative BigInts contextually", () => {
  const exponent = parseSync("1n ** 2n;").program;
  exponent.body[0].expression.left.value = -1n;
  assert.equal(printSync(exponent).code, "(-1n) ** 2n;\n");

  const property = parseSync("({ 1n: 0 });").program;
  property.body[0].expression.expression.properties[0].key.value = -1n;
  assert.equal(printSync(property).code, '({ "-1": 0 });\n');
});

test("printSync uses modified RegExp metadata", () => {
  const { program } = parseSync("/old/;");
  program.body[0].expression.regex.pattern = "new";
  program.body[0].expression.regex.flags = "g";

  assert.equal(printSync(program).code, "/new/g;\n");
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
