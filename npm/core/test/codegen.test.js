import assert from "node:assert/strict";
import test from "node:test";
import { execFileSync } from "node:child_process";

import { encode, encodeJs, encodeTs } from "@swc-next/encoder";
import { parseSync, printSync } from "../src-js/index.js";

test("encode writes spans in canonical little-endian order", () => {
  const buffer = encode({
    type: "Program",
    start: 0x01020304,
    end: 0x05060708,
    sourceType: "module",
    body: [],
  });

  assert.deepEqual(
    [...buffer.subarray(40, 48)],
    [0x04, 0x03, 0x02, 0x01, 0x08, 0x07, 0x06, 0x05],
  );
});

test("printSync prints modified parser-produced ESTree", () => {
  const { program } = parseSync("const answer = 1;");
  program.body[0].declarations[0].id.name = "changed";
  program.body[0].declarations[0].init.value = 42;

  assert.equal(printSync(program).code, "const changed = 42;\n");
});

test("encoder preserves array rests", () => {
  const print = (source, lang) => printSync(parseSync(source).program, {}, { lang }).code;

  assert.equal(print("const [...a, b] = v;", "js"), "const [...a, b] = v;\n");
  assert.equal(print("const [...a, b] = v;", "ts"), "const [...a, b] = v;\n");

  assert.equal(print("const [...a, b, ...c] = v;", "js"), "const [...a, b, ...c] = v;\n");
  assert.equal(print("const [...a, b, ...c] = v;", "ts"), "const [...a, b, ...c] = v;\n");
});

test("printSync accepts surrogate pairs in UTF-8 fields", () => {
  const { program } = parseSync("const value = 1;");
  program.body[0].declarations[0].id.name = "\u{10400}";

  assert.equal(printSync(program).code, "const 𐐀 = 1;\n");
});

test("printSync preserves lone surrogates in string values", () => {
  for (const lang of ["js", "ts"]) {
    const { program } = parseSync('const text = "";', { lang });
    const value = "\uD800é😀\0\uDC00";
    program.body[0].declarations[0].init.value = value;

    const buffer = encode(program, { lang });
    const header = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
    assert.ok(header.getUint32(8, true) > 0);
    assert.equal(header.getUint32(12, true), 0);
    const printed = printSync(program, {}, { lang });
    assert.equal(parseSync(printed.code).program.body[0].declarations[0].init.value, value);

    const escaped = parseSync('const text = "\\uD800";', { lang }).program;
    const roundtrip = parseSync(printSync(escaped, {}, { lang }).code).program;
    assert.equal(roundtrip.body[0].declarations[0].init.value, "\uD800");
  }
});

test("printSync preserves special numeric values across FFI", () => {
  const { program } = parseSync("const value = 1;");
  program.body[0].declarations[0].init.value = -0;

  assert.equal(printSync(program).code, "const value = -0;\n");

  program.body[0].declarations[0].init.value = -Infinity;
  assert.equal(printSync(program).code, "const value = -1 / 0;\n");
});

test("printSync falls back to values for empty numeric raw text", () => {
  for (const lang of ["js", "ts"]) {
    const meta = { lang };
    const printLiteral = (value, raw) => {
      const { program } = parseSync(typeof value === "bigint" ? "0n;" : "0;", meta);
      Object.assign(program.body[0].expression, { value, raw });
      return printSync(program, {}, meta).code;
    };

    assert.equal(printLiteral(0, ""), "0;\n");
    assert.equal(printLiteral(-0, ""), "-0;\n");
    assert.equal(printLiteral(0n, ""), "0n;\n");
    assert.equal(printLiteral(0n, "n"), "0n;\n");

    assert.equal(printLiteral(42, "1"), "42;\n");
    assert.equal(printLiteral(2n, "1n"), "2n;\n");
    assert.equal(printLiteral(16, "0x10"), "0x10;\n");
    assert.equal(printLiteral(16n, "0x10n"), "0x10n;\n");
  }
});

test("encoder rebuilds the directive prologue after reordering", () => {
  const { program } = parseSync('"use strict"; work();');
  program.body.reverse();

  assert.equal(printSync(program).code, 'work();\n"use strict";\n');
});

test("encoder invalidates stale directive raw text", () => {
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

test("encoder preserves expressions with stale directive metadata", () => {
  const script = parseSync("1;").program;
  script.body[0].directive = "foo";
  assert.equal(printSync(script, {}, { lang: "js" }).code, "1;\n");

  const modified = parseSync('"use strict";', { lang: "ts" }).program;
  modified.body[0].expression.value = 1;
  assert.equal(printSync(modified, {}, { lang: "ts" }).code, "1;\n");

  const functionBody = parseSync("function f() { 1; }").program;
  functionBody.body[0].body.body[0].directive = "foo";
  assert.equal(printSync(functionBody, {}, { lang: "js" }).code, "function f() {\n  1;\n}\n");

  const namespace = parseSync("namespace N { 1; }", { lang: "ts" }).program;
  namespace.body[0].body.body[0].directive = "foo";
  assert.equal(printSync(namespace).code, "namespace N {\n  1;\n}\n");
});

test("encoder preserves method bodies and signatures", () => {
  const typescript = { lang: "ts" };

  const body = parseSync("abstract class C { abstract m() { return 1; } }", typescript).program;
  assert.equal(printSync(body).code, "abstract class C {\n  abstract m() {\n    return 1;\n  }\n}\n");
  assert.equal((new DataView(encode(body).buffer).getUint32(32, true) >> 2) & 7, 1);

  const abstractSignature = parseSync("abstract class C { abstract m(): number; }", typescript).program;
  assert.equal(printSync(abstractSignature).code, "abstract class C {\n  abstract m(): number;\n}\n");

  const signature = parseSync("class C { m(): number; }", typescript).program;
  assert.equal(printSync(signature).code, "class C {\n  m(): number;\n}\n");

  const modified = parseSync("class C { m() { return 1; } }").program;
  modified.body[0].body.body[0].type = "TSAbstractMethodDefinition";
  assert.equal(printSync(modified).code, "class C {\n  abstract m() {\n    return 1;\n  }\n}\n");
});

test("encoder accepts directive raw escapes with leading zeros", () => {
  const { program } = parseSync('"use\\u{0000020}strict";', {
    sourceType: "script",
  });

  assert.equal(printSync(program).code, '"use\\u{0000020}strict";\n');
});

test("encoder preserves BigInt raw radix", () => {
  const { program } = parseSync("0x1_2_3n;");

  assert.equal(printSync(program).code, "0x123n;\n");
});

test("encoder validates legacy octal raw text", () => {
  const { program } = parseSync("077;", { sourceType: "script" });

  assert.equal(printSync(program).code, "077;\n");

  program.body[0].expression.value = 64;
  assert.equal(printSync(program).code, "64;\n");
});

test("encoder normalizes negative BigInts and property keys", () => {
  const { program } = parseSync("1n;");
  program.body[0].expression.value = -1n;
  assert.equal(printSync(program).code, "-1n;\n");

  const property = parseSync("({ 1n: 0 });").program;
  property.body[0].expression.expression.properties[0].key.value = -1n;
  assert.equal(printSync(property).code, '({ "-1": 0 });\n');
});

test("encoder uses modified RegExp metadata", () => {
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

  const script = parseSync('const text = "</ScRiPt>";').program;
  assert.equal(
    printSync(script, { inlineScript: false }).code,
    'const text = "</ScRiPt>";\n',
  );
  assert.equal(
    printSync(script, { inlineScript: true }).code,
    'const text = "<\\/ScRiPt>";\n',
  );
});

test("encoder invalidates stale JSX attribute raw text", () => {
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

test("encoder metadata selects compatible JS and TS variants", () => {
  const { program } = parseSync("const value = (a) => a;");
  assert.deepEqual(encodeJs(program), encodeTs(program));
  assert.deepEqual(encode(program, { lang: "js" }), encodeJs(program));
  assert.deepEqual(encode(program, { lang: "jsx" }), encodeJs(program, { lang: "jsx" }));
  assert.deepEqual(encode(program), encodeTs(program));

  const typed = parseSync("const value: string = 'ok';", { lang: "ts" }).program;
  assert.deepEqual(encode(typed, { lang: "ts" }), encodeTs(typed));
  assert.deepEqual(encode(typed, { lang: "tsx" }), encodeTs(typed, { lang: "tsx" }));
  assert.equal((new DataView(encodeTs(typed).buffer).getUint32(32, true) >> 2) & 7, 1);
  assert.equal((new DataView(encode(typed, { lang: "tsx" }).buffer).getUint32(32, true) >> 2) & 7, 3);
});


function encodeWithBufferMode(program, mode) {
  const script = `
    import { Buffer as NodeBuffer } from "node:buffer";
    import { readFileSync } from "node:fs";
    const { program, mode } = JSON.parse(readFileSync(0, "utf8"));
    if (mode === "absent") delete String.prototype.isWellFormed;
    globalThis.Buffer = mode === "absent" ? undefined
      : mode === "missing" ? { prototype: {} }
      : { prototype: { write() { throw new TypeError("unsupported receiver"); } } };
    const { encodeJs, encodeTs } = await import(${JSON.stringify(new URL("../../../packages/encoder/index.js", import.meta.url).href)});
    console.log(JSON.stringify([encodeJs(program), encodeTs(program)].map(bytes =>
      NodeBuffer.from(bytes).toString("base64"))));
  `;
  const output = execFileSync(process.execPath, ["--input-type=module", "-e", script], {
    input: JSON.stringify({ program, mode }), encoding: "utf8", maxBuffer: 4 * 1024 * 1024,
  });
  return JSON.parse(output).map(value => Uint8Array.from(Buffer.from(value, "base64")));
}

test("encoder string fallbacks match Node", () => {
  const { program } = parseSync('"prefix"; "value"; "tail";');
  program.body[1].expression.value = "ab雪🌍\uD800x\uDFFF";
  program.body[1].expression.raw = null;
  const expected = encodeJs(program);
  for (const mode of ["absent", "missing", "incompatible"]) {
    const [js, ts] = encodeWithBufferMode(program, mode);
    assert.deepEqual(js, expected);
    assert.deepEqual(ts, expected);
  }

  program.body[1].expression.value = "ab雪🌍";
  const utf8 = encodeJs(program);
  const [js, ts] = encodeWithBufferMode(program, "absent");
  assert.deepEqual(js, utf8);
  assert.deepEqual(ts, utf8);
});
