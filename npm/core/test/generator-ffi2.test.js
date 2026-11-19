import assert from "node:assert/strict";
import test from "node:test";
import { parseSync, generateSync } from "@swc-next/core/ffi2";
import { generateSync as generateStandalone } from "@swc-next/generator/ffi2";

test("generateSync prints modified parser-produced ESTree", () => {
  const { program } = parseSync("const answer = 1;");
  program.body.get(0).declarations.get(0).id.name = "changed";
  program.body.get(0).declarations.get(0).init.value = 42;

  assert.equal(generateSync(program).code, "const changed = 42;\n");
});

test("AST preserves modified module attribute types", () => {
  for (const lang of ["ts", "dts"]) {
    const { program } = parseSync("declare module '*.txt' with { type: 'text' } {}", { lang });
    const module = program.body.get(0);
    assert.equal(module.attributes.type, "TSTypeLiteral");
    const value = module.attributes.members.get(0).typeAnnotation.typeAnnotation.literal;
    assert.equal(value.value, "text");
    value.value = "文本";

    const generated = generateSync(program).code;
    const roundtrip = parseSync(generated, { lang }).program.body.get(0);
    assert.equal(roundtrip.attributes.members.get(0).typeAnnotation.typeAnnotation.literal.value, "文本");
    assert.equal(roundtrip.id.value, "*.txt");
    assert.equal(roundtrip.body.type, "TSModuleBlock");
    assert.equal(roundtrip.declare, true);
    assert.equal(roundtrip.kind, "module");
  }
});

test("generateSync accepts surrogate pairs in UTF-8 fields", () => {
  const { program } = parseSync("const value = 1;");
  program.body.get(0).declarations.get(0).id.name = "\u{10400}";

  assert.equal(generateSync(program).code, "const 𐐀 = 1;\n");
});

test("generateSync preserves special numeric values across native storage", () => {
  const { program } = parseSync("const value = 1;");
  program.body.get(0).declarations.get(0).init.value = -0;

  assert.equal(generateSync(program).code, "const value = -0;\n");

  program.body.get(0).declarations.get(0).init.value = -Infinity;
  assert.equal(generateSync(program).code, "const value = -1 / 0;\n");
});

test("AST invalidates stale directive raw text", () => {
  const { program } = parseSync('"use client";');
  program.body.get(0).expression.value = "use server";

  assert.equal(generateSync(program).code, '"use server";\n');

  const updated = parseSync('"use client";').program;
  updated.body.get(0).expression.value = "use server";
  updated.body.get(0).directive = "use server";

  assert.equal(generateSync(updated).code, '"use server";\n');

  const escaped = parseSync('"use\\x20client";').program;
  assert.equal(generateSync(escaped).code, '"use\\x20client";\n');

  escaped.body.get(0).expression.value = "use server";
  assert.equal(generateSync(escaped).code, '"use server";\n');
});

test("AST accepts directive raw escapes with leading zeros", () => {
  const { program } = parseSync('"use\\u{0000020}strict";', {
    sourceType: "script",
  });

  assert.equal(generateSync(program).code, '"use\\u{0000020}strict";\n');
});

test("AST preserves BigInt raw radix", () => {
  const { program } = parseSync("0x1_2_3n;");

  assert.equal(generateSync(program).code, "0x123n;\n");
});

test("AST validates legacy octal raw text", () => {
  const { program } = parseSync("077;", { sourceType: "script" });

  assert.equal(generateSync(program).code, "077;\n");

  program.body.get(0).expression.value = 64;
  assert.equal(generateSync(program).code, "64;\n");
});

test("AST normalizes negative BigInts and property keys", () => {
  const { program } = parseSync("1n;");
  program.body.get(0).expression.value = -1n;
  assert.equal(generateSync(program).code, "-1n;\n");

  const property = parseSync("({ 1n: 0 });").program;
  property.body.get(0).expression.expression.properties.get(0).key.value = -1n;
  assert.equal(generateSync(property).code, '({ [-1n]: 0 });\n');
});

test("AST uses modified RegExp metadata", () => {
  const { program } = parseSync("/old/;");
  program.body.get(0).expression.regex.pattern = "new";
  program.body.get(0).expression.regex.flags = "g";

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

test("AST invalidates stale JSX attribute raw text", () => {
  const { program } = parseSync(
    'const element = <div title="old" />;',
    { lang: "tsx" },
  );
  const value = program.body.get(0).declarations.get(0).init.openingElement.attributes.get(0).value;

  value.value = "new";
  assert.equal(generateSync(program).code, 'const element = <div title="new" />;\n');
});


test("generators require a Program root view", () => {
  const ast = parseSync("const value = 1;");
  for (const generate of [generateSync, generateStandalone]) {
    assert.throws(() => generate(ast.materialize()), TypeError);
    assert.throws(() => generate(ast.program.body.get(0)), TypeError);
    assert.equal(generate(ast.program).code, "const value = 1;\n");
  }
});
