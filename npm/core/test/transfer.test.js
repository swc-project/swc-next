import assert from "node:assert/strict";
import test from "node:test";
import { encode, encodeOptions } from "@swc-next/encoder";
import { printSync as printStandalone } from "../../codegen/src-js/index.js";
import * as binding from "../src-js/binding.js";
import { parseSync, printSync } from "../src-js/index.js";

function assertTransfer(program, meta, options, expected) {
  const parts = encode(program, meta);
  const [header] = parts;
  const view = new DataView(header.buffer, header.byteOffset, header.byteLength);
  const nodes = view.getUint32(0, true);
  assert.deepEqual(parts.map(part => part.byteLength), [
    40, nodes * 8, nodes, nodes * 16, view.getUint32(4, true) * 8,
    view.getUint32(8, true) + view.getUint32(12, true),
  ]);
  const optionBits = encodeOptions(options);
  assert.equal(binding.printSync(...parts, optionBits), expected);
  assert.equal(printSync(program, options, meta).code, expected);
  assert.equal(printStandalone(program, options, meta).code, expected);
}

test("section transfers preserve metadata, strings, and options", () => {
  const minified = { minify: true };
  const ascii = { asciiOnly: true };
  for (const lang of ["js", "ts", "jsx", "tsx", "dts"]) {
    const meta = { lang, sourceType: "script" };
    const { program } = parseSync('const values = ["雪🌍", 1];', meta);
    assertTransfer(program, meta, undefined, 'const values = ["雪🌍", 1];\n');
    assertTransfer(program, meta, minified, 'const values=["雪🌍",1];');
    assertTransfer(program, meta, ascii, 'const values = ["\\u96EA\\u{1F30D}", 1];\n');

    program.body[0].declarations[0].init.elements[0].value = "a\uD800雪\uDC00";
    program.body[0].declarations[0].init.elements[0].raw = null;
    assertTransfer(program, meta, ascii, 'const values = ["a\\uD800\\u96EA\\uDC00", 1];\n');
  }
});

test("section transfers use current buffers after growth", () => {
  const { program, meta } = parseSync("const values = [];");
  program.body[0].declarations[0].init.elements = Array.from({ length: 2000 }, (_, index) => ({
    type: "Literal", value: `${index}雪🌍\uD800`, raw: null,
  }));
  const printed = printSync(program, { minify: true }, meta).code;
  const roundtrip = parseSync(printed).program.body[0].declarations[0].init;
  assert.deepEqual(
    roundtrip.elements.map(element => element.value),
    program.body[0].declarations[0].init.elements.map(element => element.value),
  );
  assert.equal(printStandalone(program, { minify: true }, meta).code, printed);

  const empty = parseSync("");
  assertTransfer(empty.program, empty.meta, undefined, "");
});
