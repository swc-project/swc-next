import assert from "node:assert/strict";
import test from "node:test";
import { execFileSync } from "node:child_process";

import { encode, encodeJs, encodeTs, encodeGeneratorConfig } from "@swc-next/encoder";
import { decodeJs } from "@swc-next/decoder";
import { parseSync, generateSync } from "../src-js/index.js";
import { generateSync as generateNative } from "../src-js/binding.js";

function generateEncoded(program, encoder) {
  return generateNative(...encoder(program), encodeGeneratorConfig());
}

function encodedParameterKinds(source, encoder, options) {
  const { program } = parseSync(source, options);
  const [, , kinds, payloads] = encoder(program);
  const view = new DataView(payloads.buffer, payloads.byteOffset, payloads.byteLength);
  const parameterKinds = ["formal", "unique", "arrow", "signature"];
  // FormalParameters has wire tag 58; its kind follows items and rest at byte 12.
  return Array.from(kinds).flatMap((kind, index) =>
    kind === 58 ? [parameterKinds[view.getUint8(index * 16 + 12)]] : [],
  );
}

function decodeEncoded(program, source) {
  const [header, spans, kinds, payloads, extra, strings] = encodeJs(program);
  const buffer = Buffer.concat([
    header, spans, kinds, Buffer.alloc((8 - kinds.length % 8) % 8),
    payloads, extra, strings, Buffer.alloc((4 - strings.length % 4) % 4),
  ]);
  // Keep encoded strings in the pool, independently of the original source.
  buffer.writeUInt32LE(strings.length, 8);
  buffer.writeUInt32LE(0, 12);
  buffer.writeUInt32LE(0, 36);
  return decodeJs(buffer, source, new TextEncoder().encode(source)).program;
}

function assertEncodedRoundtrip(source) {
  const { program } = parseSync(source);
  const expected = structuredClone(program);
  const decoded = decodeEncoded(program, source);
  assert.deepEqual(decoded, expected);

  const repeated = decodeEncoded(decoded, source);
  assert.deepEqual(repeated, expected);
  assert.deepEqual(program, expected);
}

function encodedLanguage(program) {
  const [header] = encode(program);
  const view = new DataView(header.buffer, header.byteOffset, header.byteLength);
  return (view.getUint32(32, true) >> 2) & 7;
}

test("encoder keeps JavaScript language for default fields", () => {
  const typescript = { lang: "ts" };
  const variable = parseSync("const value = 1;", typescript).program;
  assert.equal(encodedLanguage(variable), 0);

  const classDeclaration = parseSync("class C { value; method() {} }", typescript).program;
  assert.equal(encodedLanguage(classDeclaration), 0);
  assert.deepEqual(encodeJs(classDeclaration), encodeTs(classDeclaration));

  const importDeclaration = parseSync('import { T } from "m";', typescript).program;
  assert.equal(encodedLanguage(importDeclaration), 0);

  const exportDeclaration = parseSync("export { T };", typescript).program;
  assert.equal(encodedLanguage(exportDeclaration), 0);

  const optionalAccess = parseSync("object?.value;").program;
  assert.equal(encodedLanguage(optionalAccess), 0);

  const optionalCall = parseSync("object?.method();").program;
  assert.equal(encodedLanguage(optionalCall), 0);

  const decoratedClass = parseSync("@dec class C {}").program;
  assert.equal(encodedLanguage(decoratedClass), 0);

  const decoratedMethod = parseSync("class C { @dec method() {} }").program;
  assert.equal(encodedLanguage(decoratedMethod), 0);
});

test("encoder preserves empty parameter spans", () => {
  const { program } = parseSync("function f(a? = 0) {}", { lang: "ts" });
  const pattern = program.body[0].params[0].left;
  pattern.start = pattern.end = 0;

  const [, spans, kinds, payloads] = encode(program);
  const payloadView = new DataView(payloads.buffer, payloads.byteOffset, payloads.byteLength);
  const patternId = payloadView.getUint32(kinds.indexOf(59) * 16, true);
  const spanView = new DataView(spans.buffer, spans.byteOffset, spans.byteLength);
  assert.equal(spanView.getUint32(patternId * 8, true), 0);
  assert.equal(spanView.getUint32(patternId * 8 + 4, true), 0);
});

test("encodeJs preserves JavaScript template spans", () => {
  assertEncodedRoundtrip("`abc`;");

  assertEncodedRoundtrip("`a${x}b${y}c`;");

  assertEncodedRoundtrip("`${x}`;");

  assertEncodedRoundtrip("``;");

  assertEncodedRoundtrip("`a\r\nb\r\nc`;");

  assertEncodedRoundtrip("tag`\\xZ${x}雪🌍`;");
});

test("encoder preserves empty template spans", () => {
  const { program } = parseSync("`abc`;", { lang: "ts" });
  const quasi = program.body[0].expression.quasis[0];
  quasi.start = quasi.end = 0;

  const [, spans, kinds] = encode(program);
  const spanView = new DataView(spans.buffer, spans.byteOffset, spans.byteLength);
  const index = kinds.indexOf(10) * 8;
  assert.equal(spanView.getUint32(index, true), 0);
  assert.equal(spanView.getUint32(index + 4, true), 0);
});

test("encoders preserve parameter contexts", () => {
  const script = { sourceType: "script" };
  for (const encoder of [encodeJs, encodeTs]) {
    assert.deepEqual(encodedParameterKinds("function f(a) {}", encoder, script), ["formal"]);

    assert.deepEqual(encodedParameterKinds("(a) => a;", encoder, script), ["arrow"]);

    assert.deepEqual(encodedParameterKinds("({ m(a) {} });", encoder, script), ["unique"]);

    assert.deepEqual(encodedParameterKinds("({ m: function(a) {} });", encoder, script), ["formal"]);

    assert.deepEqual(encodedParameterKinds("({ get value() {} });", encoder, script), ["unique"]);

    assert.deepEqual(encodedParameterKinds("({ set value(a) {} });", encoder, script), ["unique"]);

    assert.deepEqual(encodedParameterKinds("class C { m(a) {} }", encoder, script), ["unique"]);

    assert.deepEqual(encodedParameterKinds("class C { constructor(a) {} }", encoder, script), ["unique"]);
  }
});

test("encoder preserves TypeScript parameter contexts", () => {
  const typescript = { lang: "ts" };
  assert.deepEqual(encodedParameterKinds("declare function f(a: number): void;", encode, typescript), ["formal"]);

  assert.deepEqual(encodedParameterKinds("declare class C { m(a: number): void; }", encode, typescript), ["unique"]);

  assert.deepEqual(encodedParameterKinds("abstract class C { abstract m(a: number): void; }", encode, typescript), ["unique"]);

  assert.deepEqual(encodedParameterKinds("type F = (a: number) => void;", encode, typescript), ["signature"]);

  assert.deepEqual(encodedParameterKinds("type F = new (a: number) => object;", encode, typescript), ["signature"]);

  assert.deepEqual(encodedParameterKinds("interface I { m(this: I, a: number): void; }", encode, typescript), ["signature"]);

  assert.deepEqual(encodedParameterKinds("interface I { (a: number): void; }", encode, typescript), ["signature"]);

  assert.deepEqual(encodedParameterKinds("interface I { new (a: number): object; }", encode, typescript), ["signature"]);
});

test("encoders recover from errors", () => {
  for (const encoder of [encodeJs, encodeTs]) {
    const broken = parseSync('const value = "雪";').program;
    broken.body.push({ type: "Unsupported" });
    assert.throws(() => encoder(broken), TypeError);

    const valid = parseSync('const value = "valid";').program;
    assert.equal(generateEncoded(valid, encoder), 'const value = "valid";\n');
  }
});

test("encode writes spans in canonical little-endian order", () => {
  const [, spans] = encode({
    type: "Program",
    start: 0x01020304,
    end: 0x05060708,
    sourceType: "module",
    body: [],
  });

  assert.deepEqual(
    [...spans.subarray(0, 8)],
    [0x04, 0x03, 0x02, 0x01, 0x08, 0x07, 0x06, 0x05],
  );
});

test("generateSync prints modified parser-produced ESTree", () => {
  const { program } = parseSync("const answer = 1;");
  program.body[0].declarations[0].id.name = "changed";
  program.body[0].declarations[0].init.value = 42;

  assert.equal(generateSync(program).code, "const changed = 42;\n");
});

test("encoder preserves modified module attribute types", () => {
  for (const lang of ["ts", "dts"]) {
    const { program } = parseSync("declare module '*.txt' with { type: 'text' } {}", { lang });
    const module = program.body[0];
    assert.equal(module.attributes.type, "TSTypeLiteral");
    const value = module.attributes.members[0].typeAnnotation.typeAnnotation.literal;
    assert.equal(value.value, "text");
    value.value = "文本";

    const generated = generateSync(program).code;
    const roundtrip = parseSync(generated, { lang }).program.body[0];
    assert.equal(roundtrip.attributes.members[0].typeAnnotation.typeAnnotation.literal.value, "文本");
    assert.equal(roundtrip.id.value, "*.txt");
    assert.equal(roundtrip.body.type, "TSModuleBlock");
    assert.equal(roundtrip.declare, true);
    assert.equal(roundtrip.kind, "module");
  }
});

test("encoder normalizes template raw line endings", () => {
  for (const encoder of [encodeJs, encodeTs]) {
    const { program } = parseSync("tag`placeholder`;");
    const value = program.body[0].expression.quasi.quasis[0].value;

    value.raw = "a\rb\r\nc";
    assert.equal(generateEncoded(program, encoder), "tag`a\nb\nc`;\n");

    value.raw = "a\\\r\nb";
    assert.equal(generateEncoded(program, encoder), "tag`a\\\nb`;\n");

    value.raw = "a\\rb";
    assert.equal(generateEncoded(program, encoder), "tag`a\\rb`;\n");

    value.raw = "\\xZ\r";
    value.cooked = null;
    assert.equal(generateEncoded(program, encoder), "tag`\\xZ\n`;\n");
  }
});

test("encoder preserves array rests", () => {
  const generate = (source, encoder) => generateEncoded(parseSync(source, { errorRecovery: true }).program, encoder);

  for (const encoder of [encodeJs, encodeTs]) {
    assert.equal(generate("const [...a, b] = v;", encoder), "const [...a, b] = v;\n");

    assert.equal(generate("const [...a, b, ...c] = v;", encoder), "const [...a, b, ...c] = v;\n");
  }
});

test("generateSync accepts surrogate pairs in UTF-8 fields", () => {
  const { program } = parseSync("const value = 1;");
  program.body[0].declarations[0].id.name = "\u{10400}";

  assert.equal(generateSync(program).code, "const 𐐀 = 1;\n");
});

test("encoders preserve lone surrogates in string values", () => {
  for (const encoder of [encodeJs, encodeTs]) {
    const { program } = parseSync('const text = "";');
    const value = "\uD800é😀\0\uDC00";
    program.body[0].declarations[0].init.value = value;

    const [buffer] = encoder(program);
    const header = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
    assert.ok(header.getUint32(8, true) > 0);
    assert.equal(header.getUint32(12, true), 0);
    const generated = generateEncoded(program, encoder);
    assert.equal(parseSync(generated).program.body[0].declarations[0].init.value, value);
  }
});

test("generateSync preserves special numeric values across FFI", () => {
  const { program } = parseSync("const value = 1;");
  program.body[0].declarations[0].init.value = -0;

  assert.equal(generateSync(program).code, "const value = -0;\n");

  program.body[0].declarations[0].init.value = -Infinity;
  assert.equal(generateSync(program).code, "const value = -1 / 0;\n");
});

test("encoder rebuilds the directive prologue after reordering", () => {
  const { program } = parseSync('"use strict"; work();');
  program.body.reverse();

  assert.equal(generateSync(program).code, 'work();\n"use strict";\n');
});

test("encoder invalidates stale directive raw text", () => {
  const { program } = parseSync('"use client";');
  program.body[0].expression.value = "use server";

  assert.equal(generateSync(program).code, '"use server";\n');

  const updated = parseSync('"use client";').program;
  updated.body[0].expression.value = "use server";
  updated.body[0].directive = "use server";

  assert.equal(generateSync(updated).code, '"use server";\n');

  const escaped = parseSync('"use\\x20client";').program;
  assert.equal(generateSync(escaped).code, '"use\\x20client";\n');

  escaped.body[0].expression.value = "use server";
  assert.equal(generateSync(escaped).code, '"use server";\n');
});

test("encoder preserves expressions with stale directive metadata", () => {
  const script = parseSync("1;").program;
  script.body[0].directive = "foo";
  assert.equal(generateSync(script).code, "1;\n");

  const modified = parseSync('"use strict";', { lang: "ts" }).program;
  modified.body[0].expression.value = 1;
  assert.equal(generateSync(modified).code, "1;\n");

  const functionBody = parseSync("function f() { 1; }").program;
  functionBody.body[0].body.body[0].directive = "foo";
  assert.equal(generateSync(functionBody).code, "function f() {\n  1;\n}\n");

  const namespace = parseSync("namespace N { 1; }", { lang: "ts" }).program;
  namespace.body[0].body.body[0].directive = "foo";
  assert.equal(generateSync(namespace).code, "namespace N {\n  1;\n}\n");
});

test("encoder preserves modified method types", () => {
  const modified = parseSync("class C { m() { return 1; } }").program;
  modified.body[0].body.body[0].type = "TSAbstractMethodDefinition";
  assert.equal(generateSync(modified).code, "class C {\n  abstract m() {\n    return 1;\n  }\n}\n");
});

test("encoder accepts directive raw escapes with leading zeros", () => {
  const { program } = parseSync('"use\\u{0000020}strict";', {
    sourceType: "script",
  });

  assert.equal(generateSync(program).code, '"use\\u{0000020}strict";\n');
});

test("encoder preserves BigInt raw radix", () => {
  const { program } = parseSync("0x1_2_3n;");

  assert.equal(generateSync(program).code, "0x123n;\n");
});

test("encoder validates legacy octal raw text", () => {
  const { program } = parseSync("077;", { sourceType: "script" });

  assert.equal(generateSync(program).code, "077;\n");

  program.body[0].expression.value = 64;
  assert.equal(generateSync(program).code, "64;\n");
});

test("encoder normalizes negative BigInts and property keys", () => {
  const { program } = parseSync("1n;");
  program.body[0].expression.value = -1n;
  assert.equal(generateSync(program).code, "-1n;\n");

  const property = parseSync("({ 1n: 0 });").program;
  property.body[0].expression.expression.properties[0].key.value = -1n;
  assert.equal(generateSync(property).code, '({ "-1": 0 });\n');
});

test("encoder uses modified RegExp metadata", () => {
  const { program } = parseSync("/old/;");
  program.body[0].expression.regex.pattern = "new";
  program.body[0].expression.regex.flags = "g";

  assert.equal(generateSync(program).code, "/new/g;\n");
});

test("generateSync forwards generator options", () => {
  const { program } = parseSync("const emoji = '😀';");

  assert.equal(
    generateSync(program, { asciiOnly: true, minify: true }).code,
    'const emoji="\\u{1F600}";',
  );

  const script = parseSync('const text = "</ScRiPt>";').program;
  assert.equal(
    generateSync(script, { inlineScript: false }).code,
    'const text = "</ScRiPt>";\n',
  );
  assert.equal(
    generateSync(script, { inlineScript: true }).code,
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
  assert.equal(generateSync(program).code, 'const element = <div title="new" />;\n');
});

test("generateSync rejects unsupported ESTree nodes", () => {
  assert.throws(
    () =>
      generateSync({
        type: "Program",
        sourceType: "script",
        body: [{ type: "Unsupported" }],
      }),
    /Unsupported ESTree node type Unsupported/,
  );
});

test("encoders infer language and preserve source type from ESTree", () => {
  const { program } = parseSync("const value = (a) => a;");
  assert.deepEqual(encodeJs(program), encodeTs(program));
  assert.deepEqual(encode(program), encodeTs(program));

  for (const lang of ["ts", "tsx"]) {
    const typed = parseSync("const value: string = 'ok';", { lang }).program;
    assert.deepEqual(encode(typed), encodeTs(typed));
    const [header] = encode(typed);
    assert.equal((new DataView(header.buffer).getUint32(32, true) >> 2) & 7, 1);
  }

  program.sourceType = "commonjs";
  const [updatedHeader] = encode(program);
  assert.equal(new DataView(updatedHeader.buffer).getUint32(32, true) & 3, 2);
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
    console.log(JSON.stringify([encodeJs(program), encodeTs(program)].map(parts =>
      parts.map(bytes => NodeBuffer.from(bytes).toString("base64")))));
  `;
  const output = execFileSync(process.execPath, ["--input-type=module", "-e", script], {
    input: JSON.stringify({ program, mode }), encoding: "utf8", maxBuffer: 4 * 1024 * 1024,
  });
  return JSON.parse(output).map(parts => parts.map(value => Uint8Array.from(Buffer.from(value, "base64"))));
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
