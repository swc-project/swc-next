import assert from "node:assert/strict";
import test from "node:test";

import { decode, encodeOptions as encodeParserOptions } from "@swc-next/decoder";
import { encode, encodeOptions as encodeCodegenOptions } from "@swc-next/encoder";
import {
  parseSync,
  printSync,
} from "../../../target/wasm-package/core/swc-next-core.node.mjs";

test("WASM exposes parseSync and printSync", () => {
  const source = "const value = 1;";
  const buffer = parseSync(source, encodeParserOptions());
  const { program } = decode(buffer, source);
  program.body[0].declarations[0].init.value = 2;

  assert.equal(printSync(encode(program), encodeCodegenOptions()), "const value = 2;\n");
});
