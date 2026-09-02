import assert from "node:assert/strict";
import test from "node:test";

import { decode } from "../src-js/decode.js";

const HEADER_SIZE = 36;

function identifierBuffer(source, stringPool, start, end) {
  const pool = new TextEncoder().encode(stringPool);
  const strings = HEADER_SIZE + 8 + 4 + 16;
  const buffer = new ArrayBuffer(strings + ((pool.length + 3) & ~3));
  const view = new DataView(buffer);

  view.setUint32(0, 1, true);
  view.setUint32(8, pool.length, true);
  view.setUint32(12, new TextEncoder().encode(source).length, true);
  view.setUint32(28, 0, true);
  view.setUint8(HEADER_SIZE + 8, 11);
  view.setUint32(HEADER_SIZE + 8 + 4, start, true);
  view.setUint32(HEADER_SIZE + 8 + 8, end, true);
  new Uint8Array(buffer, strings, pool.length).set(pool);

  return buffer;
}

test("decode reads extra and compact generated strings", () => {
  const extra = identifierBuffer("", "renamed\0", 0x80000000, 7);
  assert.equal(decode(extra, "").program.name, "renamed");

  const compactExtra = identifierBuffer("", "_ref\0", 0xc0000000, 0xc0000004);
  assert.equal(decode(compactExtra, "").program.name, "_refa");

  const compactSource = identifierBuffer("_ref", "\0", 0x40000000, 0xc0000004);
  assert.equal(decode(compactSource, "_ref").program.name, "_refa");
});
