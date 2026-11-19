import assert from "node:assert/strict";
import test from "node:test";
import { encodeGeneratorConfig } from "@swc-next/protocol/generator";
import { generateSync as generateStandalone, generateBuffersSync as generateStandaloneBuffers } from "@swc-next/generator/ffi2";
import * as binding from "../dist/binding.js";
import { parseSync, generateSync, generateBuffersSync, NodeKind } from "@swc-next/core/ffi2";

function assertTransfer(ast, options, expected) {
  const buffers = ast.encode();
  const config = encodeGeneratorConfig(options);
  assert.equal(binding.generateSync2(...buffers, config), expected);
  assert.equal(generateBuffersSync(...buffers, options).code, expected);
  assert.equal(generateStandaloneBuffers(...buffers, options).code, expected);
  assert.equal(generateSync(ast.program, options).code, expected);
  assert.equal(generateStandalone(ast.program, options).code, expected);
}

test("buffer transfers preserve source types, strings, and options", () => {
  const minified = { minify: true };
  const ascii = { asciiOnly: true };
  for (const lang of ["js", "ts", "jsx", "tsx", "dts"]) {
    const ast = parseSync('const values = ["雪🌍", 1];', { lang, sourceType: "script" });
    assertTransfer(ast, undefined, 'const values = ["雪🌍", 1];\n');
    assertTransfer(ast, minified, 'const values=["雪🌍",1];');
    assertTransfer(ast, ascii, 'const values = ["\\u96EA\\u{1F30D}", 1];\n');

    ast.program.body.get(0).declarations.get(0).init.elements.get(0).value = "a\uD800雪\uDC00";
    assertTransfer(ast, ascii, 'const values = ["a\\uD800\\u96EA\\uDC00", 1];\n');
  }
});

test("buffer transfers preserve edits after arena growth", () => {
  const ast = parseSync("const values = [];");
  const elements = ast.program.body.get(0).declarations.get(0).init.elements;
  const expected = [];
  for (let index = 0; index < 2000; index++) {
    const node = ast.new_node(NodeKind.StringLiteral);
    node.value = `${index}雪🌍\uD800`;
    expected.push(node.value);
    elements.push(node);
  }
  const generated = generateSync(ast.program, { minify: true }).code;
  const roundtrip = parseSync(generated).program.body.get(0).declarations.get(0).init;
  assert.deepEqual(roundtrip.elements.map(element => element.value), expected);
  assertTransfer(ast, { minify: true }, generated);

  assertTransfer(parseSync(""), undefined, "");
});
