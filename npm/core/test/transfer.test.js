import assert from "node:assert/strict";
import test from "node:test";
import { encode, encodeGeneratorConfig } from "@swc-next/encoder";
import { generateSync as generateStandalone } from "../../generator/src-js/index.js";
import * as binding from "../src-js/binding.js";
import { parseSync, generateSync } from "../src-js/index.js";

function assertTransfer(program, options, expected) {
  const parts = encode(program);
  const [header] = parts;
  const view = new DataView(header.buffer, header.byteOffset, header.byteLength);
  const nodes = view.getUint32(0, true);
  assert.deepEqual(parts.map(part => part.byteLength), [
    40, nodes * 8, nodes, nodes * 16, view.getUint32(4, true) * 4,
    view.getUint32(8, true) + view.getUint32(12, true),
  ]);
  const optionBits = encodeGeneratorConfig(options);
  assert.equal(binding.generateSync(...parts, optionBits), expected);
  assert.equal(generateSync(program, options).code, expected);
  assert.equal(generateStandalone(program, options).code, expected);
}

test("section transfers preserve source types, strings, and options", () => {
  const minified = { minify: true };
  const ascii = { asciiOnly: true };
  for (const lang of ["js", "ts", "jsx", "tsx", "dts"]) {
    const options = { lang, sourceType: "script" };
    const { program } = parseSync('const values = ["雪🌍", 1];', options);
    assertTransfer(program, undefined, 'const values = ["雪🌍", 1];\n');
    assertTransfer(program, minified, 'const values=["雪🌍",1];');
    assertTransfer(program, ascii, 'const values = ["\\u96EA\\u{1F30D}", 1];\n');

    program.body[0].declarations[0].init.elements[0].value = "a\uD800雪\uDC00";
    program.body[0].declarations[0].init.elements[0].raw = null;
    assertTransfer(program, ascii, 'const values = ["a\\uD800\\u96EA\\uDC00", 1];\n');
  }
});

test("section transfers use current buffers after growth", () => {
  const { program } = parseSync("const values = [];");
  program.body[0].declarations[0].init.elements = Array.from({ length: 2000 }, (_, index) => ({
    type: "Literal", value: `${index}雪🌍\uD800`, raw: null,
  }));
  const generated = generateSync(program, { minify: true }).code;
  const roundtrip = parseSync(generated).program.body[0].declarations[0].init;
  assert.deepEqual(
    roundtrip.elements.map(element => element.value),
    program.body[0].declarations[0].init.elements.map(element => element.value),
  );
  assert.equal(generateStandalone(program, { minify: true }).code, generated);

  const empty = parseSync("");
  assertTransfer(empty.program, undefined, "");
});
