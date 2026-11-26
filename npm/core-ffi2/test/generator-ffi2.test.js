import assert from "node:assert/strict";
import test from "node:test";
import { parseSync, generateSync } from "@swc-next/core-ffi2";

test("generateSync prints modified native views", () => {
  const { program } = parseSync("const answer = 1;");
  program.body.get(0).declarators.get(0).id.name = "changed";
  program.body.get(0).declarators.get(0).init.value = 42;

  assert.equal(generateSync(program).code, "const changed = 42;\n");
});

test("AST preserves modified module attribute types", () => {
  for (const lang of ["ts", "dts"]) {
    const { program } = parseSync("declare module '*.txt' with { type: 'text' } {}", { lang });
    const module = program.body.get(0);
    assert.equal(module.attributes.type, "TsTypeLiteral");
    const value = module.attributes.members.get(0).typeAnnotation.typeAnnotation.literal;
    assert.equal(value.value, "text");
    value.value = "文本";

    const generated = generateSync(program).code;
    const roundtrip = parseSync(generated, { lang }).program.body.get(0);
    assert.equal(roundtrip.attributes.members.get(0).typeAnnotation.typeAnnotation.literal.value, "文本");
    assert.equal(roundtrip.id.value, "*.txt");
    assert.equal(roundtrip.body.type, "TsModuleBlock");
    assert.equal(roundtrip.declare, true);
    assert.equal(roundtrip.kind, "Module");
  }
});

test("generateSync accepts surrogate pairs in UTF-8 fields", () => {
  const { program } = parseSync("const value = 1;");
  program.body.get(0).declarators.get(0).id.name = "\u{10400}";

  assert.equal(generateSync(program).code, "const 𐐀 = 1;\n");
});

test("generateSync preserves special numeric values across native storage", () => {
  const { program } = parseSync("const value = 1;");
  program.body.get(0).declarators.get(0).init.value = -0;

  assert.equal(generateSync(program).code, "const value = -0;\n");

  program.body.get(0).declarators.get(0).init.value = -Infinity;
  assert.equal(generateSync(program).code, "const value = -1 / 0;\n");
});

test("AST edits native directive value and raw text", () => {
  const { program } = parseSync('"use client";');
  const directive = program.directives.get(0);
  directive.value = "use server";
  directive.expression.value = "use server";
  directive.expression.raw = '"use server"';

  assert.equal(generateSync(program).code, '"use server";\n');
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

test("native BigInt and unary fields encode independently", () => {
  const { program } = parseSync("-1n;");
  const unary = program.body.get(0).expression;
  assert.equal(unary.operator, "Negate");
  unary.argument.raw = "2";
  assert.equal(generateSync(program).code, "-2n;\n");
});

test("AST uses modified RegExp metadata", () => {
  const { program } = parseSync("/old/;");
  program.body.get(0).expression.pattern = "new";
  program.body.get(0).expression.flags = "g";

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

test("AST stores JSX attribute value and raw text separately", () => {
  const { program } = parseSync(
    'const element = <div title="old" />;',
    { lang: "tsx" },
  );
  const value = program.body.get(0).declarators.get(0).init.openingElement.attributes.get(0).value;

  value.value = "new";
  value.raw = '"new"';
  assert.equal(generateSync(program).code, 'const element = <div title="new" />;\n');
});


test("generateSync requires a Program root view", () => {
  const ast = parseSync("const value = 1;");
  assert.throws(() => generateSync(ast.materialize()), TypeError);
  assert.throws(() => generateSync(ast.program.body.get(0)), TypeError);
  assert.equal(generateSync(ast.program).code, "const value = 1;\n");
});
